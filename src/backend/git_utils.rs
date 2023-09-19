use std::path::PathBuf;
use anyhow::{bail, Error, Result};
use directories::UserDirs;
use git2::{Config, Cred, CredentialHelper, Reference, RemoteCallbacks, Repository};
use rfd::FileDialog;

pub fn open_repo() -> Result<Option<(String, Repository)>> {
    let start_dir = match UserDirs::new() {
        Some(ud) => PathBuf::from(ud.home_dir()),
        None => PathBuf::from("/"),
    };
    let folder = FileDialog::new()
        .set_directory(start_dir)
        .pick_folder();
    if let Some(pf) = folder {
        // Get the name of the repo from the path.
        let mut name = String::from("(Invalid UTF-8 in Name)");
        if let Some(os_s) = pf.file_name() {
            if let Some(s) = os_s.to_str() {
                name = String::from(s);
            }
        }

        match Repository::open(pf) {
            Ok(repo) => return Ok(Some((name, repo))),
            Err(e) => bail!(e),
        }
    }
    Ok(None)
}

pub fn get_all_refs(repo: &Repository) -> Result<[Vec<Reference>; 3]> {
    let mut local_ref_shorthands = vec![];
    let mut remote_ref_shorthands = vec![];
    let mut tag_ref_shorthands = vec![];

    for ref_result in repo.references()? {
        let reference = ref_result?;
        let branch_shorthand = reference.shorthand().ok_or(Error::msg("Branch Shorthand has invalid UTF-8!"))?;

        if reference.is_branch() {
            local_ref_shorthands.push(reference);
        } else if reference.is_remote() && !branch_shorthand.ends_with("/HEAD") {
            remote_ref_shorthands.push(reference);
        } else if reference.is_tag() {
            tag_ref_shorthands.push(reference);
        }
    }
    Ok([local_ref_shorthands, remote_ref_shorthands, tag_ref_shorthands])
}

pub fn get_remote_callbacks() -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|url, _username_from_url, _allowed_types| {
        let default_git_config = match Config::open_default() {
            Ok(c) => c,
            Err(e) => return Err(e),
        };
        let user_pass_opt = CredentialHelper::new(url).config(&default_git_config).execute();
        match user_pass_opt {
            Some((username, password)) => {
                Cred::userpass_plaintext(username.as_str(), password.as_str())
            },
            None => {
                Err(git2::Error::from_str("Error: Can't retrieve username and password from credential helper! Maybe you need to set a credential helper in your git config?"))
            },
        }
    });
    callbacks.push_update_reference(|_ref_name, status_msg| {
        match status_msg {
            Some(m) => Err(git2::Error::from_str(&*format!("Error(s) during push: {}", m))),
            None => Ok(()),
        }
    });
    callbacks
}
