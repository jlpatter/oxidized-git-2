use anyhow::Result;
use crate::backend::git_utils;
use crate::frontend::tab::OG2Tab;

pub fn open_repo_as_tab(tabs: &mut Vec<OG2Tab>, active_tab: &mut usize) -> Result<()> {
    let repo_opt = git_utils::open_repo()?;
    // If a repo was actually opened
    if let Some((name, repo)) = repo_opt {
        let new_tab = OG2Tab::new(name, repo)?;
        tabs.push(new_tab);
        *active_tab = tabs.len() - 1;
    }
    Ok(())
}
