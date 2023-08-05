use iced::widget::{button, row};
use iced::{Alignment, Element, Padding, Sandbox, Settings};


fn main()  -> iced::Result {
    OxidizedGit2::run(Settings::default())
}

struct OxidizedGit2 {}

#[derive(Debug, Clone, Copy)]
enum Message {
    HelloWorldPressed,
}

impl Sandbox for OxidizedGit2 {
    type Message = Message;

    fn new() -> Self {
        Self {}
    }

    fn title(&self) -> String {
        String::from("Oxidized Git 2")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::HelloWorldPressed => {
                println!("Hello World!");
            }
        }
    }

    fn view(&self) -> Element<Message> {
        row![
            button("Click me!").on_press(Message::HelloWorldPressed)
        ]
        .padding(Padding::from([20, 0, 0, 20]))
        .align_items(Alignment::Center)
        .into()
    }
}
