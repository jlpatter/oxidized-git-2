mod git_functions;

use anyhow::Result;
use egui::{Color32, Stroke};
use git2::Repository;

fn handle_error<T>(result: Result<T>) -> Option<T> {
    match result {
        Ok(t) => Some(t),
        Err(_e) => {
            // TODO: Handle errors in some way!
            None
        },
    }
}

fn main()  -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        min_window_size: Some(egui::vec2(840.0, 680.0)),
        // TODO: Figure out why this isn't working :(
        maximized: true,
        ..Default::default()
    };
    eframe::run_native("Oxidized Git 2", options, Box::new(|cc| Box::new(OG2App::new(cc))))
}


#[derive(Default)]
struct OG2App {
    repo: Option<Repository>,
}

impl OG2App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for OG2App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("Open").clicked() {
                        let repo_opt_opt = handle_error(git_functions::open_repo());
                        if let Some(repo_opt) = repo_opt_opt {
                            self.repo = repo_opt;
                        }
                    }
                    if ui.button("Fetch").clicked() {
                        // TODO: Implement Fetch
                        match &self.repo {
                            Some(r) => println!("{:?}", r.path()),
                            None => println!("None"),
                        };
                    }
                });

                // This is an example of how the graph could be rendered.
                let start_position = ui.cursor().left_top();
                let painter = ui.painter();
                painter.line_segment([start_position + egui::vec2(10.0, 10.0), start_position + egui::vec2(10.0, 40.0)], Stroke::new(3.0, Color32::RED));
                painter.circle_filled(start_position + egui::vec2(10.0, 10.0), 7.0, Color32::RED);
                painter.circle_filled(start_position + egui::vec2(10.0, 40.0), 7.0, Color32::RED);
            });
        });
    }
}
