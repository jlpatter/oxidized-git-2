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

struct GraphRow {
    // NOTE: X and Y here are not pixel coordinates, they act more like indexes of valid 'positions'.
    oid: Oid,
    circle_location: LocationIndex,
    summary_location: LocationIndex,
    summary: String,
    // These are lines that start in this row.
    lines: Vec<Line>,
}

impl GraphRow {
    pub fn new(commit: git2::Commit, i: usize) -> Result<Self> {
        Ok(Self {
            oid: commit.id(),
            circle_location: LocationIndex::new(0, i),
            summary_location: LocationIndex::new(1, i),
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
        painter.text(
            self.summary_location.get_relative_pos2(scroll_area_top_left),
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
    graph_rows: Vec<Rc<RefCell<GraphRow>>>,
}

impl CommitGraph {
    pub fn new(repo: &Repository) -> Result<Self> {
        let graph_rows = CommitGraph::get_graph_rows(repo)?;
        Ok(Self {
            graph_rows,
        })
    }

    fn get_graph_rows(repo: &Repository) -> Result<Vec<Rc<RefCell<GraphRow>>>> {
        // Loop through once to get all the commits and create a mapping to get the parents later.
        let oid_vec = git_revwalk(repo)?;
        let mut graph_rows = vec![];
        // commit_map and commit_parent_oid_map are just used to get the parents within this fn.
        let mut commit_map: HashMap<Oid, Rc<RefCell<GraphRow>>> = HashMap::new();
        let mut commit_parent_oid_map: HashMap<Oid, Vec<Oid>> = HashMap::new();
        for (i, oid) in oid_vec.iter().enumerate() {
            let git_commit = repo.find_commit(*oid)?;
            commit_parent_oid_map.insert(*oid, git_commit.parents().map(|p| p.id()).collect());
            let graph_row_rc = Rc::new(RefCell::new(GraphRow::new(git_commit, i)?));
            commit_map.insert(*oid, graph_row_rc.clone());
            graph_rows.push(graph_row_rc);
        }

        // Now, loop through a second time to set the parent locations.
        let mut occupied_locations_table: Vec<Vec<usize>> = vec![];
        for graph_row_rc in &graph_rows {
            let mut graph_row = graph_row_rc.borrow_mut();

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
                    if let Some(parent_graph_row_rc) = commit_map.get(parent_oid) {
                        let mut parent_graph_row = parent_graph_row_rc.borrow_mut();
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
        for graph_row_rc in &graph_rows {
            let graph_row = graph_row_rc.borrow();
            // This is to set summary text positions next to curved lines.
            if let Some(parent_oids) = commit_parent_oid_map.get(&graph_row.oid) {
                for parent_oid in parent_oids {
                    if let Some(parent_commit_rc) = commit_map.get(parent_oid) {
                        let parent_graph_row = parent_commit_rc.borrow();
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
            let mut graph_row = graph_row_rc.borrow_mut();

            graph_row.summary_location.x = *occupied_locations_table[graph_row.circle_location.y].iter().max().unwrap_or(&0) + 1;

            if let Some(parent_oids) = commit_parent_oid_map.get(&graph_row.oid) {
                for parent_oid in parent_oids {
                    if let Some(parent_commit_rc) = commit_map.get(parent_oid) {
                        let parent_graph_row = parent_commit_rc.borrow();

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
                                    graph_rows[i].borrow_mut().lines.push(Line::new(line_x, i, line_x, i + 1));
                                }
                            }
                        }

                        if before_parent_y == child_y {
                            // This is so graph_row doesn't get borrowed twice.
                            graph_row.lines.push(Line::new(child_x, before_parent_y, parent_x, parent_y));
                        } else {
                            graph_rows[before_parent_y].borrow_mut().lines.push(Line::new(child_x, before_parent_y, parent_x, parent_y));
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
                    self.graph_rows[i].borrow().show(&painter, scroll_area_top_left);
                }
            });
        });
    }
}