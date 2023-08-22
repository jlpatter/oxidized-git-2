use crate::frontend::app::OG2App;

mod frontend;
mod backend;

fn main()  -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        // This is broken for some reason :(
        maximized: true,
        centered: true,
        app_id: Some("oxidized-git-2".to_owned()),
        ..Default::default()
    };
    eframe::run_native("Oxidized Git 2", options, Box::new(|cc| Box::new(OG2App::new(cc))))
}
