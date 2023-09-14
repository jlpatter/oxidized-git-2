use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::Result;
use egui::{Align, Context, CursorIcon, Layout, ScrollArea, Sense, Ui};
use git2::Repository;
use crate::backend::git_functions::git_fetch;
use crate::frontend::branch_tree::{BranchTreeNode, get_branch_trees};
use crate::frontend::commit_graph::CommitGraph;
use crate::frontend::modals::ErrorModal;

pub struct OG2Tab {
    pub(crate) name: String,
    repo: Arc<Mutex<Repository>>,
    error_modal: Arc<Mutex<ErrorModal>>,
    branch_trees: [BranchTreeNode; 3],
    branch_tree_col_width: f32,
    commit_graph: CommitGraph,
}

impl OG2Tab {
    pub fn new(name: String, repo: Repository, error_modal: Arc<Mutex<ErrorModal>>, ctx: &Context) -> Result<Self> {
        let branch_trees = get_branch_trees(&repo, ctx)?;
        let commit_graph = CommitGraph::new(&repo)?;
        Ok(Self {
            name,
            repo: Arc::new(Mutex::new(repo)),
            error_modal,
            branch_trees,
            branch_tree_col_width: 200.0,
            commit_graph,
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
                    let repo_c = self.repo.clone();
                    let error_modal_c = self.error_modal.clone();
                    thread::spawn(move || {
                        let res = git_fetch(&repo_c.lock().unwrap());
                        let opt = error_modal_c.lock().unwrap().handle_error(res);
                        if let Some(()) = opt {
                            // TODO: Need to refresh the graph here!
                        }
                    });
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
