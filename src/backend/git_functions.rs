use anyhow::{Error, Result};
use git2::{AutotagOption, FetchOptions, FetchPrune, Oid, Repository, Sort};
use crate::backend::git_utils;

pub fn git_revwalk(repo: &Repository) -> Result<Vec<Oid>> {
    // First, we need to get the commits to start/include in the revwalk.
    let mut initial_oid_vec: Vec<Oid> = vec![];
    for branch_result in repo.branches(None)? {
        let (branch, _) = branch_result?;
        match branch.get().target() {
            Some(oid) => {
                if !initial_oid_vec.contains(&oid) {
                    initial_oid_vec.push(oid);
                }
            },
            None => (),
        };
    };

    if repo.head_detached()? {
        match repo.head()?.target() {
            Some(oid) => {
                if !initial_oid_vec.contains(&oid) {
                    initial_oid_vec.push(oid);
                }
            },
            None => (),
        };
    }

    // Sort Oids by date first
    initial_oid_vec.sort_by(|a, b| {
        repo.find_commit(*b).unwrap().time().seconds().partial_cmp(&repo.find_commit(*a).unwrap().time().seconds()).unwrap()
    });

    let mut revwalk = repo.revwalk()?;

    for oid in initial_oid_vec {
        revwalk.push(oid)?;
    }
    revwalk.set_sorting(Sort::TOPOLOGICAL)?;

    let mut all_oids_vec: Vec<Oid> = vec![];
    for commit_oid_result in revwalk {
        all_oids_vec.push(commit_oid_result?);
    }
    Ok(all_oids_vec)
}

pub fn git_fetch(repo: &Repository) -> Result<()> {
    let remote_string_array = repo.remotes()?;
    let empty_refspecs: &[String] = &[];
    for remote_string_opt in remote_string_array.iter() {
        let remote_string = remote_string_opt.ok_or(Error::msg("Remote Name has invalid UTF-8!"))?;
        let mut remote = repo.find_remote(remote_string)?;
        let mut fetch_options = FetchOptions::new();
        fetch_options.download_tags(AutotagOption::All);
        fetch_options.prune(FetchPrune::On);
        fetch_options.remote_callbacks(git_utils::get_remote_callbacks());
        remote.fetch(empty_refspecs, Some(&mut fetch_options), None)?;
    }
    Ok(())
}
