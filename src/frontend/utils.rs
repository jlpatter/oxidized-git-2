use std::path::Path;
use anyhow::Result;
use egui::{ColorImage, Context};
use image::io::Reader;
use crate::backend::git_utils;
use crate::frontend::tab::OG2Tab;

pub fn open_repo_as_tab(tabs: &mut Vec<OG2Tab>, active_tab: &mut usize, ctx: &Context) -> Result<()> {
    let repo_opt = git_utils::open_repo()?;
    // If a repo was actually opened
    if let Some((name, repo)) = repo_opt {
        let new_tab = OG2Tab::new(name, repo, ctx)?;
        tabs.push(new_tab);
        *active_tab = tabs.len() - 1;
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
