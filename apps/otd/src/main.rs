//! The `otd` executable. For now it opens an empty main window; E01-S1 builds the real shell.

fn main() -> eframe::Result {
    // ponytail: empty window only; the tab strip, Home, menus and status bar are E01-S1's.
    eframe::run_native("OverTheDiff", eframe::NativeOptions::default(), Box::new(|_cc| Ok(Box::new(App))))
}

struct App;

impl eframe::App for App {
    fn ui(&mut self, _ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {}
}
