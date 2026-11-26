//! `otd <name>`: opens a saved session by name in its view (E06-S1-T7). The launch dispatch gives
//! this handler one argument that names no existing file or folder, so an existing path wins over a
//! session with the same name, and a name no session has gives the usage error.

use otd_core::i18n::{fmt, t};
use otd_launch::{Completion, Handler, Pattern, Request};
use otd_settings::{Location, sessions};
use otd_ui::{Session, SessionType, Shell};

const NO_SUCH_SESSION: &str = "No saved session is named \"{name}\".";
const UNKNOWN_TYPE: &str = "The saved session \"{name}\" is of a type this build doesn't have.";

/// Opens the saved session the argument names, from the settings store at `.0`.
pub struct OpenSession(pub Location);

impl OpenSession {
    /// The session saved as `name` and its type, or why it can't open: the not-found message, then
    /// any warning about a session file that was skipped, which may be the one this name was saved
    /// in; or that this build has no such session type.
    // ponytail: reads every session file to find one. Opening a store only reads `config.toml` (and
    // sets a damaged one aside, as the instance's start does); it writes nothing back, since it is
    // never ticked or flushed. Fine until there are thousands of sessions.
    fn find(&self, name: &str) -> Result<(Session, SessionType), String> {
        let (store, _notice) = otd_settings::open(self.0.clone());
        let (mut saved, warnings) = sessions::load(&store, &crate::session_settings::session_settings());
        let session = saved
            .remove(name)
            .ok_or_else(|| std::iter::once(fmt(t(NO_SUCH_SESSION), &[("name", name)])).chain(warnings).collect::<Vec<_>>().join("\n"))?;
        let session_type = crate::session_types::session_types()
            .into_iter()
            .find(|t| t.id == session.type_id)
            .ok_or_else(|| fmt(t(UNKNOWN_TYPE), &[("name", name)]))?;
        Ok((session, session_type))
    }
}

impl Handler<Shell> for OpenSession {
    fn id(&self) -> &'static str {
        "session"
    }

    fn pattern(&self) -> Pattern {
        Pattern::Name
    }

    fn usage(&self) -> &'static str {
        "<session>        Open a saved session by name"
    }

    fn check(&self, args: &[String]) -> Result<(), String> {
        self.find(args.first().map_or("", String::as_str)).map(drop)
    }

    // ponytail: `done` drops here, which tells a waiting launcher the view failed; nothing waits on
    // this handler yet (`main` launches with `Requested::Default`). A view-agnostic completion
    // comes with the first waiting launch of a session (E11).
    fn open(&self, request: Request, _done: Completion, shell: &mut Shell) {
        let name = request.args.first().map_or("", String::as_str);
        // Loaded again: the session may have changed or gone since the launcher checked it.
        let (session, session_type) = match self.find(name) {
            Ok(found) => found,
            Err(why) => return eprintln!("otd: {why}"),
        };
        let mut view = (session_type.new)();
        if let Some(slot) = view.session() {
            *slot = session;
        }
        shell.open(view);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use otd_launch::{Config, EXIT_UNCLAIMED, Launch, Registry, Requested};
    use otd_ui::View as _;
    use otd_ui_folder_compare::FolderCompare;
    use otd_ui_text_compare::TextCompareView;

    use super::*;

    fn save(dir: &Path, name: &str, type_id: &str, specs: &[&str]) -> Session {
        let (store, _) = otd_settings::open(Location::Dir(dir.to_path_buf()));
        let mut session = Session::new(type_id);
        session.specs = specs.iter().map(|s| s.to_string()).collect();
        session.description = format!("{name} description");
        session.fill_defaults(&crate::session_settings::session_settings());
        sessions::save(&store, name, &session).unwrap();
        session
    }

    /// Launches `otd <name>` as a new instance of its own (no running instance is looked for) with
    /// sessions saved in `dir`, and gives what reaches the instance to `shell`.
    fn launch(dir: &Path, name: &str, shell: &mut Shell) -> Option<i32> {
        let mut registry = Registry::new();
        registry.register(OpenSession(Location::Dir(dir.to_path_buf())));
        let config = Config { app_version: "0.0.0".into(), runtime: None, spawn_instance: Box::new(|| Ok(())) };
        match otd_launch::launch(&registry, &[name.to_owned()], Requested::NewInstance, &config) {
            Launch::Exit { code, .. } => Some(code),
            Launch::Instance(instance) => {
                let incoming = instance.try_next().expect("the launch's request");
                registry.open(incoming, shell).unwrap();
                None
            }
        }
    }

    #[test]
    fn a_saved_session_opens_in_its_own_view_with_its_specs_and_settings() {
        let dir = tempfile::tempdir().unwrap();
        let text = save(dir.path(), "Weekly text", otd_ui_text_compare::TEXT_COMPARE, &["/a.txt", "/b.txt"]);
        let folders = save(dir.path(), "Weekly folders", otd_ui_folder_compare::ID, &["/left", "/right"]);
        let mut shell = Shell::new(crate::session_types::session_types());

        assert_eq!(launch(dir.path(), "Weekly text", &mut shell), None);
        let opened = shell.view_mut::<TextCompareView>().expect("a Text Compare tab").session().cloned();
        assert_eq!(opened, Some(text));

        assert_eq!(launch(dir.path(), "Weekly folders", &mut shell), None);
        let opened = shell.view_mut::<FolderCompare>().expect("a Folder Compare tab").session().cloned();
        assert_eq!(opened, Some(folders));
    }

    #[test]
    fn an_unknown_name_gives_the_usage_error_and_opens_nothing() {
        let dir = tempfile::tempdir().unwrap();
        save(dir.path(), "Weekly", otd_ui_text_compare::TEXT_COMPARE, &[]);
        let mut shell = Shell::new(crate::session_types::session_types());
        let home = shell.view_type();
        assert_eq!(launch(dir.path(), "Monthly", &mut shell), Some(EXIT_UNCLAIMED));
        assert_eq!(launch(dir.path(), "weekly", &mut shell), Some(EXIT_UNCLAIMED), "names are exact");
        assert_eq!(shell.view_type(), home, "still on Home");
    }

    #[test]
    fn a_session_of_a_type_this_build_lacks_gives_the_usage_error_and_opens_nothing() {
        let dir = tempfile::tempdir().unwrap();
        save(dir.path(), "From a newer build", "not-a-session-type", &[]);
        let mut shell = Shell::new(crate::session_types::session_types());
        let home = shell.view_type();
        assert_eq!(launch(dir.path(), "From a newer build", &mut shell), Some(EXIT_UNCLAIMED));
        assert_eq!(shell.view_type(), home, "still on Home");
    }
}
