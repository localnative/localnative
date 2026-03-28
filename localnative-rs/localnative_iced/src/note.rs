use iced::{
    Element,
    Length::FillPortion,
    widget::{QRCode, Space, button, column, container, row, text},
};
use localnative_core::db::models::Note;

use crate::icons::IconItem;

/// Wrapper to make qr_code::Data usable across thread boundaries.
/// qr_code::Data is only created and accessed on the main/GUI thread.
struct SendableQrData(iced::widget::qr_code::Data);

// SAFETY: qr_code::Data is only created and used on the GUI thread.
// It passes through Task::perform but is always None during transit.
unsafe impl Send for SendableQrData {}

impl std::fmt::Debug for SendableQrData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SendableQrData").finish()
    }
}

#[derive(Debug)]
pub struct NoteView {
    note: Note,
    tags: Vec<Tag>,
    qrcode: Option<SendableQrData>,
}

#[derive(Debug, Clone)]
pub struct Tag {
    name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenUrl,
    Delete(i64),
    QRCode,
    Search(String),
}

impl From<Note> for NoteView {
    fn from(note: Note) -> Self {
        let tags: Vec<Tag> = note
            .tags
            .split(',')
            .filter(|name| !name.is_empty())
            .map(|name| Tag {
                name: name.to_owned(),
            })
            .collect();
        NoteView {
            note,
            tags,
            qrcode: None,
        }
    }
}

impl NoteView {
    pub fn view(&self) -> Element<'_, Message> {
        let qrcode_widget = self.qrcode.as_ref().map(|d| QRCode::new(&d.0));
        let url_button = self.create_url_button();
        let delete_button = self.create_delete_button();
        let qrcode_button = self.create_qrcode_button();
        let info_row = row![
            text(&self.note.created_at),
            text(&self.note.uuid4),
            text(format!("rowid {}", self.note.rowid)),
            qrcode_button
        ]
        .spacing(5);

        let wrap = self.tags.iter().fold(
            iced_aw::Wrap::new()
                .spacing(5.)
                .push(Element::from(info_row)),
            |wrap, tag| {
                let tag_button = button(text(&tag.name))
                    .style(crate::style::tag_style)
                    .padding(0)
                    .on_press(Message::Search(tag.name.to_owned()));
                wrap.push(tag_button)
            },
        );

        let mut col = column![wrap];
        if let Some(qrcode) = qrcode_widget {
            col = col.push(row![
                Space::new().width(iced::Length::Fill),
                qrcode,
                Space::new().width(iced::Length::Fill)
            ]);
        }
        if !self.note.title.is_empty() {
            col = col.push(text(&self.note.title));
        }
        if !self.note.url.is_empty() {
            col = col.push(url_button);
        }
        if !self.note.description.is_empty() {
            col = col.push(text(&self.note.description));
        }
        if !self.note.comments.is_empty() {
            col = col.push(text(&self.note.comments));
        }
        col = col.push(row![
            Space::new().width(FillPortion(12)),
            delete_button,
            Space::new().width(FillPortion(1))
        ]);

        container(col)
            .padding(1)
            .style(crate::style::simple_box_style)
            .into()
    }

    fn create_url_button(&self) -> button::Button<'_, Message> {
        button(text(&self.note.url))
            .style(crate::style::url_style)
            .padding(0)
            .on_press(Message::OpenUrl)
    }

    fn create_delete_button(&self) -> button::Button<'_, Message> {
        button(IconItem::Delete)
            .style(button::text)
            .on_press(Message::Delete(self.note.rowid))
    }

    fn create_qrcode_button(&self) -> button::Button<'_, Message> {
        button(IconItem::QRCode)
            .style(button::text)
            .padding(0)
            .on_press(Message::QRCode)
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::OpenUrl => self.open_url(),
            Message::Delete(_) => tracing::debug!("delete action (handled by parent)"),
            Message::QRCode => self.toggle_qrcode(),
            Message::Search(ref tag) => tracing::debug!(tag, "search by tag (handled by parent)"),
        }
    }

    fn open_url(&self) {
        if let Err(err) = open::that(&self.note.url) {
            tracing::error!(?err, "failed to open url");
        }
    }

    fn toggle_qrcode(&mut self) {
        match self.qrcode {
            Some(_) => {
                self.qrcode.take();
            }
            None => {
                if let Ok(data) = iced::widget::qr_code::Data::new(self.note.url.as_bytes()) {
                    self.qrcode.replace(SendableQrData(data));
                }
            }
        };
    }
}

// Preview support removed - iced::Sandbox no longer exists in iced 0.13
