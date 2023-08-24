use anyhow::Result;
use egui::{Align2, Color32, FontId, ScrollArea, Sense, Ui, Vec2};
use git2::Repository;
use crate::backend::git_functions::git_revwalk;

const X_OFFSET: f32 = 10.0;
const Y_OFFSET: f32 = 10.0;
const Y_SPACING: f32 = 30.0;
const CIRCLE_RADIUS: f32 = 7.0;
// const LINE_STROKE_WIDTH: f32 = 3.0;
const GRAPH_COLORS: [Color32; 4] = [Color32::BLUE, Color32::GREEN, Color32::YELLOW, Color32::RED];

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
        ScrollArea::both().id_source("graph-scroll-area").auto_shrink([false, false]).show(ui, |ui| {
            ui.vertical(|ui| {
                let scroll_area_height = self.commit_summaries.len() as f32 * Y_SPACING;
                let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), scroll_area_height), Sense::hover());
                let start_position = response.rect.left_top();

                for (i, summary) in self.commit_summaries.iter().enumerate() {
                    let circle_position = start_position + Vec2::new(X_OFFSET, Y_OFFSET + Y_SPACING * i as f32);
                    painter.circle_filled(circle_position, CIRCLE_RADIUS, GRAPH_COLORS[0]);
                    painter.text(circle_position + Vec2::new(X_OFFSET, 0.0), Align2::LEFT_CENTER, summary, FontId::default(), Color32::WHITE);
                }
            });
        });
    }
}