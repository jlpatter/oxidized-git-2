mod git_functions;

use anyhow::Result;
use git2::Repository;
use iced::widget::{button, Column, Row};
use iced::{Alignment, Element, Padding, Sandbox, Settings};


fn main()  -> iced::Result {
    OxidizedGit2::run(Settings::default())
}

struct OxidizedGit2 {
    repo: Option<Repository>
}

#[derive(Debug, Clone, Copy)]
enum Message {
    OpenPressed,
    FetchPressed,
}

impl OxidizedGit2 {
    fn handle_error<T>(result: Result<T>) -> Option<T> {
        match result {
            Ok(t) => Some(t),
            Err(e) => {
                // TODO: Handle errors in some way!
                None
            },
        }
    }
}

impl Sandbox for OxidizedGit2 {
    type Message = Message;

    fn new() -> Self {
        Self {
            repo: None,
        }
    }

    fn title(&self) -> String {
        String::from("Oxidized Git 2")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::OpenPressed => {
                let repo_opt_opt = OxidizedGit2::handle_error(git_functions::open_repo());
                if let Some(repo_opt) = repo_opt_opt {
                    self.repo = repo_opt;
                }
            }
            Message::FetchPressed => {
                // TODO: Implement Fetch.
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let btn_row: Row<Message> = Row::new()
            .push(button("Open").on_press(Message::OpenPressed))
            .push(button("Fetch").on_press(Message::FetchPressed));

        Column::new().push(btn_row)
            .padding(Padding::from([20, 0, 0, 20]))
            .align_items(Alignment::Center)
            .into()
    }
}
