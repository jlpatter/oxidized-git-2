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

enum GraphElement {
    Circle,
    Summary,
    Line(Line),
}

struct GraphRow {
    // NOTE: X and Y here are not pixel coordinates, they act more like indexes of valid 'positions'.
    oid: Oid,
    summary: String,
    elements: Vec<GraphElement>,
}

impl GraphRow {
    pub fn new(commit: git2::Commit, i: usize) -> Result<Self> {
        Ok(Self {
            oid: commit.id(),
            summary: String::from(commit.summary().ok_or(Error::msg("Commit summary has invalid UTF-8!"))?),
            elements: vec![],
        })
    }

    fn get_pixel_x(x: usize) -> f32 {
        X_OFFSET + X_SPACING * x as f32
    }

    fn get_pixel_y(y: usize) -> f32 {
        Y_OFFSET + Y_SPACING * y as f32
    }

    fn get_relative_pos2(x: usize, y: usize, scroll_area_top_left: Pos2) -> Pos2 {
        scroll_area_top_left + Vec2::new(GraphRow::get_pixel_x(x), GraphRow::get_pixel_y(y))
    }

    fn get_color(x: usize) -> Color32 {
        GRAPH_COLORS[x % GRAPH_COLORS.len()]
    }

    pub fn show(&self, painter: &Painter, y: usize, scroll_area_top_left: Pos2) {
        for (x, element) in self.elements.iter().enumerate() {
            match element {
                GraphElement::Circle => {
                    painter.circle_filled(
                        GraphRow::get_relative_pos2(x, y, scroll_area_top_left),
                        CIRCLE_RADIUS,
                        GraphRow::get_color(x)
                    );
                }
                GraphElement::Summary => {
                    painter.text(
                        GraphRow::get_relative_pos2(x, y, scroll_area_top_left),
                        Align2::LEFT_CENTER,
                        self.summary.clone(),
                        FontId::default(),
                        Color32::WHITE
                    );
                }
                GraphElement::Line(line) => {
                    line.show(painter, scroll_area_top_left);
                }
            };
        }
    }
}

struct Line {
    end_element: Rc<RefCell<GraphElement>>,
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
        painter.line_segment(
            [self.start.get_relative_pos2(scroll_area_top_left), self.end.get_relative_pos2(scroll_area_top_left)],
            Stroke::new(LINE_STROKE_WIDTH, self.color)
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
        for (i, oid_result) in revwalk.enumerate() {
            let oid = oid_result?;
            let git_commit = repo.find_commit(oid)?;
            let graph_row_rc = Rc::new(RefCell::new(GraphRow::new(git_commit, i)?));
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
                    self.graph_rows[i].borrow().show(&painter, i, scroll_area_top_left);
                }
            });
        });
    }
}