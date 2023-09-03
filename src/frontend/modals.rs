use anyhow::Result;
use egui::{Align, Align2, Area, Button, Color32, Frame, Layout, Stroke, Ui, Vec2};
use crate::frontend::tab::OG2Tab;
use crate::frontend::utils;

const MODAL_Y_OFFSET: f32 = 100.0;
const MODAL_BORDER_WIDTH: f32 = 2.0;

pub trait Modal {
    fn open(&mut self);
    fn close(&mut self);
    fn show_in_modal<R>(&mut self, modal_id: String, ui: &mut Ui, modal_contents: impl FnOnce(&mut Self, &mut Ui) -> R) -> R {
        Area::new(modal_id).anchor(Align2::CENTER_TOP, Vec2::new(0.0, MODAL_Y_OFFSET)).show(ui.ctx(), |ui| {
            Frame::popup(&*ui.ctx().style()).fill(Color32::BLACK).stroke(Stroke::new(MODAL_BORDER_WIDTH, Color32::WHITE)).show(ui, |ui| {
                ui.allocate_ui_with_layout(ui.max_rect().size() / 3.0, Layout::top_down(Align::Center).with_main_justify(true).with_cross_justify(true), |ui| {
                    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                        if ui.add(Button::new("X").fill(Color32::RED)).clicked() {
                            self.close();
                        }
                    });
                    modal_contents(self, ui)
                }).inner
            }).inner
        }).inner
    }
}

#[derive(Default)]
pub struct ErrorModal {
    is_open: bool,
    error_msg: String,
}

impl Modal for ErrorModal {
    fn open(&mut self) {
        self.is_open = true;
    }

    fn close(&mut self) {
        self.is_open = false;
    }
}

impl ErrorModal {
    pub fn set_error_msg(&mut self, msg: String) {
        self.error_msg = msg;
    }

    pub fn handle_error<T>(&mut self, result: Result<T>) -> Option<T> {
        match result {
            Ok(t) => Some(t),
            Err(e) => {
                self.set_error_msg(e.to_string());
                self.open();
                None
            },
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        if self.is_open {
            self.show_in_modal(String::from("error-modal"), ui, |other_self, ui| {
                ui.label(other_self.error_msg.clone());
            });
        }
    }
}

#[derive(Default)]
pub struct AddTabModal {
    is_open: bool,
}

impl Modal for AddTabModal {
    fn open(&mut self) {
        self.is_open = true;
    }

    fn close(&mut self) {
        self.is_open = false;
    }
}

impl AddTabModal {
    pub fn show(&mut self, ui: &mut Ui, tabs: &mut Vec<OG2Tab>, active_tab: &mut usize) -> Result<()> {
        if self.is_open {
            return self.show_in_modal(String::from("add-tab-modal"), ui, |inner_self, ui| -> Result<()> {
                ui.label("To open a new tab, please initialize, open, or clone another repository.");
                ui.horizontal(|ui| -> Result<()> {
                    if ui.button("Init").clicked() {
                        // TODO: Implement Init.
                        inner_self.close();
                    }
                    if ui.button("Open").clicked() {
                        utils::open_repo_as_tab(tabs, active_tab, ui.ctx())?;
                        inner_self.close();
                    }
                    if ui.button("Clone").clicked() {
                        // TODO: Implement Clone.
                        inner_self.close();
                    }
                    Ok(())
                }).inner
            });
        }
        Ok(())
    }
}
