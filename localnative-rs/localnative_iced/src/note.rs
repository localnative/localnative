use crate::icons::text;
use iced::{
    theme,
    widget::{button, column, container, horizontal_space, row, QRCode},
    Element,
    Length::Fill,
    Length::FillPortion,
};
use localnative_core::Note;

use crate::icons::IconItem;
#[derive(Debug)]
pub struct NoteView {
    note: Note,
    tags: Vec<Tag>,
    qrcode: Option<iced::widget::qr_code::State>,
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
    pub fn view(&self) -> Element<Message> {
        let Self { note, tags, qrcode } = self;
        let qrcode = qrcode.as_ref().map(|state| QRCode::new(state));
        let url = button(text(&note.url))
            .style(crate::style::Url.into())
            .padding(0)
            .on_press(Message::OpenUrl);
        let delete = button(IconItem::Delete)
            .style(theme::Button::Text)
            .on_press(Message::Delete(note.rowid));
        let qrcode_button = button(IconItem::QRCode)
            .style(theme::Button::Text)
            .padding(0)
            .on_press(Message::QRCode);
        let row = row![
            text(&note.created_at),
            text(&note.uuid4),
            text(format!("rowid {}", note.rowid)),
            qrcode_button
        ]
        .spacing(5);
        let wrap = tags
            .iter()
            .fold(iced_aw::Wrap::new().spacing(5.).push(row), |wrap, tag| {
                let Tag { name } = tag;
                let tag_button = button(text(&name).size(14))
                    .style(crate::style::Tag.into())
                    .padding(1)
                    .on_press(Message::Search(name.to_owned()));
                wrap.push(tag_button)
            })
            .align_items(iced::Alignment::Center);
        let mut column = column![wrap];
        if let Some(qrcode) = qrcode {
            column = column.push(row![horizontal_space(Fill), qrcode, horizontal_space(Fill)]);
        }
        if !note.title.is_empty() {
            column = column.push(text(&note.title));
        }
        if !note.url.is_empty() {
            column = column.push(url);
        }
        if !note.description.is_empty() {
            column = column.push(text(&note.description));
        }
        if !note.comments.is_empty() {
            column = column.push(text(&note.comments));
        }

        column = column.push(row![
            horizontal_space(FillPortion(12)),
            delete,
            horizontal_space(FillPortion(1))
        ]);

        container(column)
            .padding(1)
            .style(crate::style::SimpleBox)
            .into()
    }
    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::OpenUrl => open(self.note.url.as_str()),
            Message::Delete(..) => {
                // 上层处理
                println!("delete");
            }
            Message::QRCode => match self.qrcode {
                Some(_) => {
                    self.qrcode.take();
                }
                None => {
                    self.qrcode.replace(
                        iced::widget::qr_code::State::new(self.note.url.as_bytes()).unwrap(),
                    );
                }
            },
            Message::Search(tag) => {
                // 上层处理
                println!("search tag: {}", tag);
            }
        }
    }
}
fn open(url: &str) {
    if let Err(err) = open::that(url) {
        println!("open url fail:{:?}", err);
    }
}

#[cfg(feature = "preview")]
impl iced::Sandbox for NoteView {
    type Message = Message;

    fn new() -> Self {
        Note {
            rowid: 1,
            uuid4: "490b28dc-8d96-4fd8-b0ae-1c3c200901f3".to_owned(),
            title: "localnative".to_owned(),
            url: "https://localnative.app/".to_owned(),
            tags: "tool,rust,note,tag,description,url,title".to_owned(),
            description: "test description".to_owned(),
            comments: "test comments".to_owned(),
            annotations: "".to_owned(),
            created_at: "2021-05-28 08:30:00:000000000 UTC".to_owned(),
            is_public: true,
        }
        .into()
    }

    fn title(&self) -> String {
        "noteview-preview".to_owned()
    }

    fn update(&mut self, message: Self::Message) {
        self.update(message)
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.view()
    }
}
