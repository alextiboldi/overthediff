//! The Options dialog (E01-S1-T6): Tools > Options, and Options in the macOS application menu.
//!
//! Features contribute [`OptionSpec`]s to named pages ([`Shell::with_options`]). The dialog shows a
//! page tree of the pages something contributed to (and the pages above them), and the selected
//! page's options. It edits a copy of the values: Apply makes the copy the values and stays open,
//! OK does the same and closes, and Cancel and Esc drop the copy. A click outside it does nothing.
//! The app saves the applied values ([`Shell::take_applied_options`]); any view reads one with
//! [`option`].

use std::collections::BTreeMap;
use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Context, Id, Key, Modifiers, Ui};
use otd_core::i18n::{fmt, t};
pub use otd_core::options::{DuplicateOption, OptionSpec};
use otd_core::session::Value;

use crate::session::{OK, edit};
use crate::{CANCEL, Shell};

// The dialog's own text: English keys, drawn only through `t`, as in lib.rs.
/// The dialog's title and the command's caption.
pub const OPTIONS: &str = "Options";
pub const APPLY: &str = "Apply";
/// The page tree's name.
pub const OPTION_PAGES: &str = "Option pages";
pub const NO_OPTIONS: &str = "No options are available yet.";
/// What a page with no options of its own shows; `{page}` is its name.
pub const CHOOSE_A_PAGE: &str = "Choose one of the pages under {page}.";

const VALUES: &str = "otd-option-values";

type Values = BTreeMap<String, Value>;

/// The applied value of the option `id`, from any view: `None` if no option has that id.
pub fn option(ctx: &Context, id: &str) -> Option<Value> {
    let values: Option<Arc<Values>> = ctx.data(|data| data.get_temp(Id::unique(VALUES)));
    values?.get(id).cloned()
}

/// The contributed options, their applied values and the open dialog.
#[derive(Default)]
pub(crate) struct Options {
    specs: Vec<OptionSpec>,
    values: Arc<Values>,
    /// Applied since the app last took the values to save.
    applied: bool,
    pub(crate) dialog: Option<Dialog>,
}

impl Options {
    /// Makes the applied values the ones [`option`] reads.
    pub(crate) fn publish(&self, ctx: &Context) {
        ctx.data_mut(|data| data.insert_temp(Id::unique(VALUES), self.values.clone()));
    }
}

/// The open Options dialog.
pub(crate) struct Dialog {
    /// The copy Apply and OK make the values.
    working: Values,
    /// The page tree, depth first: each page's path.
    pages: Vec<&'static [&'static str]>,
    /// The selected page, in `pages`.
    selected: usize,
    /// Focus moves to the selected page on the next draw.
    focus: bool,
    /// The tree's items drawn last frame, in order: Up and Down move between them.
    items: Vec<Id>,
    /// The single-line edits drawn last frame, where Enter is OK.
    single_line: Vec<Id>,
}

impl Shell {
    /// Adds the options features contribute, with their saved values (`otd_settings::options::read`);
    /// an option without one starts at its default. Fails when two options share an id.
    pub fn with_options(mut self, options: Vec<OptionSpec>, saved: Values) -> Result<Self, DuplicateOption> {
        self.options.specs.extend(options);
        otd_core::options::check_unique(&self.options.specs)?;
        let mut values = saved;
        for spec in &self.options.specs {
            values.entry(spec.id.to_owned()).or_insert_with(|| spec.default.clone());
        }
        self.options.values = Arc::new(values);
        Ok(self)
    }

    /// Tools > Options: opens the dialog on the first page that has options.
    pub fn open_options(&mut self) {
        let specs = &self.options.specs;
        let pages = tree(specs);
        let selected = pages.iter().position(|page| specs.iter().any(|s| s.page == *page)).unwrap_or(0);
        let working = (*self.options.values).clone();
        self.options.dialog = Some(Dialog { working, pages, selected, focus: true, items: Vec::new(), single_line: Vec::new() });
    }

    /// The values, when Apply or OK changed them since the last call: what the app saves.
    pub fn take_applied_options(&mut self) -> Option<Values> {
        std::mem::take(&mut self.options.applied).then(|| (*self.options.values).clone())
    }

    /// Draws the Options dialog, if open, over the window.
    pub(crate) fn options_ui(&mut self, ctx: &Context) {
        let Options { specs, values, applied, dialog: Some(dialog) } = &mut self.options else { return };
        let enter = ctx.memory(|m| m.focused()).is_some_and(|id| dialog.single_line.contains(&id)) && ctx.input(|i| i.key_pressed(Key::Enter));
        let (mut ok, mut apply, mut cancel) = (enter, false, false);
        let id = Id::unique("otd-options");
        let modal = egui::Modal::new(id).show(ctx, |ui| {
            // The modal's area is its Dialog node; it takes the area's id.
            ui.ctx().accesskit_node_builder(id, |node| node.set_label(t(OPTIONS)));
            ui.heading(t(OPTIONS));
            dialog.single_line.clear();
            if specs.is_empty() {
                ui.label(t(NO_OPTIONS));
            } else {
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| dialog.tree_ui(ui));
                    ui.separator();
                    ui.vertical(|ui| dialog.page_ui(ui, specs));
                });
            }
            ui.separator();
            ui.horizontal(|ui| {
                ok |= ui.button(t(OK)).clicked();
                cancel = ui.button(t(CANCEL)).clicked();
                apply = ui.button(t(APPLY)).clicked();
            });
        });
        // Esc closes it as Cancel does, unless it closes a popup first.
        let escape = modal.is_top_modal && !modal.any_popup_open && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        if ok || apply {
            *values = Arc::new(dialog.working.clone());
            *applied = true;
            // Views read the new values from the next frame on.
            ctx.request_repaint();
        }
        if ok || cancel || escape {
            self.options.dialog = None;
            // Focus goes back to the window: its selected tab.
            self.focus_active = true;
        }
    }
}

/// Every page an option names and every page above one, depth first, each level in the order its
/// pages were first contributed.
fn tree(specs: &[OptionSpec]) -> Vec<&'static [&'static str]> {
    let mut known: Vec<&'static [&'static str]> = Vec::new();
    for spec in specs {
        for depth in 1..=spec.page.len() {
            let page = &spec.page[..depth];
            if !known.contains(&page) {
                known.push(page);
            }
        }
    }
    fn under(parent: &[&str], known: &[&'static [&'static str]], tree: &mut Vec<&'static [&'static str]>) {
        for page in known.iter().filter(|p| p.len() == parent.len() + 1 && p.starts_with(parent)) {
            tree.push(page);
            under(page, known, tree);
        }
    }
    let mut pages = Vec::new();
    under(&[], &known, &mut pages);
    pages
}

impl Dialog {
    /// The page tree. Up and Down move the selection, and focus with it, while an item has focus;
    /// Tab reaches each item too.
    fn tree_ui(&mut self, ui: &mut Ui) {
        let focused = ui.ctx().memory(|m| m.focused());
        if let Some(at) = focused.and_then(|f| self.items.iter().position(|item| *item == f)) {
            let (up, down) = ui.input_mut(|i| (i.consume_key(Modifiers::NONE, Key::ArrowUp), i.consume_key(Modifiers::NONE, Key::ArrowDown)));
            let to = if up { at.checked_sub(1) } else { Some(at + 1).filter(|to| down && *to < self.pages.len()) };
            if let Some(to) = to {
                self.selected = to;
                self.focus = true;
            }
        }
        ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
            node.set_role(Role::Tree);
            node.set_label(t(OPTION_PAGES));
        });
        self.items.clear();
        for (index, page) in self.pages.iter().enumerate() {
            let selected = index == self.selected;
            let depth = page.len();
            let name = page.last().map_or("", |name| t(name));
            let response = ui
                .horizontal(|ui| {
                    ui.add_space(ui.spacing().indent * depth.saturating_sub(1) as f32);
                    ui.selectable_label(selected, name)
                })
                .inner;
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_role(Role::TreeItem);
                node.clear_toggled();
                node.set_selected(selected);
                node.set_level(depth);
            });
            if selected && self.focus {
                response.request_focus();
            }
            // The tree, not egui's arrow-key focus moves, takes Up and Down while an item has focus.
            let keeps_arrows = egui::EventFilter { vertical_arrows: true, ..Default::default() };
            ui.memory_mut(|m| m.set_focus_lock_filter(response.id, keeps_arrows));
            if response.clicked() {
                self.selected = index;
            }
            self.items.push(response.id);
        }
        self.focus = false;
    }

    /// The selected page's options, in the order they were contributed.
    fn page_ui(&mut self, ui: &mut Ui, specs: &[OptionSpec]) {
        let Some(&page) = self.pages.get(self.selected) else { return };
        let mut any = false;
        for spec in specs.iter().filter(|s| s.page == page) {
            any = true;
            let value = self.working.entry(spec.id.to_owned()).or_insert_with(|| spec.default.clone());
            let single = matches!(value, Value::Text(_) | Value::Path(_));
            let response = ui.push_id(spec.id, |ui| edit(ui, &format!("options/{}", spec.id), spec.id, spec.label, spec.choices, value)).inner;
            if single {
                self.single_line.push(response.id);
            }
        }
        if !any && let Some(name) = page.last() {
            // One line: the page column is only as wide as its contents.
            ui.add(egui::Label::new(fmt(t(CHOOSE_A_PAGE), &[("page", t(name))])).extend());
        }
    }
}
