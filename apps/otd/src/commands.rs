//! The commands features add to the window, beside the shell's own session and tab commands. A feature adds
//! its `otd_ui::command::Command<Shell>`s here, one line each (`commands.extend(feature::commands())`).

use otd_ui::command::Command;

use crate::Shell;

pub fn commands() -> Vec<Command<Shell>> {
    let mut commands = Vec::new();
    commands.extend(otd_ui_text_compare::commands());
    commands
}

#[cfg(test)]
mod tests {
    use otd_ui::command::Platform;

    use super::*;

    /// The keymap conventions (Q24): within a view type no two commands share a default chord, an
    /// application-wide command holds its chord in every view type, and no default is a chord the
    /// OS reserves.
    #[test]
    fn the_registered_commands_default_shortcuts_neither_clash_nor_use_a_reserved_chord() {
        let shell = Shell::new(Vec::new()).with_commands(commands());
        assert_eq!(shell.commands().len(), Shell::new(Vec::new()).commands().len() + commands().len(), "every command id is unique");
        for platform in [Platform::Mac, Platform::Windows, Platform::Linux] {
            let keymap = otd_ui::keymap(shell.commands(), platform);
            assert_eq!(keymap.conflicts(), [], "on {platform:?}");
            let chords = shell.commands().iter().flat_map(|c| c.shortcuts.iter()).filter(|s| s.applies_on(platform));
            assert!(chords.clone().all(|s| !keymap.is_reserved(&s.chord)), "a reserved chord on {platform:?}");
        }
    }
}
