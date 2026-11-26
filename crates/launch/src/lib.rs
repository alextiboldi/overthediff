//! Single-instance launch, argument hand-off and argument dispatch (E01-S1).
//!
//! [`launch`] runs first in every `otd` process, before any window exists. It finds the
//! [`Handler`] that claims the command line, then runs that handler in this process, forwards the
//! request to the running instance, or makes this process the instance. Only an instance opens the
//! main window. Features add a handler to the app's handler list; nothing here knows about views.
//!
//! The running instance is the one that holds `otd-<major>.primary` in the per-user runtime
//! folder, so two major versions run side by side (F-27.1). It publishes a craft-ipc instance
//! record, and a later launch reaches it through that record's local socket or named pipe.

use std::fs::{File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::{Duration, Instant};

use craft_ipc::client::Client;
use craft_ipc::server::{Activate, Activation, ControlSink, FrameOpen, Frames, Server, ServerConfig};
use craft_ipc::{InstanceRecord, LinkError, RuntimeDir};
use serde_json::{Map, Value, json};

/// Exit code of a launch whose arguments no handler claims (*inferred*, E01-S1).
pub const EXIT_UNCLAIMED: i32 = 100;
/// Exit code of a launch whose hand-off failed, or whose waited-for view went away unreported.
pub const EXIT_FAILED: i32 = 1;

const APP: &str = "otd";
const OPEN: &str = "otd.open";
const CLOSED: &str = "otd.closed";
const ATTACHMENT_PREFIX: &str = "otd-attach-";
/// How long a launch keeps looking for the running instance before it gives up.
const FIND_DEADLINE: Duration = Duration::from_secs(10);
/// Pause between looks while the running instance is still starting.
const FIND_PAUSE: Duration = Duration::from_millis(20);
/// How long an instance that quits keeps its connections open after a view's close was reported,
/// so the report reaches the waiting launcher.
// ponytail: a fixed grace, because craft-ipc's `Server::shutdown` drops queued events and has no
// flush (OverTheDiff issue #14). Its writer sends within about 30 ms; switch to a flush once it has one.
const CLOSE_GRACE: Duration = Duration::from_millis(500);

/// How a handler's launch runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Hand the request to the running instance, or become the instance, and exit at once with 0.
    ForwardAndExit,
    /// Run [`Handler::run_here`] in the launching process: no main window, nothing forwarded.
    RunHere,
    /// Hand the request to the running instance and exit when its view closes, with the code the
    /// handler reports through [`Completion::finish`].
    ForwardAndWait,
}

/// Which arguments a handler claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    /// A switch, named without its prefix: `Switch("qc")` claims `-qc` and `-qc=...`, and on
    /// Windows also `/qc`. A matched switch claims the whole command line.
    Switch(&'static str),
    /// Arguments that are not switches. Claims the command line when no switch matched.
    BarePaths,
    /// One or two arguments that are not switches, each an existing folder, such as two folders to
    /// compare. Claims the command line before [`Pattern::BarePaths`], whatever the registration
    /// order, so a feature that opens files and one that opens folders both take bare paths.
    /// ponytail: a folder is what `Path::is_dir` finds from the launching process's working folder;
    /// a URL or a path inside an archive goes to the bare-paths handler.
    Folders,
}

/// What a launch asks for, whichever handler claims its arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Requested {
    /// The handler's own [`Mode`].
    #[default]
    Default,
    /// Forward-and-wait instead of forward-and-exit.
    Wait,
    /// Become a new instance of its own instead of handing the request off.
    NewInstance,
}

/// What [`Handler::run_here`] decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunHere {
    /// The launching process exits with this code.
    Exit(i32),
    /// Open the request's view in the running instance and wait for it (forward-and-wait).
    HandOff,
}

/// One launch's arguments, as the instance receives them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The id of the handler that claimed the arguments; `None` for a launch with no arguments.
    pub handler: Option<String>,
    pub args: Vec<String>,
    /// The launching process's working folder; relative paths in `args` are relative to it.
    pub cwd: PathBuf,
    /// Content the handler read in the launching process (see [`Handler::attach`]).
    pub attachment: Option<Vec<u8>>,
}

/// A feature's claim on launch arguments. `S` is the shell the instance-side [`Handler::open`]
/// acts on, so this crate needs no UI.
pub trait Handler<S: ?Sized> {
    /// A stable id, unique among handlers.
    fn id(&self) -> &'static str;
    fn pattern(&self) -> Pattern;
    fn mode(&self) -> Mode {
        Mode::ForwardAndExit
    }
    /// This handler's line in the usage message.
    fn usage(&self) -> &'static str;
    /// In the launching process, first: why `args` are wrong for this handler, if they are. The
    /// launch then prints the message and the usage and exits with [`EXIT_UNCLAIMED`].
    fn check(&self, _args: &[String]) -> Result<(), String> {
        Ok(())
    }
    /// In the launching process, before the hand-off: content to send with the request, such as
    /// standard input.
    fn attach(&self, _args: &[String]) -> io::Result<Option<Vec<u8>>> {
        Ok(None)
    }
    /// In the launching process, for [`Mode::RunHere`]. It may show its own progress window and
    /// prompts; no main window exists.
    fn run_here(&self, _request: &Request) -> RunHere {
        RunHere::HandOff
    }
    /// In the instance: act on the request. When its view closes, `done` reports the code a
    /// waiting launcher exits with.
    fn open(&self, request: Request, done: Completion, shell: &mut S);
}

/// The registered handlers, in registration order.
pub struct Registry<S: ?Sized> {
    handlers: Vec<Box<dyn Handler<S>>>,
}

impl<S: ?Sized> Default for Registry<S> {
    fn default() -> Self {
        Self { handlers: Vec::new() }
    }
}

impl<S: ?Sized> Registry<S> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, handler: impl Handler<S> + 'static) {
        self.handlers.push(Box::new(handler));
    }

    /// The handler that claims `args`: the handler of the first argument, in command-line order,
    /// that names a registered switch (the earlier registered one if two share that name), else,
    /// when no argument is a switch, the folders handler for one or two existing folders, or the
    /// bare-paths handler. `Ok(None)` for no arguments, and
    /// `Err(argument)` for an argument nobody claims.
    pub fn claim<'a>(&self, args: &'a [String]) -> Result<Option<&dyn Handler<S>>, &'a str> {
        if args.is_empty() {
            return Ok(None);
        }
        let switch = |name: &str| self.handlers.iter().find(|h| matches!(h.pattern(), Pattern::Switch(n) if n == name));
        if let Some(handler) = args.iter().filter_map(|arg| switch_name(arg)).find_map(switch) {
            return Ok(Some(handler.as_ref()));
        }
        if let Some(unknown) = args.iter().find(|arg| switch_name(arg).is_some()) {
            return Err(unknown);
        }
        let pattern = |pattern: Pattern| self.handlers.iter().find(move |h| h.pattern() == pattern);
        let folders = args.len() <= 2 && args.iter().all(|arg| Path::new(arg).is_dir());
        match pattern(Pattern::Folders).filter(|_| folders).or_else(|| pattern(Pattern::BarePaths)) {
            Some(handler) => Ok(Some(handler.as_ref())),
            None => Err(args.first().map_or("", String::as_str)),
        }
    }

    /// The usage message: one line per handler.
    // ponytail: English only; it goes through the translation lookup once E01-S1-T13 adds it.
    pub fn usage(&self) -> String {
        let mut text = String::from("Usage: otd [arguments]\n");
        for handler in &self.handlers {
            text.push_str("  ");
            text.push_str(handler.usage());
            text.push('\n');
        }
        text
    }

    /// In the instance: give a request to the handler that claimed it. A launch with no arguments
    /// opens nothing. `Err` names a handler this build doesn't have.
    pub fn open(&self, incoming: Incoming, shell: &mut S) -> Result<(), String> {
        let Incoming { request, done } = incoming;
        let Some(id) = request.handler.clone() else { return Ok(()) };
        match self.handlers.iter().find(|h| h.id() == id) {
            Some(handler) => {
                handler.open(request, done, shell);
                Ok(())
            }
            None => Err(format!("no handler {id:?} in this build")),
        }
    }
}

/// The name of a switch argument, without its prefix and value; `None` for a bare argument.
fn switch_name(arg: &str) -> Option<&str> {
    switch(arg).map(|(name, _)| name)
}

/// A switch argument's name and value, without its prefix: `-qc=size` (and on Windows `/qc=size`)
/// is `("qc", Some("size"))`. `None` for a bare argument.
pub fn switch(arg: &str) -> Option<(&str, Option<&str>)> {
    let rest = arg.strip_prefix('-').or_else(|| if cfg!(windows) { arg.strip_prefix('/') } else { None })?;
    let (name, value) = rest.split_once('=').map_or((rest, None), |(name, value)| (name, Some(value)));
    (!name.is_empty()).then_some((name, value))
}

/// Where the launch machinery looks and what it starts.
pub struct Config {
    /// The app's version. Instances of the same major version share one running instance.
    pub app_version: String,
    /// The runtime folder; `None` uses the per-user one.
    pub runtime: Option<RuntimeDir>,
    /// Starts a detached instance with no arguments, for a forward-and-wait launch that finds none.
    pub spawn_instance: Box<dyn Fn() -> io::Result<()>>,
}

/// What the process does after [`launch`].
pub enum Launch {
    /// Exit with `code`, after printing `message` to standard error.
    Exit { code: i32, message: Option<String> },
    /// This process is an instance: open the main window and act on what [`Instance`] receives.
    Instance(Instance),
}

impl Launch {
    fn exit(code: i32, message: impl Into<String>) -> Self {
        Self::Exit { code, message: Some(message.into()) }
    }
}

/// Run one launch: dispatch `args`, then hand them off, run them here or become the instance.
pub fn launch<S: ?Sized>(registry: &Registry<S>, args: &[String], requested: Requested, config: &Config) -> Launch {
    let handler = match registry.claim(args) {
        Ok(handler) => handler,
        Err(arg) => return Launch::exit(EXIT_UNCLAIMED, format!("otd: no feature handles the argument {arg:?}.\n{}", registry.usage())),
    };
    if let Some(Err(message)) = handler.map(|h| h.check(args)) {
        return Launch::exit(EXIT_UNCLAIMED, format!("otd: {message}\n{}", registry.usage()));
    }
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => return Launch::exit(EXIT_FAILED, format!("otd: can't read the working folder: {err}")),
    };
    let mut request = Request { handler: handler.map(|h| h.id().to_owned()), args: args.to_vec(), cwd, attachment: None };
    let mut mode = match (handler.map(|h| h.mode()), requested) {
        // A launch with no arguments has no view to wait for.
        (None, _) => Mode::ForwardAndExit,
        (Some(Mode::RunHere), _) => Mode::RunHere,
        (Some(_), Requested::Wait) => Mode::ForwardAndWait,
        (Some(mode), _) => mode,
    };
    if let Some(handler) = handler {
        if mode == Mode::RunHere {
            match handler.run_here(&request) {
                RunHere::Exit(code) => return Launch::Exit { code, message: None },
                RunHere::HandOff => mode = Mode::ForwardAndWait,
            }
        }
        match handler.attach(args) {
            Ok(attachment) => request.attachment = attachment,
            Err(err) => return Launch::exit(EXIT_FAILED, format!("otd: can't read what {} attaches: {err}", handler.id())),
        }
    }
    if requested == Requested::NewInstance {
        return Launch::Instance(Instance::solo(request));
    }
    match hand_off(request, mode == Mode::ForwardAndWait, config) {
        Ok(launch) => launch,
        Err(err) => Launch::exit(EXIT_FAILED, format!("otd: can't reach the running instance: {err}")),
    }
}

enum Found {
    /// This process holds the primary lock and becomes the instance.
    Primary(File),
    Running(Box<Client>),
}

fn hand_off(request: Request, wait: bool, config: &Config) -> Result<Launch, String> {
    let runtime = match &config.runtime {
        Some(dir) => dir.clone(),
        None => RuntimeDir::resolve().map_err(text)?,
    };
    let major = major(&config.app_version);
    match find(&runtime, major, !wait, &config.spawn_instance)? {
        Found::Primary(lead) => Ok(Launch::Instance(Instance::primary(lead, runtime, &config.app_version, request)?)),
        Found::Running(client) => forward(*client, &request, wait, &runtime),
    }
}

/// Become the instance when `may_lead` and none runs, or connect to the running one. A launch
/// that may not lead starts an instance once when none runs.
fn find(runtime: &RuntimeDir, major: &str, may_lead: bool, spawn: &dyn Fn() -> io::Result<()>) -> Result<Found, String> {
    let lock_path = runtime.path().join(format!("{APP}-{major}.primary"));
    let deadline = Instant::now() + FIND_DEADLINE;
    let mut spawned = false;
    loop {
        if may_lead && let Some(lead) = try_lead(&lock_path)? {
            return Ok(Found::Primary(lead));
        }
        for record in records(runtime, major) {
            if let Ok(client) = Client::connect(&record) {
                return Ok(Found::Running(Box::new(client)));
            }
        }
        if !may_lead && !spawned && try_lead(&lock_path)?.is_some() {
            // Nobody leads; the lock is released again at the end of this statement.
            spawn().map_err(|err| format!("can't start an instance: {err}"))?;
            spawned = true;
        }
        if Instant::now() >= deadline {
            return Err("no running instance answered".into());
        }
        std::thread::sleep(FIND_PAUSE);
    }
}

/// The instance records of this major version. They're read, not `Discovery::scan`ned: a scan
/// that runs while the instance publishes its record can delete that record (OverTheDiff issue
/// #13). The primary lock already says whether an instance is alive, and a stale record fails to
/// connect.
fn records(runtime: &RuntimeDir, major: &str) -> Vec<InstanceRecord> {
    let Ok(entries) = std::fs::read_dir(runtime.path()) else { return Vec::new() };
    entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_name()?.to_str()?;
            if !(name.starts_with(&format!("{APP}-")) && name.ends_with(".json")) {
                return None;
            }
            let record: InstanceRecord = serde_json::from_slice(&std::fs::read(&path).ok()?).ok()?;
            (record.validate().is_ok() && record.app == APP && self::major(&record.app_version) == major).then_some(record)
        })
        .collect()
}

fn try_lead(path: &Path) -> Result<Option<File>, String> {
    let file = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path).map_err(text)?;
    match file.try_lock() {
        Ok(()) => Ok(Some(file)),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(err)) => Err(err.to_string()),
    }
}

fn major(version: &str) -> &str {
    version.split('.').next().unwrap_or(version)
}

fn forward(mut client: Client, request: &Request, wait: bool, runtime: &RuntimeDir) -> Result<Launch, String> {
    if wait {
        // Before the request, so the close can't come first.
        client.subscribe(&[CLOSED]).map_err(text)?;
    }
    let cwd = request.cwd.to_str().ok_or("the working folder is not valid Unicode")?;
    let attachment = match &request.attachment {
        Some(bytes) => {
            let name = format!("{ATTACHMENT_PREFIX}{}", craft_ipc::generate_token().map_err(text)?);
            craft_ipc::cache::write_private(runtime.path(), &name, bytes).map_err(text)?;
            Some(name)
        }
        None => None,
    };
    let params = json!({
        "handler": request.handler,
        "args": request.args,
        "cwd": cwd,
        "attachment": attachment,
        "wait": wait,
    });
    let reply = client.call(OPEN, params);
    if let (Err(_), Some(name)) = (&reply, &attachment) {
        let _ = std::fs::remove_file(runtime.path().join(name));
    }
    let reply = reply.map_err(text)?;
    if !wait {
        return Ok(Launch::Exit { code: 0, message: None });
    }
    let ticket = reply.get("ticket").and_then(Value::as_u64).ok_or("the instance gave no ticket to wait on")?;
    loop {
        match client.next_event(Some(Duration::from_secs(3600))) {
            Ok(Some(event)) if event.event == CLOSED && event.data.get("ticket").and_then(Value::as_u64) == Some(ticket) => {
                let code = event.data.get("code").and_then(Value::as_i64).and_then(|c| i32::try_from(c).ok());
                return Ok(Launch::Exit { code: code.unwrap_or(EXIT_FAILED), message: None });
            }
            Ok(_) => {}
            Err(err) => return Ok(Launch::exit(EXIT_FAILED, format!("otd: the instance closed before the view did: {}", text(err)))),
        }
    }
}

/// One request that reached the instance.
pub struct Incoming {
    pub request: Request,
    pub done: Completion,
}

/// Reports a waiting launcher's exit code when the request's view closes. For a request nobody
/// waits for, it does nothing. Dropped unfinished, it reports [`EXIT_FAILED`].
pub struct Completion {
    waiter: Option<(u64, Weak<Shared>)>,
}

impl Completion {
    pub fn finish(mut self, code: i32) {
        self.report(code);
    }

    fn report(&mut self, code: i32) {
        let Some((ticket, shared)) = self.waiter.take() else { return };
        if let Some(shared) = shared.upgrade() {
            shared.close(ticket, code);
        }
    }
}

impl Drop for Completion {
    fn drop(&mut self) {
        self.report(EXIT_FAILED);
    }
}

/// State the instance shares with its connection threads.
struct Shared {
    tx: Sender<Incoming>,
    waker: Mutex<Option<Box<dyn Fn() + Send>>>,
    server: Mutex<Option<Server>>,
    /// When a view's close was last reported to a waiting launcher.
    last_close: Mutex<Option<Instant>>,
    /// Tickets whose launchers still wait.
    waiting: Mutex<Vec<u64>>,
    next_ticket: AtomicU64,
    runtime: Option<RuntimeDir>,
}

impl Shared {
    /// Tells the launcher waiting on `ticket` that its view closed with `code`, once.
    fn close(&self, ticket: u64, code: i32) {
        let was_waiting = {
            let mut waiting = lock(&self.waiting);
            let before = waiting.len();
            waiting.retain(|&t| t != ticket);
            waiting.len() < before
        };
        if was_waiting && let Some(server) = lock(&self.server).as_ref() {
            let _ = server.emit(CLOSED, json!({"ticket": ticket, "code": code}));
            *lock(&self.last_close) = Some(Instant::now());
        }
    }
}

/// This process as an instance: the requests it acts on, its own launch's first.
pub struct Instance {
    shared: Arc<Shared>,
    incoming: Receiver<Incoming>,
    /// The primary lock; `None` for a new instance of its own.
    _lead: Option<File>,
}

impl Instance {
    fn new(runtime: Option<RuntimeDir>, lead: Option<File>, first: Request) -> Self {
        let (tx, incoming) = mpsc::channel();
        let shared = Arc::new(Shared {
            tx,
            waker: Mutex::new(None),
            server: Mutex::new(None),
            last_close: Mutex::new(None),
            waiting: Mutex::new(Vec::new()),
            next_ticket: AtomicU64::new(1),
            runtime,
        });
        let _ = shared.tx.send(Incoming { request: first, done: Completion { waiter: None } });
        Self { shared, incoming, _lead: lead }
    }

    /// An instance of its own, which later launches don't reach.
    fn solo(first: Request) -> Self {
        Self::new(None, None, first)
    }

    fn primary(lead: File, runtime: RuntimeDir, app_version: &str, first: Request) -> Result<Self, String> {
        let instance = Self::new(Some(runtime.clone()), Some(lead), first);
        let server = Server::start(ServerConfig {
            app: APP.into(),
            app_version: app_version.into(),
            instance: None,
            runtime: Some(runtime),
            tcp_port: None,
            sink: Arc::new(Sink(Arc::downgrade(&instance.shared))),
            activate: Arc::new(NoActivate),
            frames: Arc::new(NoFrames),
        })
        .map_err(text)?;
        *lock(&instance.shared.server) = Some(server);
        Ok(instance)
    }

    /// Called whenever a request arrives, from another thread, so an idle window wakes up.
    pub fn set_waker(&self, wake: impl Fn() + Send + 'static) {
        *lock(&self.shared.waker) = Some(Box::new(wake));
    }

    /// The next request, if one is waiting.
    pub fn try_next(&self) -> Option<Incoming> {
        self.incoming.try_recv().ok()
    }

    /// The next request, waiting up to `timeout` for one.
    pub fn next_timeout(&self, timeout: Duration) -> Option<Incoming> {
        self.incoming.recv_timeout(timeout).ok()
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // Quitting closes every view, so every launcher still waiting gets its answer and hangs up
        // before the server stops: a connection still open can hold up the shutdown.
        let waiting = lock(&self.shared.waiting).clone();
        for ticket in waiting {
            self.shared.close(ticket, EXIT_FAILED);
        }
        // Taken out first: a completion dropped on a connection thread locks `server` too.
        let server = lock(&self.shared.server).take();
        if let Some(server) = server {
            let last_close = *lock(&self.shared.last_close);
            if let Some(at) = last_close {
                std::thread::sleep(CLOSE_GRACE.saturating_sub(at.elapsed()));
            }
            let _ = server.shutdown();
        }
    }
}

/// Receives `otd.open` on a connection thread and queues it for the instance.
struct Sink(Weak<Shared>);

impl ControlSink for Sink {
    fn submit(&self, method: String, params: Value) -> Option<Receiver<Value>> {
        let shared = self.0.upgrade()?;
        let (reply, rx) = mpsc::channel();
        let answer = if method == OPEN {
            match receive(&shared, &params) {
                Ok(incoming) => {
                    let ticket = incoming.done.waiter.as_ref().map(|(ticket, _)| *ticket);
                    shared.tx.send(incoming).ok()?;
                    if let Some(wake) = lock(&shared.waker).as_ref() {
                        wake();
                    }
                    json!({ "ticket": ticket })
                }
                Err(err) => json!({ "ok": false, "error": err }),
            }
        } else {
            json!({ "ok": false, "error": format!("otd has no method {method:?}") })
        };
        let _ = reply.send(answer);
        Some(rx)
    }
}

fn receive(shared: &Arc<Shared>, params: &Value) -> Result<Incoming, String> {
    let wire = decode(params)?;
    let mut request = wire.request;
    if let Some(name) = wire.attachment {
        let dir = shared.runtime.as_ref().ok_or("no runtime folder")?.path();
        let bytes = craft_ipc::cache::read_private(dir, &name).map_err(text);
        let _ = std::fs::remove_file(dir.join(&name));
        request.attachment = Some(bytes?);
    }
    let waiter = wire.wait.then(|| {
        let ticket = shared.next_ticket.fetch_add(1, Ordering::Relaxed);
        lock(&shared.waiting).push(ticket);
        (ticket, Arc::downgrade(shared))
    });
    Ok(Incoming { request, done: Completion { waiter } })
}

/// An `otd.open` request as it arrives: the attachment is still a file name in the runtime folder.
#[derive(Debug)]
struct Wire {
    request: Request,
    attachment: Option<String>,
    wait: bool,
}

fn decode(params: &Value) -> Result<Wire, String> {
    let obj = params.as_object().ok_or("params must be an object")?;
    let handler = optional_str(obj, "handler")?.map(str::to_owned);
    let args = obj
        .get("args")
        .and_then(Value::as_array)
        .ok_or("args must be an array")?
        .iter()
        .map(|arg| arg.as_str().map(str::to_owned).ok_or("args must be strings"))
        .collect::<Result<Vec<_>, _>>()?;
    let cwd = PathBuf::from(obj.get("cwd").and_then(Value::as_str).ok_or("cwd must be a string")?);
    let attachment = optional_str(obj, "attachment")?.map(str::to_owned);
    if let Some(name) = &attachment {
        let token = name.strip_prefix(ATTACHMENT_PREFIX).unwrap_or_default();
        if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            return Err("attachment is not an attachment name".into());
        }
    }
    let wait = obj.get("wait").and_then(Value::as_bool).ok_or("wait must be a boolean")?;
    Ok(Wire { request: Request { handler, args, cwd, attachment: None }, attachment, wait })
}

fn optional_str<'a>(obj: &'a Map<String, Value>, key: &str) -> Result<Option<&'a str>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text)),
        Some(_) => Err(format!("{key} must be a string or null")),
    }
}

/// otd takes no focus requests from other Craft apps yet, and says so.
struct NoActivate;

impl Activate for NoActivate {
    fn activate(&self, _token: Option<String>, _from_pid: Option<u32>, _open: Option<Value>) -> Activation {
        Activation::No
    }
}

/// otd serves no frames.
struct NoFrames;

impl Frames for NoFrames {
    fn open(&self, _params: &Map<String, Value>) -> Result<FrameOpen, LinkError> {
        Err(LinkError::Unknown("otd serves no frames".into()))
    }

    fn render(&self, _ticket: &str, _header: &Value) -> Result<(Value, Vec<u8>), LinkError> {
        Err(LinkError::Unknown("otd serves no frames".into()))
    }

    fn close_ticket(&self, _ticket: &str) {}
}

fn text(err: impl std::fmt::Display) -> String {
    err.to_string()
}

fn lock<T: ?Sized>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use craft_testkit::hostile::{HammerConfig, hammer};

    use super::decode;

    #[test]
    fn the_request_decoder_never_panics_or_hangs_on_damaged_requests() {
        let seed = br#"{"handler":"files","args":["a.txt","-x=1"],"cwd":"/w","attachment":"otd-attach-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef","wait":true}"#;
        let value = serde_json::from_slice(seed).unwrap_or_default();
        assert!(decode(&value).is_ok_and(|wire| wire.wait && wire.attachment.is_some()), "the seed decodes");

        hammer(&HammerConfig::new("otd.open params", 2_000), seed, |data| {
            if let Ok(value) = serde_json::from_slice(data) {
                let _ = decode(&value);
            }
        });
    }
}
