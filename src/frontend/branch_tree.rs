use anyhow::Result;
use egui::{Label, Sense, Ui};
use git2::Repository;
use crate::backend::git_utils;

static TAB_SIZE: f32 = 20.0;

pub fn get_branch_trees(repo: &Repository) -> Result<Vec<BranchTreeNode>> {
    let ref_shorthands = git_utils::get_all_ref_shorthands(repo)?;
    let mut branch_tree = BranchTreeNode::new(String::new());

    for ref_shorthand in ref_shorthands {
        branch_tree.insert_shorthand(ref_shorthand);
    }
    // TODO: Add local, remote, and tag trees as separate trees!
    Ok(vec![branch_tree])
}

pub struct BranchTreeNode {
    text: String,
    is_expanded: bool,
    children: Vec<BranchTreeNode>,
}

impl BranchTreeNode {
    pub fn new(text: String) -> Self {
        Self {
            text,
            is_expanded: false,
            children: vec![],
        }
    }

    pub fn insert_shorthand(&mut self, branch_shorthand: String) {
        // self should be the root node in this case.
        assert_eq!(self.text, String::from(""));
        let mut current_tree_node = self;

        let split_shorthand: Vec<&str> = branch_shorthand.split("/").collect();

        for (i, string_ref) in split_shorthand.iter().enumerate() {
            let shorthand_piece = *string_ref;
            let child_index = current_tree_node.children.iter().position(|child| {
                child.text == shorthand_piece
            });
            match child_index {
                Some(j) => {
                    current_tree_node = &mut current_tree_node.children[j];
                },
                None => {
                    if i == split_shorthand.len() - 1 {
                        // TODO: This is where branch information can be passed!
                        current_tree_node.children.push(BranchTreeNode::new(String::from(shorthand_piece)));
                    } else {
                        current_tree_node.children.push(BranchTreeNode::new(String::from(shorthand_piece)));
                    }
                    let last_index = current_tree_node.children.len() - 1;
                    current_tree_node = &mut current_tree_node.children[last_index];
                },
            };
        }
    }

    pub fn show(&mut self, ui: &mut Ui, rec_depth: f32) {
        let is_root = rec_depth < 0.0;
        if !is_root {
            ui.horizontal(|ui| {
                ui.add_space(rec_depth * TAB_SIZE);
                let resp = ui.add(Label::new(self.text.clone()).wrap(false)).interact(Sense::click());
                if resp.clicked() {
                    self.is_expanded = !self.is_expanded;
                }
            });
        }
        if self.is_expanded || is_root {
            for child in &mut self.children {
                child.show(ui, rec_depth + 1.0);
            }
        }
    }
}