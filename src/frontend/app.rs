use anyhow::Result;
use eframe::Frame;
use egui::{Button, Context, SelectableLabel, Ui};
use crate::frontend::modals::AddTabModal;
use crate::frontend::tab::OG2Tab;
use crate::frontend::utils;

const TAB_HEIGHT: f32 = 20.0;
const TAB_ADD_BTN_WIDTH: f32 = 20.0;

fn handle_error<T>(result: Result<T>) -> Option<T> {
    match result {
        Ok(t) => Some(t),
        Err(_e) => {
            // TODO: Handle errors in some way!
            None
        },
    }
}

#[derive(Default)]
pub struct OG2App {
    tabs: Vec<OG2Tab>,
    active_tab: usize,
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

    fn show_starting_btns(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Init").clicked() {
                // TODO: Implement Init
            }
            if ui.button("Open").clicked() {
                handle_error(utils::open_repo_as_tab(&mut self.tabs, &mut self.active_tab));
            }
            if ui.button("Clone").clicked() {
                // TODO: Implement Clone
            }
        });
    }

    fn show_tabs(&mut self, ui: &mut Ui) {
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
            handle_error(self.add_tab_modal.show(ctx, ui, &mut self.tabs, &mut self.active_tab));

            if self.tabs.len() > 0 {
                self.show_tabs(ui);
                self.tabs[self.active_tab].show(ui);
            } else {
                // TODO: Add welcome splash screen?
                self.show_starting_btns(ui);
            }
        });
    }
}
