//! The path edit (E01-S1-T9): a single-line edit for a typed path, Browse for File and Browse for
//! Folder through the OS picker, and a dropdown of the paths recently used in that field.
//!
//! A feature draws one with [`PathEdit`], naming its field with a stable id such as
//! `"text-compare.left"`; fields with the same id share their recent paths. The app owns one
//! [`PathEdits`]: it shows the OS pickers the edits ask for and keeps the recent paths in the
//! settings store, so they survive restarts. Nothing needs registering.
//!
//! The text is the user's: it isn't checked or normalised, since later stories accept URLs,
//! `clipboard://` and paths inside archives here.
//!
//! A craft-libs candidate (alextiboldi/overthediff#60): it stays here until craft-libs has it.

use std::path::Path;
use std::sync::Arc;

use craft_dialogs::{Dialogs, FileDialog};
use egui::accesskit::Live;
use egui::{Context, Id, Key, Response, TextEdit, Ui};
use otd_core::i18n::{fmt, t};
use otd_settings::Store;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use serde::{Deserialize, Serialize};

pub use craft_dialogs::{Answer, Kind as PickerKind, Override, Request};

// English keys, drawn only through `t`.
pub const BROWSE_FOR_FILE: &str = "Browse for File";
pub const BROWSE_FOR_FOLDER: &str = "Browse for Folder";
pub const RECENT_PATHS: &str = "Recent paths";
pub const NO_RECENT_PATHS: &str = "No recent paths";
const PICKER_UNAVAILABLE: &str = "The file picker couldn't be shown: {reason}";
const NOT_UNICODE: &str = "The chosen path can't be used, because it isn't valid Unicode: {path}";

/// The settings section that holds the recent paths.
pub const SECTION: &str = "recent-paths";

/// How many paths a field keeps, newest first.
// ponytail: fixed until E22-S5 adds the Max most recently used items tweak.
pub const MAX_RECENT: usize = 20;

/// Where the edits and [`PathEdits`] meet in the egui context.
const SHARED: &str = "otd-path-edits";

/// Which Browse buttons a path edit offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Browse {
    File,
    Folder,
    /// Both buttons, for a field that takes either.
    Both,
}

/// The recent paths of every field, as the settings store keeps them: arrays, which a save
/// replaces whole, so a dropped path stays dropped (otd-settings).
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct RecentPaths {
    fields: Vec<RecentField>,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
#[serde(default)]
struct RecentField {
    field: String,
    paths: Vec<String>,
}

impl RecentPaths {
    /// `field`'s recent paths, newest first.
    pub fn paths(&self, field: &str) -> &[String] {
        self.fields.iter().find(|f| f.field == field).map_or(&[], |f| &f.paths)
    }

    /// Puts `path` first in `field`'s list, once, keeping at most [`MAX_RECENT`].
    pub fn push(&mut self, field: &str, path: &str) {
        let index = match self.fields.iter().position(|f| f.field == field) {
            Some(index) => index,
            None => {
                self.fields.push(RecentField { field: field.to_owned(), paths: Vec::new() });
                self.fields.len() - 1
            }
        };
        let paths = &mut self.fields[index].paths;
        paths.retain(|p| p != path);
        paths.insert(0, path.to_owned());
        paths.truncate(MAX_RECENT);
    }
}

/// What the edits and [`PathEdits`] pass each other through the egui context.
#[derive(Clone, Default)]
struct Shared {
    recent: RecentPaths,
    /// `recent` changed since it was last saved.
    changed: bool,
    /// Pickers asked for, by the asking edit's id.
    requests: Vec<(Id, FileDialog)>,
    /// Picker answers, for the asking edit to take on its next draw.
    answers: Vec<(Id, Answer)>,
    /// The edits drawn since the last [`PathEdits::frame`].
    drawn: Vec<Id>,
}

fn shared<R>(ctx: &Context, f: impl FnOnce(&mut Shared) -> R) -> R {
    ctx.data_mut(|data| f(data.get_temp_mut_or_default::<Shared>(Id::unique(SHARED))))
}

/// The app's side of path edits: it shows the OS pickers they ask for, hands back the answers
/// and saves their recent paths. One per window.
pub struct PathEdits {
    dialogs: Dialogs<Id>,
    /// The edit whose picker is open, and whether a frame went by without drawing it.
    asker: Option<(Id, bool)>,
}

impl PathEdits {
    /// Loads the recent paths from `store` for the edits drawn in `ctx`.
    pub fn open(ctx: &Context, store: &mut Store) -> Self {
        let mut recent: RecentPaths = store.section(SECTION);
        // A hand-edited file may hold more than an edit would keep.
        for field in &mut recent.fields {
            field.paths.truncate(MAX_RECENT);
        }
        shared(ctx, |shared| shared.recent = recent);
        let wake = ctx.clone();
        Self { dialogs: Dialogs::new(Arc::new(move || wake.request_repaint())), asker: None }
    }

    /// Answers every picker with `answer_with` instead of showing the OS one (tests and
    /// automation); `None` shows the OS pickers again.
    pub fn set_override(&mut self, answer_with: Option<Override>) {
        self.dialogs.set_override(answer_with);
    }

    /// Call at the end of every frame, with the window as `parent`: shows the pickers asked for,
    /// passes their answers to the edits and saves changed recent paths to `store`.
    pub fn frame<W: HasWindowHandle + HasDisplayHandle>(&mut self, ctx: &Context, parent: Option<&W>, store: &mut Store) {
        let (requests, drawn, changed) = shared(ctx, |shared| {
            let changed = std::mem::take(&mut shared.changed).then(|| shared.recent.clone());
            (std::mem::take(&mut shared.requests), std::mem::take(&mut shared.drawn), changed)
        });
        // An edit that went undrawn has gone (its dialog or tab closed); one drawn later with the
        // same id is another, so the old picker's answer mustn't fill it.
        if let Some((asker, gone)) = &mut self.asker {
            *gone |= !drawn.contains(asker);
        }
        for (id, dialog) in requests {
            if self.dialogs.is_open() || self.dialogs.queued() > 0 {
                // One picker at a time: Browse while the macOS panel is open brings it forward.
                self.dialogs.raise();
            } else {
                self.dialogs.ask(dialog, id);
                self.asker = Some((id, false));
            }
        }
        self.dialogs.show_pending(parent);
        let mut answers = self.dialogs.take_answers();
        if !answers.is_empty() {
            ctx.request_repaint();
        }
        if !self.dialogs.is_open()
            && let Some((asker, true)) = self.asker.take()
        {
            answers.retain(|(id, _)| *id != asker);
        }
        // Answers posted last frame that no edit took were for an edit no longer drawn (its
        // dialog or tab closed while the picker was open), so they go rather than fill a later one.
        shared(ctx, |shared| shared.answers = answers);
        if let Some(recent) = changed {
            // Strings always serialise, so `set` can't fail here.
            store.update(|sections| sections.set(SECTION, &recent).unwrap_or(()));
        }
    }
}

/// A path edit for one field. [`Self::show`] returns the text edit's response, which is changed
/// whenever the path is: typed, browsed or picked from the recent paths.
pub struct PathEdit<'a> {
    field: &'a str,
    path: &'a mut String,
    browse: Browse,
}

impl<'a> PathEdit<'a> {
    /// An edit of `path` for the field `field`, offering the `browse` buttons.
    pub fn new(field: &'a str, path: &'a mut String, browse: Browse) -> Self {
        Self { field, path, browse }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let Self { field, path, browse } = self;
        let id = ui.make_persistent_id(("otd-path-edit", field));
        let ctx = ui.ctx().clone();
        shared(&ctx, |shared| {
            if !shared.drawn.contains(&id) {
                shared.drawn.push(id);
            }
        });
        let mut chosen = None;
        let mut error: Option<String> = ctx.data(|data| data.get_temp(id.with("error")));
        let answers: Vec<Answer> = shared(&ctx, |shared| {
            let (mine, others) = std::mem::take(&mut shared.answers).into_iter().partition(|(asker, _)| *asker == id);
            shared.answers = others;
            mine.into_iter().map(|(_, answer)| answer).collect()
        });
        for answer in answers {
            match answer {
                Answer::Paths(paths) => match paths.into_iter().next().map(|p| p.into_os_string().into_string()) {
                    Some(Ok(text)) => chosen = Some(text),
                    Some(Err(raw)) => error = Some(fmt(t(NOT_UNICODE), &[("path", &raw.to_string_lossy())])),
                    None => {}
                },
                Answer::Unavailable(reason) => error = Some(fmt(t(PICKER_UNAVAILABLE), &[("reason", &reason)])),
                Answer::Cancelled | Answer::Message(_) => {}
            }
        }

        ui.horizontal(|ui| {
            let mut response = ui.add(TextEdit::singleline(path).id(id));
            // A typed path counts as used when the edit loses focus (Enter does that), not per key.
            let dirty = id.with("dirty");
            if response.changed() {
                ctx.data_mut(|data| data.insert_temp(dirty, true));
            }
            if response.lost_focus() && ctx.data_mut(|data| data.remove_temp::<bool>(dirty)).unwrap_or(false) {
                use_path(&ctx, field, path);
                error = None;
            }
            for (offered, kind, caption) in
                [(Browse::File, PickerKind::Open { multiple: false }, BROWSE_FOR_FILE), (Browse::Folder, PickerKind::Folder, BROWSE_FOR_FOLDER)]
            {
                if (browse == offered || browse == Browse::Both) && ui.button(t(caption)).clicked() {
                    let mut dialog = FileDialog::new(kind).title(t(caption));
                    if let Some(dir) = start_folder(path) {
                        dialog = dialog.directory(dir);
                    }
                    shared(&ctx, |shared| shared.requests.push((id, dialog)));
                }
            }
            let recent = egui::ComboBox::from_id_salt(id.with("recent")).selected_text("").width(0.0).show_ui(ui, |ui| {
                let paths = shared(&ctx, |shared| shared.recent.paths(field).to_vec());
                if paths.is_empty() {
                    ui.add_enabled(false, egui::Label::new(t(NO_RECENT_PATHS)));
                }
                for recent in paths {
                    if ui.selectable_label(false, &recent).clicked() {
                        chosen = Some(recent);
                    }
                }
            });
            // egui drops focus on Esc; closing the list with it puts focus back on the dropdown.
            if recent.inner.is_some() && ctx.input(|i| i.key_pressed(Key::Escape)) {
                recent.response.request_focus();
            }
            ctx.accesskit_node_builder(recent.response.id, |node| node.set_label(t(RECENT_PATHS)));
            recent.response.on_hover_text(t(RECENT_PATHS));
            if let Some(text) = chosen {
                *path = text;
                use_path(&ctx, field, path);
                error = None;
                response.mark_changed();
                response.request_focus();
            }
            match &error {
                Some(text) => {
                    let label = ui.label(text);
                    // Announced: it comes from the picker, away from focus.
                    ctx.accesskit_node_builder(label.id, |node| node.set_live(Live::Polite));
                    ctx.data_mut(|data| data.insert_temp(id.with("error"), text.clone()));
                }
                None => {
                    ctx.data_mut(|data| data.remove_temp::<String>(id.with("error")));
                }
            }
            response
        })
        .inner
    }
}

/// Records `path` as used in `field`, for [`PathEdits`] to save. An empty edit isn't a path.
fn use_path(ctx: &Context, field: &str, path: &str) {
    if path.trim().is_empty() {
        return;
    }
    shared(ctx, |shared| {
        shared.recent.push(field, path);
        shared.changed = true;
    });
}

/// The folder a picker starts in: the typed path when it is a folder, else its parent folder.
fn start_folder(path: &str) -> Option<&Path> {
    let path = Path::new(path);
    if path.as_os_str().is_empty() {
        return None;
    }
    if path.is_dir() { Some(path) } else { path.parent().filter(|parent| parent.is_dir()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_used_again_moves_first_and_is_listed_once() {
        let mut recent = RecentPaths::default();
        for path in ["p0", "p1", "p2"] {
            recent.push("a", path);
        }
        recent.push("a", "p1");
        assert_eq!(recent.paths("a"), ["p1", "p2", "p0"]);
    }
}
