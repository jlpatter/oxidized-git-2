use anyhow::{bail, Result};
use git2::Repository;
use rfd::FileDialog;

pub fn open_repo() -> Result<Option<Repository>> {
    let folder = FileDialog::new()
        .set_directory("/")
        .pick_folder();
    if let Some(pf) = folder {
        match Repository::open(pf) {
            Ok(repo) => return Ok(Some(repo)),
            Err(e) => bail!(e),
        }
    }
    Ok(None)
}
