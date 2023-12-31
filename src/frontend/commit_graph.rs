use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{Error, Result};
use egui::{Align2, Color32, FontId, Painter, Pos2, Rounding, ScrollArea, Sense, Stroke, Ui, Vec2};
use git2::{BranchType, Oid, Repository};
use crate::backend::git_functions::git_revwalk;

const X_OFFSET: f32 = 10.0;
const X_SPACING: f32 = 15.0;
const Y_OFFSET: f32 = 10.0;
const Y_SPACING: f32 = 30.0;
const REF_X_SPACING: f32 = 5.0;
const REF_RECT_ROUNDING: f32 = 5.0;
const REF_RECT_MARGIN: Vec2 = Vec2::new(3.0, 1.0);
const CIRCLE_RADIUS: f32 = 7.0;
const LINE_STROKE_WIDTH: f32 = 3.0;
const GRAPH_COLORS: [Color32; 4] = [Color32::BLUE, Color32::GREEN, Color32::YELLOW, Color32::RED];
const LOCAL_BRANCH_COLOR: Color32 = Color32::from_rgb(200, 0, 0);
const REMOTE_BRANCH_COLOR: Color32 = Color32::from_rgb(0, 139, 0);
const TAG_COLOR: Color32 = Color32::from_rgb(160, 160, 160);
const REF_GAMMA_MULTIPLIER: f32 = 0.3;  // Set higher to make more opaque.
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

#[derive(Clone)]
struct GraphRowRef {
    color: Color32,
    shorthand: String,
    is_head: bool,
}

impl GraphRowRef {
    fn new(color: Color32, shorthand: String, is_head: bool) -> Self {
        Self {
            color,
            shorthand,
            is_head,
        }
    }

    pub fn get_commit_branch_map(repo: &Repository) -> Result<HashMap<Oid, Vec<GraphRowRef>>> {
        let mut commit_branch_map: HashMap<Oid, Vec<GraphRowRef>> = HashMap::new();
        for ref_result in repo.references()? {
            let reference = ref_result?;
            let branch_shorthand = reference.shorthand().ok_or(Error::msg("Branch Shorthand has invalid UTF-8!"))?;

            let mut is_head = false;
            let color;
            if reference.is_branch() {
                color = LOCAL_BRANCH_COLOR.gamma_multiply(REF_GAMMA_MULTIPLIER);
                if repo.find_branch(branch_shorthand, BranchType::Local)?.is_head() {
                    is_head = true;
                }
            } else if reference.is_remote() && !branch_shorthand.ends_with("/HEAD") {
                color = REMOTE_BRANCH_COLOR.gamma_multiply(REF_GAMMA_MULTIPLIER);
            } else if reference.is_tag() {
                color = TAG_COLOR.gamma_multiply(REF_GAMMA_MULTIPLIER);
            } else {
                continue;
            }

            let target_oid = reference.peel_to_commit()?.id();
            let graph_row_ref = GraphRowRef::new(color, String::from(branch_shorthand), is_head);
            match commit_branch_map.get_mut(&target_oid) {
                Some(v) => v.push(graph_row_ref),
                None => {
                    commit_branch_map.insert(target_oid, vec![graph_row_ref]);
                },
            };
        }
        Ok(commit_branch_map)
    }

    pub fn show(&self, painter: &Painter, next_text_position: Pos2) -> Pos2 {
        let text;
        if self.is_head {
            text = format!("* {}", self.shorthand);
        } else {
            text = self.shorthand.clone();
        }
        let ref_rect = painter.text(
            next_text_position,
            Align2::LEFT_CENTER,
            text,
            FontId::default(),
            Color32::WHITE
        ).expand2(REF_RECT_MARGIN);
        painter.rect_filled(ref_rect, Rounding::same(REF_RECT_ROUNDING), self.color);
        // Return the next text position.
        ref_rect.right_center() + Vec2::new(REF_X_SPACING, 0.0)
    }
}

struct GraphRow {
    oid: Oid,
    circle_location: LocationIndex,
    summary_location: LocationIndex,
    refs: Vec<GraphRowRef>,
    summary: String,
    // These are lines that start in this row.
    lines: Vec<Line>,
}

impl GraphRow {
    pub fn new(commit: git2::Commit, refs: Vec<GraphRowRef>, i: usize) -> Result<Self> {
        Ok(Self {
            oid: commit.id(),
            circle_location: LocationIndex::new(0, i),
            summary_location: LocationIndex::new(1, i),
            refs,
            summary: String::from(commit.summary().ok_or(Error::msg("Commit summary has invalid UTF-8!"))?),
            lines: vec![],
        })
    }

    pub fn show(&self, painter: &Painter, scroll_area_top_left: Pos2) {
        for line in &self.lines {
            line.show(painter, scroll_area_top_left);
        }
        painter.circle_filled(
            self.circle_location.get_relative_pos2(scroll_area_top_left),
            CIRCLE_RADIUS,
            self.circle_location.get_color()
        );
        let mut next_text_position = self.summary_location.get_relative_pos2(scroll_area_top_left);
        for commit_ref in &self.refs {
            next_text_position = commit_ref.show(painter, next_text_position);
        }
        painter.text(
            next_text_position,
            Align2::LEFT_CENTER,
            self.summary.clone(),
            FontId::default(),
            Color32::WHITE
        );
    }
}

struct Line {
    start: LocationIndex,
    end: LocationIndex,
    color: Color32,
}

impl Line {
    pub fn new(start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> Self {
        let start = LocationIndex::new(start_x, start_y);
        let end = LocationIndex::new(end_x, end_y);
        let color;
        if start_x < end_x {
            color = end.get_color();
        } else {
            color = start.get_color();
        }
        Self {
            start,
            end,
            color,
        }
    }

    pub fn show(&self, painter: &Painter, scroll_area_top_left: Pos2) {
        painter.line_segment([self.start.get_relative_pos2(scroll_area_top_left), self.end.get_relative_pos2(scroll_area_top_left)], Stroke::new(LINE_STROKE_WIDTH, self.color));
    }
}

pub struct CommitGraph {
    graph_rows: Vec<Arc<Mutex<GraphRow>>>,
}

impl CommitGraph {
    pub fn new(repo: &Repository) -> Result<Self> {
        let graph_rows = CommitGraph::get_graph_rows(repo)?;
        Ok(Self {
            graph_rows,
        })
    }

    pub fn refresh_graph(&mut self, repo: &Repository) -> Result<()> {
        self.graph_rows = CommitGraph::get_graph_rows(repo)?;
        Ok(())
    }

    fn get_graph_rows(repo: &Repository) -> Result<Vec<Arc<Mutex<GraphRow>>>> {
        let oid_vec = git_revwalk(repo)?;
        let commit_branch_map = GraphRowRef::get_commit_branch_map(repo)?;
        let mut graph_rows = vec![];
        // commit_map and commit_parent_oid_map are just used to get the parents within this fn.
        let mut commit_map: HashMap<Oid, Arc<Mutex<GraphRow>>> = HashMap::new();
        let mut commit_parent_oid_map: HashMap<Oid, Vec<Oid>> = HashMap::new();

        // Loop through once to get all the commits and create a mapping to get the parents later.
        for (i, oid) in oid_vec.iter().enumerate() {
            let git_commit = repo.find_commit(*oid)?;
            commit_parent_oid_map.insert(*oid, git_commit.parents().map(|p| p.id()).collect());

            let mut graph_row_refs = vec![];
            if let Some(refs) = commit_branch_map.get(&git_commit.id()) {
                graph_row_refs = refs.clone();
            }

            let graph_row_arc = Arc::new(Mutex::new(GraphRow::new(git_commit, graph_row_refs, i)?));
            commit_map.insert(*oid, graph_row_arc.clone());
            graph_rows.push(graph_row_arc);
        }

        // Now, loop through a second time to set the parent locations.
        let mut occupied_locations_table: Vec<Vec<usize>> = vec![];
        for graph_row_arc in &graph_rows {
            let mut graph_row = graph_row_arc.lock().unwrap();

            // Set the current node position as occupied (or find a position that's unoccupied and occupy it).
            if graph_row.circle_location.y < occupied_locations_table.len() {
                while occupied_locations_table[graph_row.circle_location.y].contains(&graph_row.circle_location.x) {
                    graph_row.circle_location.x += 1;
                }
                occupied_locations_table[graph_row.circle_location.y].push(graph_row.circle_location.x);
            } else {
                occupied_locations_table.push(vec![graph_row.circle_location.x]);
            }

            if let Some(parent_oids) = commit_parent_oid_map.get(&graph_row.oid) {
                for parent_oid in parent_oids {
                    // Set the space of the line from the current node to its parents as occupied.
                    if let Some(parent_graph_row_arc) = commit_map.get(parent_oid) {
                        let mut parent_graph_row = parent_graph_row_arc.lock().unwrap();
                        let mut moved_x_val = 0;
                        for i in (graph_row.circle_location.y + 1)..parent_graph_row.circle_location.y {
                            let mut x_val = graph_row.circle_location.x;
                            if i < occupied_locations_table.len() {
                                while occupied_locations_table[i].contains(&x_val) {
                                    x_val += 1;
                                    // Note: this has to stay in the loop so it's only set when x changes!
                                    // and not just to graph_row.circle_location.x
                                    moved_x_val = x_val;
                                }
                                occupied_locations_table[i].push(x_val);
                            } else {
                                occupied_locations_table.push(vec![x_val]);
                            }
                        }
                        // This is used particularly for merging lines
                        parent_graph_row.circle_location.x = moved_x_val;
                    }
                }
            }
        }

        // Loop through after everything's set in order to properly occupy spaces by curved lines just for summary text positions.
        for graph_row_arc in &graph_rows {
            let graph_row = graph_row_arc.lock().unwrap();
            // This is to set summary text positions next to curved lines.
            if let Some(parent_oids) = commit_parent_oid_map.get(&graph_row.oid) {
                for parent_oid in parent_oids {
                    if let Some(parent_commit_arc) = commit_map.get(parent_oid) {
                        let parent_graph_row = parent_commit_arc.lock().unwrap();
                        if graph_row.circle_location.x < parent_graph_row.circle_location.x {
                            let x_val = parent_graph_row.circle_location.x;
                            occupied_locations_table[graph_row.circle_location.y].push(x_val);
                        } else if graph_row.circle_location.x > parent_graph_row.circle_location.x {
                            let x_val = graph_row.circle_location.x;
                            occupied_locations_table[parent_graph_row.circle_location.y].push(x_val);
                        }
                    }
                }
            }
        }

        // Loop through a final time to add lines and set summary text positions.
        for graph_row_rc in &graph_rows {
            let mut graph_row = graph_row_rc.lock().unwrap();

            graph_row.summary_location.x = *occupied_locations_table[graph_row.circle_location.y].iter().max().unwrap_or(&0) + 1;

            if let Some(parent_oids) = commit_parent_oid_map.get(&graph_row.oid) {
                for parent_oid in parent_oids {
                    if let Some(parent_commit_arc) = commit_map.get(parent_oid) {
                        let parent_graph_row = parent_commit_arc.lock().unwrap();

                        let child_x = graph_row.circle_location.x;
                        let child_y = graph_row.circle_location.y;
                        let parent_x = parent_graph_row.circle_location.x;
                        let parent_y = parent_graph_row.circle_location.y;
                        let before_parent_y = parent_graph_row.circle_location.y - 1;
                        if before_parent_y != child_y {
                            let start_index;
                            let end_index;
                            let line_x;
                            if parent_x > child_x {
                                line_x = parent_x;
                                start_index = child_y + 1;
                                end_index = before_parent_y;
                            } else {
                                line_x = child_x;
                                start_index = child_y;
                                end_index = before_parent_y - 1;
                            }
                            for i in start_index..=end_index {
                                if i == child_y {
                                    // This is so graph_row doesn't get borrowed twice.
                                    graph_row.lines.push(Line::new(line_x, i, line_x, i + 1));
                                } else {
                                    graph_rows[i].lock().unwrap().lines.push(Line::new(line_x, i, line_x, i + 1));
                                }
                            }
                        }

                        if before_parent_y == child_y {
                            // This is so graph_row doesn't get borrowed twice.
                            graph_row.lines.push(Line::new(child_x, before_parent_y, parent_x, parent_y));
                        } else {
                            graph_rows[before_parent_y].lock().unwrap().lines.push(Line::new(child_x, before_parent_y, parent_x, parent_y));
                        }
                    }
                }
            }
        }
        Ok(graph_rows)
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let visible_area_top = ui.min_rect().min.y;
        let visible_area_height = ui.min_rect().max.y - visible_area_top;
        ScrollArea::both().id_source("graph-scroll-area").auto_shrink([false, false]).show(ui, |ui| {
            // This ui.vertical is just to keep the contents at the top of the scroll area if they're
            // smaller than it.
            ui.vertical(|ui| {
                let graph_height = self.graph_rows.len() as f32 * Y_SPACING;
                let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), graph_height), Sense::hover());
                let scroll_area_top_left = response.rect.left_top();

                let scroll_position = visible_area_top - scroll_area_top_left.y;
                let visible_area_top_index = (((scroll_position - Y_OFFSET) / Y_SPACING) as isize - VISIBLE_SCROLL_AREA_PADDING as isize).max(0) as usize;
                let visible_area_bottom_index = (((scroll_position + visible_area_height - Y_OFFSET) / Y_SPACING) as usize + VISIBLE_SCROLL_AREA_PADDING).min(self.graph_rows.len());

                for i in visible_area_top_index..visible_area_bottom_index {
                    self.graph_rows[i].lock().unwrap().show(&painter, scroll_area_top_left);
                }
            });
        });
    }
}