use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::Result;
use egui::{ColorImage, Context};
use image::io::Reader;
use crate::backend::git_utils;
use crate::frontend::modals::ErrorModal;
use crate::frontend::tab::OG2Tab;

pub fn open_repo_as_tab(tabs_arc: Arc<Mutex<Vec<OG2Tab>>>, active_tab_arc: Arc<Mutex<usize>>, error_modal_arc: Arc<Mutex<ErrorModal>>, ctx: &Context) -> Result<()> {
    let repo_opt = git_utils::open_repo()?;
    // If a repo was actually opened
    if let Some((name, repo)) = repo_opt {
        thread::spawn(move || {
            let mut error_modal = error_modal_arc.lock().unwrap();
            let new_tab_opt = error_modal.handle_error(OG2Tab::new(name, repo, ctx));
            if let Some(new_tab) = new_tab_opt {
                let mut tabs = tabs_arc.lock().unwrap();
                tabs.push(new_tab);

                let mut active_tab = active_tab_arc.lock().unwrap();
                *active_tab = tabs.len() - 1;
            }
        });
    }
    Ok(())
}

pub fn load_image_from_path(path: &Path) -> Result<ColorImage> {
    let image = Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}
