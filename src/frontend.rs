use anyhow::Result;
use egui::{Color32, Stroke, Ui};
use git2::Repository;
use crate::git_functions;

fn handle_error<T>(result: Result<T>) -> Option<T> {
    match result {
        Ok(t) => Some(t),
        Err(_e) => {
            // TODO: Handle errors in some way!
            None
        },
    }
}

#[derive(Default)]
pub struct OG2App {
    tabs: Vec<OG2Tab>,
    active_tab: usize,
}

impl OG2App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }

    fn show_welcome_page(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Open").clicked() {
                let repo_opt_opt = handle_error(git_functions::open_repo());
                // If it didn't throw an error
                if let Some(repo_opt) = repo_opt_opt {
                    // If a repo was actually opened
                    if let Some(repo) = repo_opt {
                        self.tabs.push(OG2Tab::new(repo));
                    }
                }
            }
        });
    }
}

impl eframe::App for OG2App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.tabs.len() > 0 {
                self.tabs[self.active_tab].show(ui);
            } else {
                self.show_welcome_page(ui);
            }
        });
    }
}

pub struct OG2Tab {
    repo: Repository,
}

impl OG2Tab {
    fn new(repo: Repository) -> Self {
        Self {
            repo
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if ui.button("Fetch").clicked() {
                    // TODO: Implement Fetch
                }
            });

            // This is an example of how the graph could be rendered.
            let start_position = ui.cursor().left_top();
            let painter = ui.painter();
            painter.line_segment([start_position + egui::vec2(10.0, 10.0), start_position + egui::vec2(10.0, 40.0)], Stroke::new(3.0, Color32::RED));
            painter.circle_filled(start_position + egui::vec2(10.0, 10.0), 7.0, Color32::RED);
            painter.circle_filled(start_position + egui::vec2(10.0, 40.0), 7.0, Color32::RED);
        });
    }
}
