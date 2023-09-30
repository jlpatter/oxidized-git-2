use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use anyhow::Result;
use egui::{ColorImage, Context};
use git2::Repository;
use image::io::Reader;
use crate::backend::git_utils;
use crate::frontend::commit_graph::CommitGraph;
use crate::frontend::modals::ErrorModal;
use crate::frontend::tab::OG2Tab;

pub fn open_repo_as_tab(tabs_arc: Arc<Mutex<Vec<OG2Tab>>>, active_tab_arc: Arc<Mutex<usize>>, error_modal_arc: Arc<Mutex<ErrorModal>>, is_loading: Arc<Mutex<bool>>, ctx_c: Context) -> Result<()> {
    let repo_opt = git_utils::open_repo()?;
    // If a repo was actually opened
    if let Some((name, repo)) = repo_opt {
        thread::spawn(move || {
            *is_loading.lock().unwrap() = true;
            let new_tab_res = OG2Tab::new(name, repo, is_loading.clone(), error_modal_arc.clone(), &ctx_c);  // This line is slow!
            // This is on a separate line so it doesn't get locked too early.
            let new_tab_opt = error_modal_arc.lock().unwrap().handle_error(new_tab_res);
            if let Some(new_tab) = new_tab_opt {
                let mut tabs = tabs_arc.lock().unwrap();
                tabs.push(new_tab);

                let mut active_tab = active_tab_arc.lock().unwrap();
                *active_tab = tabs.len() - 1;
            }
            *is_loading.lock().unwrap() = false;
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

pub fn perform_fn_in_thread(
    some_fn: fn(&Repository) -> Result<()>,
    repo_c: Arc<Mutex<Repository>>,
    error_modal_c: Arc<Mutex<ErrorModal>>,
    commit_graph_c: Arc<Mutex<CommitGraph>>,
    is_loading_c: Arc<Mutex<bool>>
) {
    thread::spawn(move || {
        *is_loading_c.lock().unwrap() = true;
        let res = some_fn(&repo_c.lock().unwrap());
        let opt = error_modal_c.lock().unwrap().handle_error(res);
        if let Some(()) = opt {
            let res = commit_graph_c.lock().unwrap().refresh_graph(&repo_c.lock().unwrap());
            error_modal_c.lock().unwrap().handle_error(res);
        }
        *is_loading_c.lock().unwrap() = false;
    });
}