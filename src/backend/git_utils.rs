use std::path::PathBuf;
use anyhow::{bail, Result};
use directories::UserDirs;
use git2::Repository;
use rfd::FileDialog;

fn get_utf8_string<'a, 'b>(value: Option<&'a str>, str_name_type: &'b str) -> Result<&'a str> {
    match value {
        Some(n) => Ok(n),
        None => bail!(format!("{} uses invalid utf-8!", str_name_type)),
    }
}

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

pub fn get_branch_trees(repo: &Repository) -> Result<Vec<String>> {
    let mut branches: Vec<String> = vec![];
    for ref_result in repo.references()? {
        let reference = ref_result?;

        let branch_shorthand = get_utf8_string(reference.shorthand(), "Branch Shorthand")?;

        branches.push(String::from(branch_shorthand));
    }
    Ok(branches)
}