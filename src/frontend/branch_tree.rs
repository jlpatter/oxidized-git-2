use std::path::Path;
use anyhow::{Error, Result};
use egui::{Context, Image, Label, Sense, TextureHandle, TextureOptions, Ui};
use git2::{BranchType, Reference, Repository};
use crate::backend::git_utils;
use crate::frontend::utils::load_image_from_path;

const TAB_SIZE: f32 = 20.0;

pub fn get_branch_trees(repo: &Repository, ctx: &Context) -> Result<[BranchTreeNode; 3]> {
    let ref_shorthand_types = git_utils::get_all_refs(repo)?;

    let mut branch_trees = [
        BranchTreeNode::new(String::from("Local"), false, true),
        BranchTreeNode::new(String::from("Remote"), false, true),
        BranchTreeNode::new(String::from("Tags"), false, true)
    ];

    // Load the arrow images here so they can be cheaply cloned.
    let right_arrow_image = load_image_from_path(Path::new("./images/right_arrow.png"))?;
    let right_arrow_texture = ctx.load_texture("right-arrow-image", right_arrow_image, TextureOptions::default());
    let down_arrow_image = load_image_from_path(Path::new("./images/down_arrow.png"))?;
    let down_arrow_texture = ctx.load_texture("down-arrow-image", down_arrow_image, TextureOptions::default());

    for (i, refs) in ref_shorthand_types.iter().enumerate() {
        for reference in refs {
            branch_trees[i].insert_shorthand(repo, reference, &right_arrow_texture, &down_arrow_texture)?;
        }
    }
    Ok(branch_trees)
}

pub struct BranchTreeNode {
    text: String,
    is_head: bool,
    is_expanded: bool,
    children: Vec<BranchTreeNode>,
    right_arrow_texture: Option<TextureHandle>,
    down_arrow_texture: Option<TextureHandle>,
}

impl BranchTreeNode {
    pub fn new(text: String, is_head: bool, is_expanded: bool) -> Self {
        Self {
            text,
            is_head,
            is_expanded,
            children: vec![],
            right_arrow_texture: None,
            down_arrow_texture: None,
        }
    }

    fn set_arrow_images(&mut self, right_arrow_texture: &TextureHandle, down_arrow_texture: &TextureHandle) {
        if self.right_arrow_texture == None {
            self.right_arrow_texture = Some(right_arrow_texture.clone());
        }
        if self.down_arrow_texture == None {
            self.down_arrow_texture = Some(down_arrow_texture.clone());
        }
    }

    pub fn insert_shorthand(&mut self, repo: &Repository, reference: &Reference, right_arrow_texture: &TextureHandle, down_arrow_texture: &TextureHandle) -> Result<()> {
        // NOTE: This function should only be called on a root node!
        let mut current_tree_node = self;

        let shorthand = reference.shorthand().ok_or(Error::msg("Branch Shorthand has invalid UTF-8!"))?;
        let mut is_head = false;
        if reference.is_branch() && repo.find_branch(shorthand, BranchType::Local)?.is_head() {
            is_head = true;
        }

        let split_shorthand: Vec<&str> = shorthand.split("/").collect();

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
                        // If this is a leaf...
                        // This is where branch information can be passed!
                        current_tree_node.children.push(BranchTreeNode::new(String::from(shorthand_piece), is_head, false));
                        current_tree_node.set_arrow_images(right_arrow_texture, down_arrow_texture);
                    } else {
                        // Otherwise, if this node has children...
                        current_tree_node.children.push(BranchTreeNode::new(String::from(shorthand_piece), false, false));
                        current_tree_node.set_arrow_images(right_arrow_texture, down_arrow_texture);
                    }
                    let last_index = current_tree_node.children.len() - 1;
                    current_tree_node = &mut current_tree_node.children[last_index];
                },
            };
        }
        Ok(())
    }

    pub fn show(&mut self, ui: &mut Ui, rec_depth: f32) {
        ui.horizontal(|ui| {
            ui.add_space(rec_depth * TAB_SIZE);
            let mut row_was_clicked = false;

            // Add arrows next to collapsables.
            if self.is_expanded {
                if let Some(down_arrow_texture) = &self.down_arrow_texture {
                    if ui.add(Image::new(down_arrow_texture, down_arrow_texture.size_vec2()).sense(Sense::click())).clicked() {
                        row_was_clicked = true;
                    }
                }
            } else {
                if let Some(right_arrow_texture) = &self.right_arrow_texture {
                    if ui.add(Image::new(right_arrow_texture, right_arrow_texture.size_vec2()).sense(Sense::click())).clicked() {
                        row_was_clicked = true;
                    }
                }
            }

            // Add text.
            let text;
            if self.is_head {
                text = format!("* {}", self.text);
            } else {
                text = self.text.clone();
            }
            if ui.add(Label::new(text).wrap(false)).interact(Sense::click()).clicked() {
                row_was_clicked = true;
            }
            if row_was_clicked {
                self.is_expanded = !self.is_expanded;
            }
        });
        if self.is_expanded {
            for child in &mut self.children {
                child.show(ui, rec_depth + 1.0);
            }
        }
    }
}