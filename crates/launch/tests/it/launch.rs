//! Launches against a running instance, each test in a runtime folder of its own. Launches run in
//! this process, on threads where they must wait, so a launch that hands off and one that becomes
//! the instance are told apart by what `launch` returns rather than by watching processes.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use craft_ipc::RuntimeDir;
use otd_launch::{
    Completion, Config, EXIT_FAILED, EXIT_UNCLAIMED, Handler, Incoming, Instance, Launch, Mode, Pattern, Registry, Request, Requested, RunHere,
};

/// What the `stdin` test handler attaches; not valid UTF-8, as standard input needn't be.
const ATTACHED: &[u8] = b"left\xffright\n";

/// The views a test instance opened, each with the completion it reports through when it closes.
#[derive(Default)]
struct Views(Vec<(Request, Completion)>);

struct TestHandler {
    id: &'static str,
    pattern: Pattern,
    mode: Mode,
    run_here: fn(&Request) -> RunHere,
    attach: Option<&'static [u8]>,
}

impl Handler<Views> for TestHandler {
    fn id(&self) -> &'static str {
        self.id
    }

    fn pattern(&self) -> Pattern {
        self.pattern
    }

    fn mode(&self) -> Mode {
        self.mode
    }

    fn usage(&self) -> &'static str {
        self.id
    }

    fn attach(&self, _args: &[String]) -> io::Result<Option<Vec<u8>>> {
        Ok(self.attach.map(<[u8]>::to_vec))
    }

    fn run_here(&self, request: &Request) -> RunHere {
        (self.run_here)(request)
    }

    fn open(&self, request: Request, done: Completion, views: &mut Views) {
        views.0.push((request, done));
    }
}

fn handler(id: &'static str, pattern: Pattern, mode: Mode) -> TestHandler {
    TestHandler { id, pattern, mode, run_here: |_| RunHere::HandOff, attach: None }
}

fn registry() -> Registry<Views> {
    let mut registry = Registry::new();
    registry.register(handler("files", Pattern::BarePaths, Mode::ForwardAndExit));
    registry.register(handler("wait-view", Pattern::Switch("wait-view"), Mode::ForwardAndWait));
    // `-here=N` exits with N.
    registry.register(TestHandler {
        run_here: |request| RunHere::Exit(request.args.iter().find_map(|arg| arg.strip_prefix("-here=")?.parse().ok()).unwrap_or(0)),
        ..handler("here", Pattern::Switch("here"), Mode::RunHere)
    });
    registry.register(handler("review", Pattern::Switch("review"), Mode::RunHere));
    registry.register(TestHandler { attach: Some(ATTACHED), ..handler("stdin", Pattern::Switch("stdin"), Mode::ForwardAndExit) });
    registry
}

type Spawn = Box<dyn Fn() -> io::Result<()>>;

fn no_spawn() -> Spawn {
    Box::new(|| Err(io::Error::other("this test starts no instance")))
}

fn launch_at(dir: &Path, args: &[&str], requested: Requested, spawn_instance: Spawn) -> Launch {
    let args: Vec<String> = args.iter().map(|&arg| arg.to_owned()).collect();
    let config = Config { app_version: "1.2.3".into(), runtime: Some(RuntimeDir::at(dir).expect("runtime folder")), spawn_instance };
    otd_launch::launch(&registry(), &args, requested, &config)
}

/// A runtime folder of its own, short enough for a socket path. Tests that hold one run one at a
/// time: an instance's Windows pipe is named after the app and the process, not the folder, so two
/// instances in this process would claim the same pipe.
struct TestDir {
    path: PathBuf,
    _one_at_a_time: MutexGuard<'static, ()>,
}

impl TestDir {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
        let serial = ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner);
        let path = std::env::temp_dir().join(format!("otd{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        Self { path, _one_at_a_time: serial }
    }

    fn launch(&self, args: &[&str], requested: Requested) -> Launch {
        launch_at(&self.path, args, requested, no_spawn())
    }

    fn launch_in_thread(&self, args: &'static [&'static str], requested: Requested) -> JoinHandle<Launch> {
        let dir = self.path.clone();
        thread::spawn(move || launch_at(&dir, args, requested, no_spawn()))
    }

    /// The running instance, from a launch with no arguments, with its own empty request taken.
    fn start(&self) -> Instance {
        let instance = instance(self.launch(&[], Requested::Default));
        assert!(instance.try_next().is_some_and(|first| first.request.handler.is_none()), "the instance's own launch comes first");
        instance
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn code(launch: Launch) -> i32 {
    match launch {
        Launch::Exit { code, .. } => code,
        Launch::Instance(_) => panic!("the launch became an instance"),
    }
}

fn instance(launch: Launch) -> Instance {
    match launch {
        Launch::Instance(instance) => instance,
        Launch::Exit { code, message } => panic!("the launch exited with {code}: {message:?}"),
    }
}

/// The next request that reaches `instance`. The deadline only stops a hang; it isn't asserted.
fn next(instance: &Instance) -> Incoming {
    instance.next_timeout(Duration::from_secs(30)).expect("a request reaches the instance")
}

/// Opens the next request's view in `instance`.
fn open_next(instance: &Instance) -> (Request, Completion) {
    let mut views = Views::default();
    registry().open(next(instance), &mut views).expect("a handler for the request");
    views.0.pop().expect("the handler opened a view")
}

#[test]
fn two_launches_at_once_leave_one_instance_that_handled_both_argument_sets() {
    let dir = TestDir::new();
    let barrier = Arc::new(Barrier::new(2));
    let threads: Vec<_> = ["a.txt", "b.txt"]
        .into_iter()
        .map(|arg| {
            let (dir, barrier) = (dir.path.clone(), Arc::clone(&barrier));
            thread::spawn(move || {
                barrier.wait();
                launch_at(&dir, &[arg], Requested::Default, no_spawn())
            })
        })
        .collect();
    // Both launches return before the instance has acted on anything: the one that hands off
    // doesn't wait for it.
    let (mut instances, mut codes) = (Vec::new(), Vec::new());
    for thread in threads {
        match thread.join().expect("launch thread") {
            Launch::Instance(instance) => instances.push(instance),
            Launch::Exit { code, .. } => codes.push(code),
        }
    }
    assert_eq!((instances.len(), codes), (1, vec![0]), "one launch becomes the instance, and the other hands off and exits with 0");

    let mut handled: Vec<Vec<String>> = (0..2).map(|_| next(&instances[0]).request.args).collect();
    handled.sort();
    assert_eq!(handled, [["a.txt"], ["b.txt"]], "the instance has both argument sets");
}

#[test]
fn an_argument_nobody_claims_gives_the_usage_message_and_exit_code_100() {
    let dir = TestDir::new();
    let Launch::Exit { code, message } = dir.launch(&["a.txt", "-nobody"], Requested::Default) else { panic!("the launch became an instance") };
    assert_eq!(code, EXIT_UNCLAIMED);
    assert!(message.is_some_and(|message| message.contains(&registry().usage())), "the message holds the usage text");
}

#[test]
fn a_switch_handler_claims_the_whole_command_line_before_the_bare_paths_handler() {
    let args = ["a.txt", "-stdin", "b.txt"].map(String::from);
    let registry = registry();
    let claimed = registry.claim(&args).ok().flatten().map(|handler| handler.id());
    assert_eq!(claimed, Some("stdin"));
}

#[test]
fn a_run_here_handler_exits_with_its_own_code_and_sends_nothing_to_the_running_instance() {
    let dir = TestDir::new();
    let instance = dir.start();
    assert_eq!(code(dir.launch(&["-here=7"], Requested::Default)), 7);
    // A forwarded request is queued before the launch returns, so it would be here by now.
    assert!(instance.try_next().is_none(), "nothing reached the instance");
}

#[test]
fn a_run_here_handler_with_no_instance_running_leaves_the_instance_role_free() {
    let dir = TestDir::new();
    assert_eq!(code(dir.launch(&["-here=3"], Requested::Default)), 3);
    assert!(matches!(dir.launch(&["a.txt"], Requested::Default), Launch::Instance(_)), "the next launch becomes the instance");
}

/// Launches `args` against a running instance and closes the view it opens with a code the
/// launcher can only learn from that close.
fn exit_code_after_the_view_closes(args: &'static [&'static str], requested: Requested) -> (String, i32) {
    let dir = TestDir::new();
    let instance = dir.start();
    let waiting = dir.launch_in_thread(args, requested);
    let (request, done) = open_next(&instance);
    done.finish(42);
    (request.handler.unwrap_or_default(), code(waiting.join().expect("launch thread")))
}

#[test]
fn a_forward_and_wait_launcher_exits_after_its_view_closes_with_the_code_the_handler_reported() {
    assert_eq!(exit_code_after_the_view_closes(&["-wait-view"], Requested::Default), ("wait-view".into(), 42));
}

#[test]
fn a_launch_requesting_wait_waits_for_the_view_of_a_forward_and_exit_handler() {
    assert_eq!(exit_code_after_the_view_closes(&["a.txt"], Requested::Wait), ("files".into(), 42));
}

#[test]
fn a_handler_that_hands_off_from_the_launching_process_opens_its_view_in_the_instance_and_waits() {
    assert_eq!(exit_code_after_the_view_closes(&["-review"], Requested::Default), ("review".into(), 42));
}

#[test]
fn a_waiting_launcher_exits_with_a_failure_when_the_instance_quits_first() {
    let dir = TestDir::new();
    let instance = dir.start();
    let waiting = dir.launch_in_thread(&["-wait-view"], Requested::Default);
    let _open = next(&instance);
    drop(instance);
    assert_eq!(code(waiting.join().expect("launch thread")), EXIT_FAILED);
}

#[test]
fn a_forward_and_wait_launch_with_no_instance_starts_one_and_waits_on_it() {
    let dir = TestDir::new();
    let (path, started) = (dir.path.clone(), Arc::new(Mutex::new(None)));
    let started_here = Arc::clone(&started);
    let spawn: Spawn = Box::new(move || {
        let path = path.clone();
        *started_here.lock().unwrap_or_else(PoisonError::into_inner) = Some(thread::spawn(move || {
            let instance = instance(launch_at(&path, &[], Requested::Default, no_spawn()));
            let _own = next(&instance);
            let (_request, done) = open_next(&instance);
            done.finish(6);
        }));
        Ok(())
    });
    assert_eq!(code(launch_at(&dir.path, &["-wait-view"], Requested::Default, spawn)), 6);
    let instance_thread = started.lock().unwrap_or_else(PoisonError::into_inner).take();
    instance_thread.expect("the launch started an instance").join().expect("instance thread");
}

#[test]
fn a_launch_requesting_a_new_instance_becomes_one_and_later_launches_still_reach_the_first() {
    let dir = TestDir::new();
    let first = dir.start();
    let solo = instance(dir.launch(&["a.txt"], Requested::NewInstance));
    assert_eq!(solo.try_next().map(|incoming| incoming.request.args), Some(vec!["a.txt".to_owned()]), "the new instance acts on its own arguments");
    assert!(first.try_next().is_none(), "nothing reached the first instance");

    assert_eq!(code(dir.launch(&["b.txt"], Requested::Default)), 0);
    assert_eq!(next(&first).request.args, ["b.txt"], "a later launch hands off to the first instance");
}

#[test]
fn content_a_handler_attaches_in_the_launching_process_reaches_the_instance() {
    let dir = TestDir::new();
    let instance = dir.start();
    assert_eq!(code(dir.launch(&["-stdin"], Requested::Default)), 0);
    assert_eq!(next(&instance).request.attachment.as_deref(), Some(ATTACHED));
}
