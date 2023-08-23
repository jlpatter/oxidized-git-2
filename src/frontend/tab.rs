use anyhow::Result;
use egui::{Align, Color32, CursorIcon, Label, Layout, ScrollArea, Sense, Stroke, Ui};
use git2::Repository;
use crate::backend::git_utils;

pub struct OG2Tab {
    pub(crate) name: String,
    branch_trees: Vec<String>,
    branch_tree_col_width: f32,
}

impl OG2Tab {
    pub fn new(name: String, repo: Repository) -> Result<Self> {
        let branch_trees = git_utils::get_branch_trees(&repo)?;
        Ok(Self {
            name,
            branch_trees,
            branch_tree_col_width: 200.0,
        })
    }

    fn show_branch_tree(&mut self, ui: &mut Ui) {
        ScrollArea::both().max_width(self.branch_tree_col_width).auto_shrink([false, false]).show(ui, |ui| {
            ui.vertical(|ui| {
                for branch in &self.branch_trees {
                    ui.add(Label::new(branch).wrap(false));
                }
            })
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
