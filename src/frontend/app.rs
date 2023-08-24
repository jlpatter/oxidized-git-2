use anyhow::Result;
use eframe::Frame;
use egui::{Button, Context, SelectableLabel, Ui};
use crate::frontend::modals::{AddTabModal, ErrorModal, Modal};
use crate::frontend::tab::OG2Tab;
use crate::frontend::utils;

const TAB_HEIGHT: f32 = 20.0;
const TAB_ADD_BTN_WIDTH: f32 = 20.0;

#[derive(Default)]
pub struct OG2App {
    tabs: Vec<OG2Tab>,
    active_tab: usize,
    error_modal: ErrorModal,
    add_tab_modal: AddTabModal,
}

impl OG2App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }

    fn handle_error<T>(&mut self, result: Result<T>) -> Option<T> {
        match result {
            Ok(t) => Some(t),
            Err(e) => {
                self.error_modal.set_error_msg(e.to_string());
                self.error_modal.open();
                None
            },
        }
    }

    fn show_modals(&mut self, ctx: &Context, ui: &mut Ui) {
        self.error_modal.show(ctx, ui);
        let add_tab_modal_res = self.add_tab_modal.show(ctx, ui, &mut self.tabs, &mut self.active_tab);
        self.handle_error(add_tab_modal_res);
    }

    fn show_welcome_btns(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Init").clicked() {
                // TODO: Implement Init
            }
            if ui.button("Open").clicked() {
                let res = utils::open_repo_as_tab(&mut self.tabs, &mut self.active_tab);
                self.handle_error(res);
            }
            if ui.button("Clone").clicked() {
                // TODO: Implement Clone
            }
        });
    }

    fn show_tab_btns(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tab_width = ui.available_width() / self.tabs.len() as f32 - TAB_ADD_BTN_WIDTH;
            for (i, tab) in self.tabs.iter().enumerate() {
                let selectable_label = SelectableLabel::new(self.active_tab == i, &tab.name);
                if ui.add_sized(egui::vec2(tab_width, TAB_HEIGHT), selectable_label).clicked() {
                    self.active_tab = i;
                }
            }
            if ui.add_sized(egui::vec2(TAB_ADD_BTN_WIDTH, TAB_HEIGHT), Button::new("+")).clicked() {
                self.add_tab_modal.open();
            }
        });
    }
}

impl eframe::App for OG2App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_modals(ctx, ui);

            if self.tabs.len() > 0 {
                self.show_tab_btns(ui);
                self.tabs[self.active_tab].show(ui);
            } else {
                // TODO: Add welcome splash screen?
                self.show_welcome_btns(ui);
            }
        });
    }
}
