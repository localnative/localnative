use iced::{button, Button, Element, Row, Text};
use serde::{Deserialize, Serialize};

use crate::style::{self, Theme};

#[derive(Debug, Clone)]
pub enum Message {
    Search(String),
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Tag {
    #[serde(rename = "k")]
    pub name: String,
    #[serde(rename = "v")]
    pub count: i32,
}
#[derive(Debug, Default)]
pub struct TagView {
    pub tag: Tag,
    pub search_button: button::State,
    pub count_button: button::State,
}
impl From<Tag> for TagView {
    fn from(tag: Tag) -> Self {
        Self {
            tag,
            search_button: button::State::new(),
            count_button: button::State::new(),
        }
    }
}
impl TagView {
    pub fn view(&mut self,theme:Theme) -> Element<Message> {
        Row::new()
            .push(
                Button::new(
                    &mut self.search_button,
                    Text::new(self.tag.name.as_str()).size(16),
                )
                .style(style::tag(theme))
                .on_press(Message::Search(self.tag.name.clone())),
            )
            .push(
                Button::new(
                    &mut self.count_button,
                    Text::new(self.tag.count.to_string()).color([1.0, 0.0, 0.0]),
                )
                .on_press(Message::Search(self.tag.count.to_string()))
                .style(style::count(theme)),
            )
            .into()
    }
}


#[cfg(feature = "preview")]
impl iced::Sandbox for TagView {
    type Message = Message;

    fn new() -> Self {
        Tag {
            name:"testtag".to_owned(),
            count: 16
        }.into()
    }

    fn title(&self) -> String {
        "tagview preview".to_owned()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::Search(s) => println!("{}", s),
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.view(Theme::Light)
    }
}
