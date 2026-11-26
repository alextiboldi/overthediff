//! The `otd` executable. `otd-launch` decides first whether this process hands its arguments to the
//! running instance, runs them here or becomes the instance; only an instance opens the window.
// A release build on Windows is a GUI binary, so a Start menu launch opens no console window. Its
// standard streams then reach no console; E11-S1's waiting launchers serve consoles.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod auto_save;
mod colour_roles;
mod commands;
mod handlers;
mod open_session;
mod options;
mod quick_compare;
mod session_settings;
mod session_types;
mod settings;

use std::process::{Command, Stdio};

use eframe::egui;
use otd_launch::{Config, EXIT_FAILED, EXIT_UNCLAIMED, Instance, Launch, Registry, Requested};

/// What an argument handler acts on in the instance: the main window's tabs.
pub use otd_ui::Shell;

/// The app's reverse-DNS id: the Linux desktop entry's and icon's name, and the window's app id.
/// It ends in the major version, so two major versions install and run side by side (E01-S2-T5);
/// `APP_ID` in packaging/env.sh is the same.
const APP_ID: &str = concat!("io.github.alextiboldi.overthediff", env!("CARGO_PKG_VERSION_MAJOR"));

fn main() {
    let args = match std::env::args_os().skip(1).map(|arg| arg.into_string()).collect::<Result<Vec<_>, _>>() {
        Ok(args) => args,
        Err(arg) => {
            eprintln!("otd: the argument {arg:?} is not valid Unicode.");
            std::process::exit(EXIT_UNCLAIMED);
        }
    };
    let args = if cfg!(target_os = "macos") { drop_process_serial_number(args) } else { args };
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

/// macOS passes a process serial number (`-psn_…`) when Finder opens the app, on older versions and
/// on the first open of a downloaded copy: it is dropped, unless it comes after `--`.
fn drop_process_serial_number(args: Vec<String>) -> Vec<String> {
    let mut options = true;
    args.into_iter()
        .filter(|arg| {
            options &= arg != "--";
            !(options && arg.starts_with("-psn_"))
        })
        .collect()
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
    // On Wayland the window's app id must name the Linux desktop entry
    // (packaging/linux/<APP_ID>.desktop), so the launcher matches the window to its entry and icon.
    // egui-winit also sets it as the X11 window class, which the entry's StartupWMClass names.
    let options = eframe::NativeOptions { viewport: egui::ViewportBuilder::default().with_app_id(APP_ID), ..Default::default() };
    eframe::run_native(
        "OverTheDiff",
        options,
        Box::new(move |cc| {
            otd_ui::colour::install(&cc.egui_ctx, colour_roles::colour_roles()?);
            // ponytail: always follows the OS until the Appearance option (E05-S1-T2) chooses
            // between this and a fixed palette (`set_palette`).
            otd_ui::colour::follow_system(&cc.egui_ctx);
            // On the UI thread, which draws all the text.
            let catalogs = otd_core::i18n::embedded();
            otd_core::i18n::set_language(catalogs, catalogs.system_lang());
            // After the language, so its notice is translated.
            let mut settings = settings::Settings::open(otd_settings::Location::Auto);
            let path_edits = otd_ui::PathEdits::open(&cc.egui_ctx, &mut settings.store);
            let options = options::options();
            let saved = otd_settings::options::read(&mut settings.store, &options);
            let ctx = cc.egui_ctx.clone();
            instance.set_waker(move || ctx.request_repaint());
            let mut shell = Shell::new(session_types::session_types())
                .with_settings(session_settings::session_settings())?
                .with_options(options, saved)?
                .with_commands(commands::commands());
            shell.load_session_defaults(&mut settings.store);
            // On macOS the menus go in the native menu bar, the application menu included; when
            // AppKit refuses them, the window keeps its own bar.
            #[cfg(target_os = "macos")]
            let native_menu = craft_menu::mac::install(&cc.egui_ctx, &shell.menu_layout(cc.egui_ctx.os()).bar);
            #[cfg(not(target_os = "macos"))]
            let native_menu = None;
            if native_menu.is_some() {
                shell.hide_in_window_menu();
            }
            Ok(Box::new(App { registry, instance, shell, native_menu, settings, path_edits }))
        }),
    )
}

struct App {
    registry: Registry<Shell>,
    instance: Instance,
    shell: Shell,
    native_menu: Option<craft_menu::NativeMenu>,
    settings: settings::Settings,
    path_edits: otd_ui::PathEdits,
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

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
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
        self.shell.sessions_ui(&ctx, &self.settings.store);
        self.path_edits.frame(&ctx, Some(&*frame), &mut self.settings.store);
        let cap = auto_save::cap(&self.shell);
        auto_save::closed(&mut self.settings.store, &mut self.shell, cap);
        if let Some(values) = self.shell.take_applied_options() {
            otd_settings::options::write(&mut self.settings.store, &values);
            otd_settings::auto_saved::trim(&mut self.settings.store, otd_settings::auto_saved::cap(values.get(otd_settings::auto_saved::MAX)));
        }
        let new_defaults = self.shell.take_session_defaults();
        for session in &new_defaults {
            otd_settings::session_defaults::write(&mut self.settings.store, session);
        }
        // Sessions opened later in this run start from them too (E06-S2-T3).
        if !new_defaults.is_empty() {
            self.shell.load_session_defaults(&mut self.settings.store);
        }
        if let Some(menu) = &mut self.native_menu {
            // Its clashes are the system items macOS gives up for the app's shortcuts (⌘W); the
            // layout has already resolved them.
            menu.sync(self.shell.menu_layout(ctx.os()));
        }
    }

    fn on_exit(&mut self) {
        let cap = auto_save::cap(&self.shell);
        auto_save::exit(&mut self.settings.store, &mut self.shell, cap);
        self.settings.exit();
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::drop_process_serial_number;

    #[test]
    fn a_finder_process_serial_number_is_dropped_before_double_dash() {
        let args = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(drop_process_serial_number(args(&["-psn_0_12345", "a.txt"])), args(&["a.txt"]));
        assert_eq!(drop_process_serial_number(args(&["--", "-psn_x.txt"])), args(&["--", "-psn_x.txt"]));
    }

    /// The launcher shows the Linux desktop entry, starts it and matches its window only when the
    /// entry is named after the app id and names this executable and an icon that ships. The
    /// entry is a template that packaging/linux/package.sh fills in for each major version.
    #[test]
    fn the_linux_desktop_entry_names_this_executable_and_a_shipped_icon() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let major = env!("CARGO_PKG_VERSION_MAJOR");
        let template = std::fs::read_to_string(root.join("packaging/linux/overthediff.desktop.in")).expect("the desktop entry");
        let entry = template.replace("@APP_ID@", super::APP_ID).replace("@MAJOR@", major);
        let key = |name: &str| entry.lines().find_map(|line| line.strip_prefix(name)?.strip_prefix('=')).unwrap_or_default();
        let bin = format!("{}-{major}", env!("CARGO_BIN_NAME"));
        assert_eq!(key("Exec").split(' ').next(), Some(bin.as_str()));
        assert_eq!(key("TryExec"), bin);
        assert_eq!(key("StartupWMClass"), super::APP_ID, "egui-winit sets the X11 window class to the app id");
        assert_eq!(key("Icon"), super::APP_ID);
        // The icons ship under the id without its major; package.sh renames them.
        let icon = super::APP_ID.strip_suffix(major).expect("the app id ends in the major version");
        let icons = root.join("assets/app-icon/hicolor");
        assert!(icons.join(format!("256x256/apps/{icon}.png")).is_file());
        assert!(icons.join(format!("scalable/apps/{icon}.svg")).is_file());
    }
}
