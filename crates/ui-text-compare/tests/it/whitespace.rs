//! *View > Show Whitespace*, read back from the frame's painted text.

use std::collections::BTreeSet;

use egui::{FontId, Shape};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use otd_ui::{Shell, View as _};
use otd_ui_text_compare::{SESSION_TYPE, TextCompareView, commands};

/// Every text painted in the last frame.
fn painted(harness: &Harness<'_, Shell>) -> BTreeSet<String> {
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            Shape::Text(text) => Some(text.galley.text().to_owned()),
            _ => None,
        })
        .collect()
}

#[test]
fn show_whitespace_marks_spaces_tabs_and_each_line_ending_style_with_its_own_glyph() {
    let folder = tempfile::tempdir().unwrap();
    // A space, a tab, and a Windows, a Mac and a Unix line break.
    let text = "a b\r\nc\td\re\n";
    let specs: Vec<String> = ["left.txt", "right.txt"]
        .iter()
        .map(|name| {
            let path = folder.path().join(name);
            std::fs::write(&path, text).unwrap();
            path.display().to_string()
        })
        .collect();
    let mut view = TextCompareView::new();
    view.session().expect("a session").specs = specs;
    let mut shell = Shell::new(vec![SESSION_TYPE]).with_commands(commands());
    shell.open(Box::new(view));
    let mut harness = Harness::builder().build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), shell);
    harness.run();
    let off = painted(&harness);

    assert!(harness.state_mut().run("text-compare.show_whitespace"), "the command ran");
    harness.run();
    let marks: BTreeSet<String> = painted(&harness).difference(&off).cloned().collect();
    assert_eq!(marks.len(), 5, "a space, a tab and three line-ending styles, each its own glyph: {marks:?}");
    let font = FontId::monospace(14.0);
    for mark in &marks {
        assert!(harness.ctx.fonts_mut(|fonts| fonts.has_glyphs(&font, mark)), "{mark:?} has a glyph in the editor's font");
    }
    assert!(harness.query_all_by_label("a b").next().is_some(), "a line is still named by its own text");

    assert!(harness.state_mut().run("text-compare.show_whitespace"), "the command ran again");
    harness.run();
    assert!(painted(&harness).is_disjoint(&marks), "turned off, the glyphs are gone");
}
