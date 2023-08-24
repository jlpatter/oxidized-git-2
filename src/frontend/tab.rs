use anyhow::Result;
use egui::{Align, CursorIcon, Layout, ScrollArea, Sense, Ui};
use git2::Repository;
use crate::frontend::branch_tree::{BranchTreeNode, get_branch_trees};
use crate::frontend::commit_graph::CommitGraph;

pub struct OG2Tab {
    pub(crate) name: String,
    branch_trees: [BranchTreeNode; 3],
    branch_tree_col_width: f32,
    commit_graph: CommitGraph,
}

impl OG2Tab {
    pub fn new(name: String, repo: Repository) -> Result<Self> {
        let branch_trees = get_branch_trees(&repo)?;
        Ok(Self {
            name,
            branch_trees,
            branch_tree_col_width: 200.0,
            commit_graph: CommitGraph::new(&repo)?,
        })
    }

    fn show_branch_tree_col(&mut self, ui: &mut Ui) {
        ScrollArea::both().id_source("branch-tree-col-scroll-area").max_width(self.branch_tree_col_width).auto_shrink([false, false]).show(ui, |ui| {
            ui.vertical(|ui| {
                for branch_tree in &mut self.branch_trees {
                    branch_tree.show(ui, 0.0);
                }
            })
        });

        // Add draggable separator.
        let separator_resp = ui.separator().interact(Sense::click_and_drag()).on_hover_and_drag_cursor(CursorIcon::ResizeHorizontal);
        if separator_resp.dragged() {
            self.branch_tree_col_width += separator_resp.drag_delta().x;
        }
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
                    self.show_branch_tree_col(ui);
                    self.commit_graph.show(ui);
                });
            });
        });
    }
}
