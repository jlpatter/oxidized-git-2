mod git_functions;

use anyhow::Result;
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
        });
    })
}
