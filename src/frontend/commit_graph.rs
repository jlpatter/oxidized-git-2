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
    circle_location: Rc<RefCell<LocationIndex>>,
    summary_location: LocationIndex,
    summary: String,
    lines: Vec<Line>,
}

impl GraphRow {
    pub fn new(circle_location: Rc<RefCell<LocationIndex>>, summary_location: LocationIndex, summary: String) -> Self {
        Self {
            circle_location,
            summary_location,
            summary,
            lines: vec![],
        }
    }

    pub fn show(&self, painter: &Painter, scroll_area_top_left: Pos2) {
        for line in &self.lines {
            line.show(painter, scroll_area_top_left);
        }
        let circle_location = self.circle_location.borrow();
        painter.circle_filled(
            circle_location.get_relative_pos2(scroll_area_top_left),
            CIRCLE_RADIUS,
            circle_location.get_color()
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
    start_location: Rc<RefCell<LocationIndex>>,
    end_location: Rc<RefCell<LocationIndex>>,
}

impl Line {
    pub fn new(start_location: Rc<RefCell<LocationIndex>>, end_location: Rc<RefCell<LocationIndex>>) -> Self {
        Self {
            start_location,
            end_location,
        }
    }

    pub fn show(&self, painter: &Painter, scroll_area_top_left: Pos2) {
        let start_location = self.start_location.borrow();
        let end_location = self.end_location.borrow();
        painter.line_segment(
            [start_location.get_relative_pos2(scroll_area_top_left), end_location.get_relative_pos2(scroll_area_top_left)],
            Stroke::new(LINE_STROKE_WIDTH, start_location.get_color())
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
        let mut graph_rows = vec![];
        let mut graph_row_commit_map: HashMap<Oid, Rc<RefCell<GraphRow>>> = HashMap::new();
        for (i, oid_result) in revwalk.enumerate() {
            let oid = oid_result?;
            let git_commit = repo.find_commit(oid)?;

            let summary = String::from(git_commit.summary().ok_or(Error::msg("Commit message has invalid UTF-8!"))?);
            let graph_row_rc = Rc::new(RefCell::new(GraphRow::new(
                Rc::new(RefCell::new(LocationIndex::new(0, i))),
                LocationIndex::new(1, i),
                summary,
            )));

            for parent_oid in git_commit.parent_ids() {
                if let Some(parent_row_rc) = graph_row_commit_map.get(&parent_oid) {
                    let parent_row = parent_row_rc.borrow();
                    // TODO: Connect the parents to the current row here!
                }
            }

            graph_row_commit_map.insert(oid, graph_row_rc.clone());
            graph_rows.push(graph_row_rc);
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