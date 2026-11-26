//! Auto-saves recent sessions (E06-S1-T4): each view's session as its tab closes, and every open
//! one on exit, within the *Max auto-saved sessions* cap (`otd_settings::auto_saved`).

use otd_settings::{Store, auto_saved};
use otd_ui::Shell;

/// The cap the applied *Max auto-saved sessions* option sets: never the Options dialog's preview,
/// since adding truncates the list and Cancel couldn't bring the dropped sessions back.
pub fn cap(shell: &Shell) -> usize {
    auto_saved::cap(shell.applied_option(auto_saved::MAX))
}

/// Auto-saves the sessions of the tabs closed since the last call.
pub fn closed(store: &mut Store, shell: &mut Shell, cap: usize) {
    for session in shell.take_closed_sessions() {
        auto_saved::add(store, &session, cap);
    }
}

/// Auto-saves every session exiting closes: the tabs just closed, then the open ones in tab order.
pub fn exit(store: &mut Store, shell: &mut Shell, cap: usize) {
    closed(store, shell, cap);
    for session in shell.open_sessions() {
        auto_saved::add(store, &session, cap);
    }
}

#[cfg(test)]
mod tests {
    use otd_settings::Location;

    use super::*;

    /// Three Text Compare tabs on different files, closed in turn, with the cap at `cap`: the
    /// auto-saved sessions' left inputs, newest first.
    fn close_three(cap: usize) -> Vec<String> {
        let (mut store, _) = otd_settings::open(Location::Memory);
        let mut shell = Shell::new(crate::session_types::session_types());
        for name in ["a", "b", "c"] {
            let mut view = (otd_ui_text_compare::SESSION_TYPE.new)();
            if let Some(session) = view.session() {
                session.specs = vec![format!("/left/{name}.txt"), format!("/right/{name}.txt")];
            }
            shell.open(view);
        }
        for _ in 0..3 {
            shell.close_tab();
            closed(&mut store, &mut shell, cap);
        }
        auto_saved::load(&mut store, &[]).into_iter().map(|s| s.specs[0].clone()).collect()
    }

    #[test]
    fn closing_three_sessions_keeps_the_two_newest_and_a_cap_of_0_keeps_none() {
        // Close Tab closes the active tab, the last opened: c, then b, then a.
        assert_eq!(close_three(2), ["/left/a.txt", "/left/b.txt"]);
        assert_eq!(close_three(0), Vec::<String>::new());
    }

    #[test]
    fn clearing_a_session_auto_saves_the_one_it_replaced() {
        let (mut store, _) = otd_settings::open(Location::Memory);
        let mut shell = Shell::new(crate::session_types::session_types());
        let mut view = (otd_ui_text_compare::SESSION_TYPE.new)();
        if let Some(session) = view.session() {
            session.specs = vec!["/left/a.txt".into(), "/right/a.txt".into()];
        }
        shell.open(view);
        assert!(shell.clear_session().is_ok());
        closed(&mut store, &mut shell, 10);
        let saved: Vec<_> = auto_saved::load(&mut store, &[]).into_iter().map(|s| s.specs).collect();
        assert_eq!(saved, [["/left/a.txt", "/right/a.txt"]]);
    }
}
