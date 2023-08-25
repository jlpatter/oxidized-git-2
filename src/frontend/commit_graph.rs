use anyhow::{Error, Result};
use egui::{Align2, Color32, FontId, Painter, Pos2, ScrollArea, Sense, Ui, Vec2};
use git2::{Oid, Repository};
use crate::backend::git_functions::git_revwalk;

const X_OFFSET: f32 = 10.0;
const Y_OFFSET: f32 = 10.0;
const Y_SPACING: f32 = 30.0;
const CIRCLE_RADIUS: f32 = 7.0;
// const LINE_STROKE_WIDTH: f32 = 3.0;
const GRAPH_COLORS: [Color32; 4] = [Color32::BLUE, Color32::GREEN, Color32::YELLOW, Color32::RED];

struct Commit {
    summary: String,
}

impl Commit {
    pub fn new(repo: &Repository, oid: Oid) -> Result<Self> {
        let commit = repo.find_commit(oid)?;
        Ok(Self {
            summary: String::from(commit.summary().ok_or(Error::msg("Commit summary has invalid UTF-8!"))?),
        })
    }

    pub fn show(&self, painter: &Painter, index: usize, start_position: Pos2) {
        let circle_position = start_position + Vec2::new(X_OFFSET, Y_OFFSET + Y_SPACING * index as f32);
        painter.circle_filled(circle_position, CIRCLE_RADIUS, GRAPH_COLORS[0]);
        painter.text(circle_position + Vec2::new(X_OFFSET, 0.0), Align2::LEFT_CENTER, self.summary.clone(), FontId::default(), Color32::WHITE);
    }
}

pub struct CommitGraph {
    commits: Vec<Commit>,
}

impl CommitGraph {
    pub fn new(repo: &Repository) -> Result<Self> {
        let oid_vec = git_revwalk(repo)?;
        let mut commits = vec![];
        for oid in oid_vec {
            commits.push(Commit::new(repo, oid)?);
        }
        Ok(Self {
            commits,
        })
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ScrollArea::both().id_source("graph-scroll-area").auto_shrink([false, false]).show(ui, |ui| {
            ui.vertical(|ui| {
                let scroll_area_height = self.commits.len() as f32 * Y_SPACING;
                let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), scroll_area_height), Sense::hover());
                let start_position = response.rect.left_top();

                for (i, commit) in self.commits.iter().enumerate() {
                    commit.show(&painter, i, start_position);
                }
            });
        });
    }
}