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

#[derive(Copy, Clone)]
struct LocationIndex {
    // NOTE: X and Y here are not pixel coordinates, they act more like indexes of valid 'positions'.
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
    circle_location: LocationIndex,
    summary_location: LocationIndex,
    summary: String,
    lines: Vec<Line>,
}

impl GraphRow {
    pub fn new(circle_location: LocationIndex, summary_location: LocationIndex, summary: String) -> Self {
        Self {
            circle_location,
            summary_location,
            summary,
            lines: vec![],
        }
    }

    pub fn shift_elements_right_one(&mut self) {
        self.circle_location.x += 1;
        self.summary_location.x += 1;
        for line in &mut self.lines {
            line.start_location.x += 1;
            line.end_location.x += 1;
        }
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
    start_location: LocationIndex,
    end_location: LocationIndex,
}

impl Line {
    pub fn new(start_location: LocationIndex, end_location: LocationIndex) -> Self {
        Self {
            start_location,
            end_location,
        }
    }

    pub fn show(&self, painter: &Painter, scroll_area_top_left: Pos2) {
        painter.line_segment(
            [self.start_location.get_relative_pos2(scroll_area_top_left), self.end_location.get_relative_pos2(scroll_area_top_left)],
            Stroke::new(LINE_STROKE_WIDTH, self.start_location.get_color())
        );
    }
}

pub struct CommitGraph {
    graph_rows: Vec<Rc<RefCell<GraphRow>>>,
}

impl CommitGraph {
    pub fn new(repo: &Repository) -> Result<Self> {
        let graph_rows = CommitGraph::get_commits_and_lines(repo)?;
        Ok(Self {
            graph_rows,
        })
    }

    fn get_commits_and_lines(repo: &Repository) -> Result<Vec<Rc<RefCell<GraphRow>>>> {
        // Loop through once to get all the commits and create a mapping to get the parents later.
        let revwalk = git_revwalk(repo)?;
        let mut graph_rows: Vec<Rc<RefCell<GraphRow>>> = vec![];
        let mut graph_row_commit_map: HashMap<Oid, Rc<RefCell<GraphRow>>> = HashMap::new();
        for (i, oid_result) in revwalk.enumerate() {
            let oid = oid_result?;
            let git_commit = repo.find_commit(oid)?;

            let summary = String::from(git_commit.summary().ok_or(Error::msg("Commit message has invalid UTF-8!"))?);
            let graph_row = GraphRow::new(
                LocationIndex::new(0, i),
                LocationIndex::new(1, i),
                summary,
            );

            let parent_count = git_commit.parent_count();
            // TODO: If the parent_count > 1, then it should swap to the 2 branches from the merge commit!
            for parent_oid in git_commit.parent_ids() {
                if let Some(parent_row_rc) = graph_row_commit_map.get(&parent_oid) {
                    let parent_x;
                    let parent_y;
                    {
                        // parent_row is put in a closure here so it doesn't get borrowed twice.
                        let parent_row = parent_row_rc.borrow();
                        parent_x = parent_row.circle_location.x;
                        parent_y = parent_row.circle_location.y;
                    }

                    let after_parent_y = parent_y + 1;
                    let old_child_x = graph_row.circle_location.x;
                    let child_y = graph_row.circle_location.y;
                    // If there are multiple rows between the parent and child.
                    if after_parent_y < child_y {

                        // Use a separate closure to update the parent_row line so it doesn't get
                        // borrowed twice with graph_rows[j - 1].
                        if parent_count == 1 {
                            let mut parent_row = parent_row_rc.borrow_mut();
                            // Add branching line to previous commits that will be moved over.
                            let line_index = parent_row.lines.iter().position(|line| line.end_location.x == old_child_x).unwrap();
                            parent_row.lines[line_index].end_location.x += 1;
                        }

                        for j in after_parent_y..=child_y {
                            let mut before_row = graph_rows[j - 1].borrow_mut();
                            let current_row_location_y;
                            if j < graph_rows.len() {
                                let mut next_row = graph_rows[j].borrow_mut();
                                if parent_count == 1 {
                                    next_row.shift_elements_right_one();
                                }
                                current_row_location_y = next_row.circle_location.y;
                            } else {
                                current_row_location_y = graph_row.circle_location.y;
                            }

                            // Create the new straight line to the existing parent.
                            let end_line_x;
                            if j == child_y {
                                end_line_x = graph_row.circle_location.x;
                            } else {
                                end_line_x = parent_x;
                            }
                            let line = Line::new(
                                LocationIndex::new(parent_x, before_row.circle_location.y),
                                LocationIndex::new(end_line_x, current_row_location_y)
                            );
                            before_row.lines.push(line);
                        }
                    } else {
                        let mut parent_row = parent_row_rc.borrow_mut();
                        let line = Line::new(parent_row.circle_location, graph_row.circle_location);
                        // Shifting other elements does not occur here because this line is on top
                        // of a circle.
                        parent_row.lines.push(line);
                    }
                }
            }

            let graph_row_rc = Rc::new(RefCell::new(graph_row));
            graph_row_commit_map.insert(oid, graph_row_rc.clone());
            graph_rows.push(graph_row_rc);
        }

        // TODO: Reverse the graph_rows and set their y's to the correct (inverse) positions!
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