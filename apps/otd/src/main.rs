//! The `otd` executable. `otd-launch` decides first whether this process hands its arguments to the
//! running instance, runs them here or becomes the instance; only an instance opens the window.

mod colour_roles;
mod commands;
mod handlers;
mod session_settings;
mod session_types;
mod settings;

use std::process::{Command, Stdio};

use eframe::egui;
use otd_launch::{Config, EXIT_FAILED, EXIT_UNCLAIMED, Instance, Launch, Registry, Requested};

/// What an argument handler acts on in the instance: the main window's tabs.
pub use otd_ui::Shell;

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
    eframe::run_native(
        "OverTheDiff",
        eframe::NativeOptions::default(),
        Box::new(move |cc| {
            otd_ui::colour::install(&cc.egui_ctx, colour_roles::colour_roles()?);
            // On the UI thread, which draws all the text.
            let catalogs = otd_core::i18n::embedded();
            otd_core::i18n::set_language(catalogs, catalogs.system_lang());
            // After the language, so its notice is translated.
            let settings = settings::Settings::open(otd_settings::Location::Auto);
            let ctx = cc.egui_ctx.clone();
            instance.set_waker(move || ctx.request_repaint());
            let mut shell =
                Shell::new(session_types::session_types()).with_settings(session_settings::session_settings())?.with_commands(commands::commands());
            // On macOS the menus go in the native menu bar, the application menu included; when
            // AppKit refuses them, the window keeps its own bar.
            #[cfg(target_os = "macos")]
            let native_menu = craft_menu::mac::install(&cc.egui_ctx, &shell.menu_layout(cc.egui_ctx.os()).bar);
            #[cfg(not(target_os = "macos"))]
            let native_menu = None;
            if native_menu.is_some() {
                shell.hide_in_window_menu();
            }
            Ok(Box::new(App { registry, instance, shell, native_menu, settings }))
        }),
    )
}

struct App {
    registry: Registry<Shell>,
    instance: Instance,
    shell: Shell,
    native_menu: Option<craft_menu::NativeMenu>,
    settings: settings::Settings,
}

impl eframe::App for App {
    /// A window close (its close button, the OS, Alt+F4, or the Exit command) asks about each
    /// unsaved view first.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.shell.on_close_request(ctx);
        // Last, so it sees the `CancelClose` above. On macOS it quits through `terminate:` instead
        // of letting eframe drop the window while AppKit still uses it; elsewhere it does nothing.
        craft_quit::on_close_request(ctx);
    }

    /// The native menu's key equivalents reach egui as the key presses they were, so shortcuts
    /// run through the shell's keymap as without a native menu.
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if let Some(menu) = &mut self.native_menu {
            menu.raw_input(raw_input);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Some(incoming) = self.instance.try_next() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
            if let Err(err) = self.registry.open(incoming, &mut self.shell) {
                // ponytail: standard error only. The status bar shows the active view's fields
                // (Q25), not the shell's messages, and the settings notice (`settings.rs`) is the
                // settings store's own.
                eprintln!("otd: {err}");
            }
        }
        let ctx = ui.ctx().clone();
        if let Some(menu) = &mut self.native_menu {
            menu.set_gate(craft_menu::Gate { text_editing: ctx.text_edit_focused(), ..Default::default() });
            for command in menu.take_clicks(&ctx) {
                self.shell.run(&command.id);
            }
        }
        self.settings.frame(ui);
        self.shell.ui(ui);
        if let Some(menu) = &mut self.native_menu {
            // Its clashes are the system items macOS gives up for the app's shortcuts (⌘W); the
            // layout has already resolved them.
            menu.sync(self.shell.menu_layout(ctx.os()));
        }
    }

    fn on_exit(&mut self) {
        self.settings.exit();
    }
}
