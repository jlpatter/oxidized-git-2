use anyhow::Result;
use git2::{Oid, Repository, Sort};

pub fn git_revwalk(repo: &Repository) -> Result<Vec<Oid>> {
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
