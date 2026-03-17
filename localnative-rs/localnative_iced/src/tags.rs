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
    pub fn view(&self) -> Element<'_, Message> {
        row![
            button(text(&self.tag.tag).size(16))
                .style(crate::style::tag_style)
                .on_press(Message::Search(self.tag.tag.clone())),
            button(text(self.tag.count).size(20))
                .style(crate::style::tag_num_style)
                .on_press(Message::Search(self.tag.count.to_string())),
        ]
        .into()
    }
}

// Preview support removed - iced::Sandbox no longer exists in iced 0.13
