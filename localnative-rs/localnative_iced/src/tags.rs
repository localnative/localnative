use iced::{button, Button, Element, Row, Text};
use serde::{Deserialize, Serialize};

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
}

impl TagView {
    pub fn view(&mut self) -> Element<Message> {
        Row::new()
            .push(
                Button::new(
                    &mut self.search_button,
                    Text::new(self.tag.name.as_str()).size(16),
                )
                .style(crate::style::tag::Tag)
                .on_press(Message::Search(self.tag.name.clone())),
            )
            .push(Text::new(self.tag.count.to_string()).color([1.0, 0.0, 0.0]))
            .into()
    }
}
