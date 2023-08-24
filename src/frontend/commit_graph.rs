use anyhow::Result;
use egui::{Color32, Stroke, Ui};
use git2::Repository;
use crate::backend::git_functions::git_revwalk;

pub struct CommitGraph {
    commit_summaries: Vec<String>,
}

impl CommitGraph {
    pub fn new(repo: &Repository) -> Result<Self> {
        let oid_vec = git_revwalk(repo)?;
        let mut commit_summaries = vec![];
        for oid in oid_vec {
            // TODO: Handle non-valid UTF-8!
            commit_summaries.push(String::from(repo.find_commit(oid)?.summary().unwrap()));
        }
        Ok(Self {
            commit_summaries,
        })
    }

    pub fn show(&mut self, ui: &mut Ui) {
        // This is an example of how the graph could be rendered.
        let start_position = ui.cursor().left_top();
        let painter = ui.painter();

        // TODO: Iterate over commit summaries and draw graph!
        painter.line_segment([start_position + egui::vec2(10.0, 10.0), start_position + egui::vec2(10.0, 40.0)], Stroke::new(3.0, Color32::RED));
        painter.circle_filled(start_position + egui::vec2(10.0, 10.0), 7.0, Color32::RED);
        painter.circle_filled(start_position + egui::vec2(10.0, 40.0), 7.0, Color32::RED);
    }
}