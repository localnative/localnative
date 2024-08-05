use iced::widget::{button, row, text};
use iced::Element;
use localnative_core::db::models::Tags;

#[derive(Debug, Clone)]
pub enum Message {
    Search(String),
}

#[derive(Debug, Default, Clone)]
pub struct TagView {
    pub tag: Tags,
}
impl From<Tags> for TagView {
    fn from(tag: Tags) -> Self {
        Self { tag }
    }
}
impl TagView {
    pub fn view(&self) -> Element<Message> {
        row![
            button(text(&self.tag.tag).size(16))
                .style(crate::style::Tag)
                .on_press(Message::Search(self.tag.tag.clone())),
            button(text(self.tag.count).size(20))
                .style(crate::style::TagNum)
                .on_press(Message::Search(self.tag.count.to_string())),
        ]
        .into()
    }
}

#[cfg(feature = "preview")]
impl iced::Sandbox for TagView {
    type Message = Message;

    fn new() -> Self {
        Tag {
            name: "testtag".to_owned(),
            count: 16,
        }
        .into()
    }

    fn title(&self) -> String {
        "tagview preview".to_owned()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::Search(s) => println!("{}", s),
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.view()
    }
}
