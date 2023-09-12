use std::sync::{Arc, Mutex};
use eframe::Frame;
use egui::{Button, Context, SelectableLabel, Ui, Vec2};
use crate::frontend::modals::{AddTabModal, ErrorModal, Modal};
use crate::frontend::tab::OG2Tab;
use crate::frontend::utils;

const TAB_HEIGHT: f32 = 20.0;
const TAB_ADD_BTN_WIDTH: f32 = 20.0;

#[derive(Default)]
pub struct OG2App {
    tabs: Arc<Mutex<Vec<OG2Tab>>>,
    active_tab: Arc<Mutex<usize>>,
    error_modal: Arc<Mutex<ErrorModal>>,
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

    fn show_modals(&mut self, ui: &mut Ui) {
        let add_tab_modal_res = self.add_tab_modal.show(ui, self.tabs.clone(), self.active_tab.clone(), self.error_modal.clone());
        let mut error_modal = self.error_modal.lock().unwrap();
        error_modal.handle_error(add_tab_modal_res);
        error_modal.show(ui);
    }

    fn show_welcome_btns(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Init").clicked() {
                // TODO: Implement Init
            }
            if ui.button("Open").clicked() {
                let res = utils::open_repo_as_tab(self.tabs.clone(), self.active_tab.clone(), self.error_modal.clone(), ui.ctx());
                self.error_modal.lock().unwrap().handle_error(res);
            }
            if ui.button("Clone").clicked() {
                // TODO: Implement Clone
            }
        });
    }

    fn show_tab_btns(&mut self, ui: &mut Ui, tabs: &Vec<OG2Tab>) {
        ui.horizontal(|ui| {
            let active_tab = *self.active_tab.lock().unwrap();
            let tab_width = ui.available_width() / tabs.len() as f32 - TAB_ADD_BTN_WIDTH;
            for (i, tab) in tabs.iter().enumerate() {
                let selectable_label = SelectableLabel::new(active_tab == i, &tab.name);
                if ui.add_sized(Vec2::new(tab_width, TAB_HEIGHT), selectable_label).clicked() {
                    self.active_tab = Arc::new(Mutex::new(i));
                }
            }
            if ui.add_sized(Vec2::new(TAB_ADD_BTN_WIDTH, TAB_HEIGHT), Button::new("+")).clicked() {
                self.add_tab_modal.open();
            }
        });
    }
}

impl eframe::App for OG2App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_modals(ui);

            // This is done so 'self' doesn't get borrowed twice.
            let tabs_c = self.tabs.clone();
            let mut tabs = tabs_c.lock().unwrap();
            if tabs.len() > 0 {
                self.show_tab_btns(ui, &tabs);
                tabs[*self.active_tab.lock().unwrap()].show(ui);
            } else {
                // TODO: Add welcome splash screen?
                self.show_welcome_btns(ui);
            }
        });
    }
}
