use anyhow::Result;
use egui::{SelectableLabel, Ui};
use crate::backend::git_utils;
use crate::frontend::tab::OG2Tab;

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

    fn show_starting_btns(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Init").clicked() {
                // TODO: Implement Init
            }
            if ui.button("Open").clicked() {
                let repo_opt_opt = handle_error(git_utils::open_repo());
                // If it didn't throw an error
                if let Some(repo_opt) = repo_opt_opt {
                    // If a repo was actually opened
                    if let Some(repo) = repo_opt {

                        let mut name = String::from("(None)");
                        let repo_path = repo.path();
                        if let Some(repo_path_root) = repo_path.parent() {
                            if let Some(os_s) = repo_path_root.file_name() {
                                if let Some(s) = os_s.to_str() {
                                    name = String::from(s);
                                }
                            }
                        }

                        self.tabs.push(OG2Tab::new(name, repo));
                    }
                }
            }
            if ui.button("Clone").clicked() {
                // TODO: Implement Clone
            }
        });
    }

    fn show_tabs(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tab_width = ui.available_width() / self.tabs.len() as f32;
            for (i, tab) in self.tabs.iter().enumerate() {
                let selectable_label = SelectableLabel::new(self.active_tab == i, &tab.name);
                if ui.add_sized(egui::vec2(tab_width, 20.0), selectable_label).clicked() {
                    self.active_tab = i;
                }
            }
        });
    }
}

impl eframe::App for OG2App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_starting_btns(ui);
            self.show_tabs(ui);

            if self.tabs.len() > 0 {
                self.tabs[self.active_tab].show(ui);
            } else {
                // TODO: Add welcome splash screen?
            }
        });
    }
}
