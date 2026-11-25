//! The `otd` executable. `otd-launch` decides first whether this process hands its arguments to the
//! running instance, runs them here or becomes the instance; only an instance opens the window.

mod handlers;

use std::process::{Command, Stdio};

use eframe::egui;
use otd_launch::{Config, EXIT_FAILED, EXIT_UNCLAIMED, Instance, Launch, Registry, Requested};

/// What an argument handler acts on in the instance.
// ponytail: empty until E01-S1-T3 builds the tab strip, Home and status bar.
pub struct Shell;

fn main() {
    let args = match std::env::args_os().skip(1).map(|arg| arg.into_string()).collect::<Result<Vec<_>, _>>() {
        Ok(args) => args,
        Err(arg) => {
            eprintln!("otd: the argument {arg:?} is not valid Unicode.");
            std::process::exit(EXIT_UNCLAIMED);
        }
    };
    let registry = handlers::registry();
    let config = Config { app_version: env!("CARGO_PKG_VERSION").into(), runtime: None, spawn_instance: Box::new(spawn_instance) };
    match otd_launch::launch(&registry, &args, Requested::Default, &config) {
        Launch::Exit { code, message } => {
            if let Some(message) = message {
                eprintln!("{message}");
            }
            std::process::exit(code);
        }
        Launch::Instance(instance) => {
            if let Err(err) = run_window(registry, instance) {
                eprintln!("otd: can't open the window: {err}");
                std::process::exit(EXIT_FAILED);
            }
        }
    }
}

/// Starts this executable with no arguments, detached, so it becomes the instance. Its standard
/// streams are closed so a tool that launched us isn't held open by them.
fn spawn_instance() -> std::io::Result<()> {
    let mut command = Command::new(std::env::current_exe()?);
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    #[cfg(windows)]
    {
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        std::os::windows::process::CommandExt::creation_flags(&mut command, DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    command.spawn().map(drop)
}

fn run_window(registry: Registry<Shell>, instance: Instance) -> eframe::Result {
    // ponytail: empty window only; the tab strip, Home, menus and status bar are E01-S1's.
    eframe::run_native(
        "OverTheDiff",
        eframe::NativeOptions::default(),
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();
            instance.set_waker(move || ctx.request_repaint());
            Ok(Box::new(App { registry, instance, shell: Shell }))
        }),
    )
}

struct App {
    registry: Registry<Shell>,
    instance: Instance,
    shell: Shell,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Some(incoming) = self.instance.try_next() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
            if let Err(err) = self.registry.open(incoming, &mut self.shell) {
                // ponytail: standard error until E01-S1-T3's status bar can say it in the window.
                eprintln!("otd: {err}");
            }
        }
    }
}
