//! The file info header, read back through the accessibility tree.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use otd_ui::View as _;
use otd_ui_text_compare::TextCompareView;

#[test]
fn each_pane_shows_its_decoded_text_with_the_detected_encoding_and_line_endings() {
    let folder = tempfile::tempdir().unwrap();
    let left = folder.path().join("left.txt");
    let right = folder.path().join("right.txt");
    // UTF-16 LE with a mark and Windows breaks; UTF-8 without one, with a blank line and Mac breaks.
    let utf16: Vec<u8> = [0xFF, 0xFE].into_iter().chain("café\r\nend\r\n".encode_utf16().flat_map(u16::to_le_bytes)).collect();
    std::fs::write(&left, utf16).unwrap();
    std::fs::write(&right, "naïve\r\rend\r").unwrap();

    let mut view = TextCompareView::new();
    if let Some(session) = view.session() {
        session.specs = vec![left.display().to_string(), right.display().to_string()];
    }
    let mut harness = Harness::builder().build_ui_state(|ui, view: &mut TextCompareView| view.ui(ui), view);
    harness.run();

    for name in ["café", "naïve", "Encoding: UTF-16 LE with BOM", "Line endings: Windows (CRLF)", "Encoding: UTF-8", "Line endings: Mac (CR)"] {
        assert!(harness.query_by_label(name).is_some(), "{name} is shown");
    }
    let path = format!("Path: {}", left.display());
    assert!(harness.query_by_label(&path).is_some(), "the left header names its file");
}
