use anyhow::Result;
use egui::{Align, Color32, CursorIcon, Label, Layout, ScrollArea, Sense, Stroke, Ui};
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

    fn show_starting_btns(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Init").clicked() {
                // TODO: Implement Init
            }
            if ui.button("Open").clicked() {
                let repo_opt_opt = handle_error(git_functions::open_repo());
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
        // TODO: Figure out how to make a layout for tabs.
        ui.horizontal(|ui| {
            for (i, tab) in self.tabs.iter().enumerate() {
                if ui.selectable_label(self.active_tab == i, &tab.name).clicked() {
                    self.active_tab = i;
                }
            }
        });
    }
}

impl eframe::App for OG2App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("my_panel").show_separator_line(false).show(ctx, |ui| {
            self.show_starting_btns(ui);
            self.show_tabs(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.tabs.len() > 0 {
                self.tabs[self.active_tab].show(ui);
            } else {
                // TODO: Add welcome splash screen?
            }
        });
    }
}

pub struct OG2Tab {
    name: String,
    repo: Repository,
    branch_tree_col_width: f32,
}

impl OG2Tab {
    fn new(name: String, repo: Repository) -> Self {
        Self {
            name,
            repo,
            branch_tree_col_width: 200.0,
        }
    }

    fn show_branch_tree(&mut self, ui: &mut Ui) {
        ScrollArea::both().max_width(self.branch_tree_col_width).auto_shrink([false, false]).show(ui, |ui| {
            ui.vertical(|ui| {
                // TODO: Insert actual branches here!
                ui.add(Label::new("BLURG 1asdfasdfasdfasdfasdfasdfasdfasdfasdfasdfasdfasdfasdfsdf").wrap(false));
                ui.label("BLURG 2");
            });
        });

        // Add draggable separator.
        let separator_resp = ui.separator().interact(Sense::click_and_drag()).on_hover_and_drag_cursor(CursorIcon::ResizeHorizontal);
        if separator_resp.dragged() {
            self.branch_tree_col_width += separator_resp.drag_delta().x;
        }
    }

    fn show_graph(&mut self, ui: &mut Ui) {
        // TODO: Show Graph.
        // This is an example of how the graph could be rendered.
        let start_position = ui.cursor().left_top();
        let painter = ui.painter();
        painter.line_segment([start_position + egui::vec2(10.0, 10.0), start_position + egui::vec2(10.0, 40.0)], Stroke::new(3.0, Color32::RED));
        painter.circle_filled(start_position + egui::vec2(10.0, 10.0), 7.0, Color32::RED);
        painter.circle_filled(start_position + egui::vec2(10.0, 40.0), 7.0, Color32::RED);
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if ui.button("Fetch").clicked() {
                    // TODO: Implement Fetch
                }
                if ui.button("Pull").clicked() {
                    // TODO: Implement Pull
                }
                if ui.button("Push").clicked() {
                    // TODO: Implement Push
                }
            });

            ui.with_layout(Layout::top_down(Align::Min).with_main_justify(true), |ui| {
                ui.horizontal(|ui| {
                    self.show_branch_tree(ui);
                    self.show_graph(ui);
                });
            });
        });
    }
}
