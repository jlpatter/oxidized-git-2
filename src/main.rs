use egui::ViewportBuilder;
use crate::frontend::app::OG2App;

mod frontend;
mod backend;

fn main()  -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_app_id("oxidized-git-2".to_owned()),
        centered: true,
        ..Default::default()
    };
    eframe::run_native("Oxidized Git 2", options, Box::new(|cc| Box::new(OG2App::new(cc))))
}
