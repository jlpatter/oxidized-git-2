use anyhow::{Error, Result};
use egui::{Align2, Color32, FontId, Painter, Pos2, ScrollArea, Sense, Ui, Vec2};
use git2::{Oid, Repository};
use crate::backend::git_functions::git_revwalk;

const X_OFFSET: f32 = 10.0;
const X_SPACING: f32 = 15.0;
const Y_OFFSET: f32 = 10.0;
const Y_SPACING: f32 = 30.0;
const CIRCLE_RADIUS: f32 = 7.0;
// const LINE_STROKE_WIDTH: f32 = 3.0;
const GRAPH_COLORS: [Color32; 4] = [Color32::BLUE, Color32::GREEN, Color32::YELLOW, Color32::RED];

struct Commit {
    // NOTE: X and Y here are not pixel coordinates, they act more like indexes of valid 'positions'.
    x: usize,
    y: usize,
    summary: String,
}

impl Commit {
    pub fn new(repo: &Repository, i: usize, oid: Oid) -> Result<Self> {
        let commit = repo.find_commit(oid)?;
        Ok(Self {
            x: 0,
            y: i,
            summary: String::from(commit.summary().ok_or(Error::msg("Commit summary has invalid UTF-8!"))?),
        })
    }

    pub fn show(&self, painter: &Painter, start_position: Pos2) {
        let circle_position = start_position + Vec2::new(X_OFFSET + X_SPACING * self.x as f32, Y_OFFSET + Y_SPACING * self.y as f32);
        painter.circle_filled(circle_position, CIRCLE_RADIUS, GRAPH_COLORS[0]);
        painter.text(circle_position + Vec2::new(X_OFFSET, 0.0), Align2::LEFT_CENTER, self.summary.clone(), FontId::default(), Color32::WHITE);
    }
}

pub struct CommitGraph {
    commits: Vec<Commit>,
}

impl CommitGraph {
    pub fn new(repo: &Repository) -> Result<Self> {
        Ok(Self {
            commits: CommitGraph::get_commits(repo)?,
        })
    }

    fn get_commits(repo: &Repository) -> Result<Vec<Commit>> {
        let oid_vec = git_revwalk(repo)?;
        let mut commits = vec![];
        for (i, oid) in oid_vec.iter().enumerate() {
            commits.push(Commit::new(repo, i, *oid)?);
        }
        Ok(commits)
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ScrollArea::both().id_source("graph-scroll-area").auto_shrink([false, false]).show(ui, |ui| {
            ui.vertical(|ui| {
                let scroll_area_height = self.commits.len() as f32 * Y_SPACING;
                let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), scroll_area_height), Sense::hover());
                let start_position = response.rect.left_top();

                for commit in &self.commits {
                    commit.show(&painter, start_position);
                }
            });
        });
    }
}