use crate::frontend::OG2App;

mod git_functions;
mod frontend;

fn main()  -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        min_window_size: Some(egui::vec2(840.0, 680.0)),
        // TODO: Figure out why this isn't working :(
        maximized: true,
        ..Default::default()
    };
    eframe::run_native("Oxidized Git 2", options, Box::new(|cc| Box::new(OG2App::new(cc))))
}
