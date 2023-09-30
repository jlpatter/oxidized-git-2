use anyhow::{bail, Error, Result};
use git2::{AutotagOption, BranchType, FetchOptions, FetchPrune, Oid, Repository, Sort};
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

pub fn git_pull(repo: &Repository) -> Result<()> {
    // Fetch first to make sure everything's up to date.
    git_fetch(repo)?;

    let mut local_ref = repo.head()?;
    let local_shorthand = local_ref.shorthand().ok_or(Error::msg("Branch Name has invalid UTF-8!"))?;
    let local_branch = repo.find_branch(local_shorthand, BranchType::Local)?;

    let remote_branch = local_branch.upstream()?;
    let remote_ref = remote_branch.get();
    let remote_target = match remote_ref.target() {
        Some(oid) => oid,
        None => bail!("Remote branch is not targeting a commit, cannot pull."),
    };
    let remote_ac = repo.find_annotated_commit(remote_target)?;

    let (ma, mp) = repo.merge_analysis(&[&remote_ac])?;

    if ma.is_none() {
        bail!("Merge analysis indicates no merge is possible. If you're reading this, your repository may be corrupted.");
    } else if ma.is_unborn() {
        bail!("The HEAD of the current repository is “unborn” and does not point to a valid commit. No pull can be performed, but the caller may wish to simply set HEAD to the target commit(s).");
    } else if ma.is_up_to_date() {
        return Ok(());
    } else if ma.is_fast_forward() && !mp.is_no_fast_forward() {
        println!("Performing fast forward merge for pull!");
        let commit = match remote_ref.target() {
            Some(oid) => repo.find_commit(oid)?,
            None => bail!("Remote branch has no target commit."),
        };
        let tree = commit.tree()?;
        repo.checkout_tree(tree.as_object(), None)?;
        local_ref.set_target(remote_target, "oxidized_git pull: setting new target for local ref")?;
        return Ok(());
    } else if ma.is_normal() && !mp.is_fastforward_only() {
        println!("Performing rebase for pull!");
        let mut rebase = repo.rebase(None, None, Some(&remote_ac), None)?;
        let mut diff_in_branches = false;
        for step in rebase.by_ref() {
            step?;
            if git_utils::has_conflicts(repo)? || git_utils::has_unstaged_changes(repo)? || git_utils::has_staged_changes(repo)? {
                diff_in_branches = true;
                break;
            }
        }
        if diff_in_branches {
            rebase.abort()?;
            bail!("Pull by rebase aborted because changes on local branch differ from remote branch!");
        }
        rebase.finish(None)?;
        return Ok(());
    } else if (ma.is_fast_forward() && mp.is_no_fast_forward()) || (ma.is_normal() && mp.is_fastforward_only()) {
        bail!("It looks like a pull may be possible, but your MergePreference(s) are preventing it. If you have merge.ff or pull.ff set to 'only' or 'false', consider unsetting it by running 'git config --global --unset merge.ff' or 'git config --global --unset pull.ff'");
    }
    bail!("Merge analysis failed to make any determination on how to proceed with the pull. If you're reading this, your repository may be corrupted.")
}
