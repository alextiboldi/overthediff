//! Opening files into Text Compare (E07-S1-T2): *File > Open File* and the hidden *Open File on File
//! System*, and two files given on the command line.

use std::path::Path;

use otd_core::command::{Args, Command, CommandError, Kind, Outcome, Param};
use otd_core::i18n::{fmt_plain, t};
use otd_launch::{Completion, Handler, Pattern, Request};
use otd_ui::Shell;

use crate::{LEFT, RIGHT, SIDES, TEXT_COMPARE, TextCompareView};

/// The File menu's command, which loads a file into the chosen pane.
pub const OPEN_FILE: &str = "Open File";
/// The same, with no menu entry until the user adds one (hidden by default).
pub const OPEN_FILE_ON_FILE_SYSTEM: &str = "Open File on File System";
pub(crate) const NOT_TEXT_COMPARE: &str = "The current tab isn't a Text Compare session.";
const TWO_FILES: &str = "Comparing files needs two files, not {count}.";

/// `side` is Left or Right, the chosen pane when left out; `path` loads that file, and without it
/// the pane's file picker opens.
const PARAMS: &[Param] = &[Param::optional("side", Kind::Choice(&[LEFT, RIGHT])), Param::optional("path", Kind::Path)];

/// Text Compare's commands, for `apps/otd/src/commands.rs`: these two, the File menu's save
/// commands (`save.rs`), then the Session menu's ([`crate::session`]), the Edit menu's, the hidden font size
/// commands (`font_size.rs`), the Search menu's Go To (`go_to.rs`), bookmarks and Next and Previous Edit
/// (`edited.rs`), and the View menu's layouts.
// ponytail: both open the OS picker on the local file system, the one data source with a picker so
// far; Open File browses other sources once they have one.
pub fn commands() -> Vec<Command<Shell>> {
    let open = [
        Command::typed("text-compare.open_file", OPEN_FILE, PARAMS, open_file).views(&[TEXT_COMPARE]).icon("folder-open").menu(&["File"]),
        Command::typed("text-compare.open_file_on_file_system", OPEN_FILE_ON_FILE_SYSTEM, PARAMS, open_file).views(&[TEXT_COMPARE]),
    ];
    open.into_iter()
        .map(Command::needs_window)
        .chain(crate::save::commands())
        .chain(crate::session::commands())
        .chain(crate::edit::commands())
        .chain(crate::font_size::commands())
        .chain(crate::go_to::commands())
        .chain(crate::bookmarks::commands())
        .chain(crate::layout::commands())
        .chain(crate::edited::commands())
        .chain(crate::whitespace::commands())
        .collect()
}

fn open_file(shell: &mut Shell, args: &Args) -> Outcome {
    let view = shell.view_mut::<TextCompareView>().ok_or_else(|| CommandError::Failed(t(NOT_TEXT_COMPARE).to_owned()))?;
    let side = args.text("side").and_then(|side| SIDES.iter().position(|s| *s == side));
    if view.open_file(side, args.text("path")) {
        shell.save_first(crate::load::load_pending);
    }
    Ok(None)
}

/// The command line's bare paths, `otd <left> <right>`: one Text Compare tab with those files.
pub struct OpenFiles;

impl Handler<Shell> for OpenFiles {
    fn id(&self) -> &'static str {
        TEXT_COMPARE
    }

    fn pattern(&self) -> Pattern {
        Pattern::BarePaths
    }

    fn usage(&self) -> &'static str {
        "<left> <right>   Compare two text files"
    }

    fn check(&self, args: &[String]) -> Result<(), String> {
        if args.len() == SIDES.len() { Ok(()) } else { Err(fmt_plain(t(TWO_FILES), &[("count", &args.len().to_string())])) }
    }

    fn open(&self, request: Request, done: Completion, shell: &mut Shell) {
        let mut view = TextCompareView::new();
        view.session.specs = request.args.iter().map(|arg| absolute(&request.cwd, arg)).collect();
        view.done = Some(done);
        shell.open(Box::new(view));
    }
}

/// `spec` with a relative path made relative to `cwd`, the launching process's working folder,
/// since the instance's differs. `~`, `~/…` and `smb://…` are left for the loader, which expands
/// them; any other name starting with `~` (`~notes.txt`) is a relative file name.
fn absolute(cwd: &Path, spec: &str) -> String {
    let home = spec.strip_prefix('~').is_some_and(|rest| rest.is_empty() || rest.starts_with('/') || (cfg!(windows) && rest.starts_with('\\')));
    if home || spec.contains("://") || Path::new(spec).is_absolute() { spec.to_owned() } else { cwd.join(spec).display().to_string() }
}
