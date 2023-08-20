use std::path::PathBuf;
use anyhow::{bail, Result};
use directories::UserDirs;
use git2::Repository;
use rfd::FileDialog;

pub fn open_repo() -> Result<Option<Repository>> {
    let start_dir = match UserDirs::new() {
        Some(ud) => PathBuf::from(ud.home_dir()),
        None => PathBuf::from("/"),
    };
    let folder = FileDialog::new()
        .set_directory(start_dir)
        .pick_folder();
    if let Some(pf) = folder {
        match Repository::open(pf) {
            Ok(repo) => return Ok(Some(repo)),
            Err(e) => bail!(e),
        }
    }
    Ok(None)
}
