mod git_functions;

use anyhow::Result;
use egui::{Color32, Stroke, Vec2};
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
        initial_window_size: Some(egui::vec2(840.0, 680.0)),
        ..Default::default()
    };

    // Our application state (put persistent stuff here):
    let mut repo: Option<Repository> = None;

    eframe::run_simple_native("Oxidized Git 2", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("Open").clicked() {
                        let repo_opt_opt = handle_error(git_functions::open_repo());
                        if let Some(repo_opt) = repo_opt_opt {
                            repo = repo_opt;
                        }
                    }
                    if ui.button("Fetch").clicked() {
                        // TODO: Implement Fetch
                        match &repo {
                            Some(r) => println!("{:?}", r.path()),
                            None => println!("None"),
                        };
                    }
                });

                // This is an example of how the graph could be rendered.
                let start_position = ui.cursor().left_top();
                let painter = ui.painter();
                painter.line_segment([start_position + Vec2::new(10.0, 10.0), start_position + Vec2::new(10.0, 40.0)], Stroke::new(3.0, Color32::RED));
                painter.circle_filled(start_position + Vec2::new(10.0, 10.0), 7.0, Color32::RED);
                painter.circle_filled(start_position + Vec2::new(10.0, 40.0), 7.0, Color32::RED);
            });
        });
    })
}
