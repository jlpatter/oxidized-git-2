use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use anyhow::{Error, Result};
use egui::{Align2, Color32, FontId, Painter, Pos2, ScrollArea, Sense, Stroke, Ui, Vec2};
use git2::{Oid, Repository};
use crate::backend::git_functions::git_revwalk;

const X_OFFSET: f32 = 10.0;
const X_SPACING: f32 = 15.0;
const Y_OFFSET: f32 = 10.0;
const Y_SPACING: f32 = 30.0;
const CIRCLE_RADIUS: f32 = 7.0;
const LINE_STROKE_WIDTH: f32 = 3.0;
const GRAPH_COLORS: [Color32; 4] = [Color32::BLUE, Color32::GREEN, Color32::YELLOW, Color32::RED];
const VISIBLE_SCROLL_AREA_PADDING: usize = 10;

struct LocationIndex {
    x: usize,
    y: usize,
}

impl LocationIndex {
    pub fn new (x: usize, y: usize) -> Self {
        Self {
            x,
            y,
        }
    }

    fn get_pixel_x(&self) -> f32 {
        X_OFFSET + X_SPACING * self.x as f32
    }

    fn get_pixel_y(&self) -> f32 {
        Y_OFFSET + Y_SPACING * self.y as f32
    }

    pub fn get_relative_pos2(&self, scroll_area_top_left: Pos2) -> Pos2 {
        scroll_area_top_left + Vec2::new(self.get_pixel_x(), self.get_pixel_y())
    }

    pub fn get_color(&self) -> Color32 {
        GRAPH_COLORS[self.x % GRAPH_COLORS.len()]
    }
}

struct Commit {
    // NOTE: X and Y here are not pixel coordinates, they act more like indexes of valid 'positions'.
    oid: Oid,
    location: LocationIndex,
    summary: String,
}

impl Commit {
    pub fn new(commit: git2::Commit, i: usize) -> Result<Self> {
        Ok(Self {
            oid: commit.id(),
            location: LocationIndex::new(0, i),
            summary: String::from(commit.summary().ok_or(Error::msg("Commit summary has invalid UTF-8!"))?),
        })
    }

    pub fn show(&self, painter: &Painter, scroll_area_top_left: Pos2) {
        let circle_position = self.location.get_relative_pos2(scroll_area_top_left);
        painter.circle_filled(circle_position, CIRCLE_RADIUS, self.location.get_color());
        painter.text(circle_position + Vec2::new(X_OFFSET, 0.0), Align2::LEFT_CENTER, self.summary.clone(), FontId::default(), Color32::WHITE);
    }
}

struct Line {
    start: LocationIndex,
    end: LocationIndex,
}

impl Line {
    pub fn new(start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> Self {
        Self {
            start: LocationIndex::new(start_x, start_y),
            end: LocationIndex::new(end_x, end_y),
        }
    }

    pub fn show(&self, painter: &Painter, scroll_area_top_left: Pos2) {
        painter.line_segment([self.start.get_relative_pos2(scroll_area_top_left), self.end.get_relative_pos2(scroll_area_top_left)], Stroke::new(LINE_STROKE_WIDTH, self.start.get_color()));
    }
}

pub struct CommitGraph {
    commits: Vec<Rc<RefCell<Commit>>>,
    lines: Vec<Line>,
}

impl CommitGraph {
    pub fn new(repo: &Repository) -> Result<Self> {
        let (commits, lines) = CommitGraph::get_commits_and_lines(repo)?;
        Ok(Self {
            commits,
            lines,
        })
    }

    fn get_commits_and_lines(repo: &Repository) -> Result<(Vec<Rc<RefCell<Commit>>>, Vec<Line>)> {
        // Loop through once to get all the commits and create a mapping to get the parents later.
        let oid_vec = git_revwalk(repo)?;
        let mut commits = vec![];
        // commit_map and commit_parent_oid_map are just used to get the parents within this fn.
        let mut commit_map: HashMap<Oid, Rc<RefCell<Commit>>> = HashMap::new();
        let mut commit_parent_oid_map: HashMap<Oid, Vec<Oid>> = HashMap::new();
        for (i, oid) in oid_vec.iter().enumerate() {
            let git_commit = repo.find_commit(*oid)?;
            commit_parent_oid_map.insert(*oid, git_commit.parents().map(|p| p.id()).collect());
            let commit_rc = Rc::new(RefCell::new(Commit::new(git_commit, i)?));
            commit_map.insert(*oid, commit_rc.clone());
            commits.push(commit_rc);
        }

        // Now, loop through a second time to set the parent locations and lines.
        let mut occupied_locations_table: Vec<Vec<usize>> = vec![];
        for commit_rc in &commits {
            let mut commit = commit_rc.borrow_mut();

            // Set the current node position as occupied (or find a position that's unoccupied and occupy it).
            if commit.location.y < occupied_locations_table.len() {
                while occupied_locations_table[commit.location.y].contains(&commit.location.x) {
                    commit.location.x += 1;
                }
                occupied_locations_table[commit.location.y].push(commit.location.x);
            } else {
                occupied_locations_table.push(vec![commit.location.x]);
            }

            if let Some(parent_oids) = commit_parent_oid_map.get(&commit.oid) {
                for parent_oid in parent_oids {
                    // Set the space of the line from the current node to its parents as occupied.
                    if let Some(parent_commit_rc) = commit_map.get(parent_oid) {
                        let mut parent_commit = parent_commit_rc.borrow_mut();
                        let mut moved_x_val = 0;
                        for i in (commit.location.y + 1)..parent_commit.location.y {
                            let mut x_val = commit.location.x;
                            if i < occupied_locations_table.len() {
                                while occupied_locations_table[i].contains(&x_val) {
                                    x_val += 1;
                                    // Note: this has to stay in the loop so it's only set when x changes!
                                    // and not just to svg_row.x
                                    moved_x_val = x_val;
                                }
                                occupied_locations_table[i].push(x_val);
                            } else {
                                occupied_locations_table.push(vec![x_val]);
                            }
                        }
                        // This is used particularly for merging lines
                        parent_commit.location.x = moved_x_val;
                    }
                }
            }
        }
        Ok((commits, vec![]))
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let visible_area_top = ui.min_rect().min.y;
        let visible_area_height = ui.min_rect().max.y - visible_area_top;
        ScrollArea::both().id_source("graph-scroll-area").auto_shrink([false, false]).show(ui, |ui| {
            // This ui.vertical is just to keep the contents at the top of the scroll area if they're
            // smaller than it.
            ui.vertical(|ui| {
                let graph_height = self.commits.len() as f32 * Y_SPACING;
                let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), graph_height), Sense::hover());
                let scroll_area_top_left = response.rect.left_top();

                let scroll_position = visible_area_top - scroll_area_top_left.y;
                let visible_area_top_index = (((scroll_position - Y_OFFSET) / Y_SPACING) as isize - VISIBLE_SCROLL_AREA_PADDING as isize).max(0) as usize;
                let visible_area_bottom_index = (((scroll_position + visible_area_height - Y_OFFSET) / Y_SPACING) as usize + VISIBLE_SCROLL_AREA_PADDING).min(self.commits.len());

                // for i in visible_area_top_index..visible_area_bottom_index {
                //     // TODO: This isn't going to work since lines may not be in vertical order :'(
                //     self.lines[i].show(&painter, scroll_area_top_left);
                // }
                for i in visible_area_top_index..visible_area_bottom_index {
                    self.commits[i].borrow().show(&painter, scroll_area_top_left);
                }
            });
        });
    }
}