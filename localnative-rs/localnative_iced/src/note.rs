use iced::pure::{
    widget::{Button, Column, QRCode, Row, Text},
    Element,
};
use localnative_core::Note;

use crate::{icons::IconItem, style};
#[derive(Debug)]
pub struct NoteView {
    note: Note,
    tags: Vec<Tag>,
    qrcode: Option<iced::pure::widget::qr_code::State>,
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
    pub fn view(&self, theme: style::Theme) -> Element<Message> {
        let Self { note, tags, qrcode } = self;
        let qrcode = qrcode
            .as_ref()
            .map(|state| style::qr_code(QRCode::new(state), theme));
        let url = Button::new(Text::new(note.url.as_str()))
            .style(style::link(theme))
            .padding(0)
            .on_press(Message::OpenUrl);
        let delete = Button::new(IconItem::Delete)
            .style(style::transparent(theme))
            .on_press(Message::Delete(note.rowid));
        let qrcode_button = Button::new(IconItem::QRCode)
            .padding(0)
            .style(style::transparent(theme))
            .on_press(Message::QRCode);
        let row = Row::new()
            .spacing(5)
            .push(Text::new(note.created_at.as_str()))
            .push(Text::new(note.uuid4.as_str()))
            .push(Text::new(format!("rowid {}", note.rowid)))
            .push(qrcode_button);
        let wrap = tags.iter().fold(
            iced_aw::pure::Wrap::new().spacing(5).push(row),
            |wrap, tag| {
                let Tag { name } = tag;
                let tag_button = Button::new(Text::new(name.as_str()))
                    .style(style::tag(theme))
                    .padding(0)
                    .on_press(Message::Search(name.to_owned()));
                wrap.push(tag_button)
            },
        );
        let mut column = Column::new().push(wrap);
        if let Some(qrcode) = qrcode {
            column = column.push(
                Row::new()
                    .push(style::horizontal_rule())
                    .push(qrcode)
                    .push(style::horizontal_rule()),
            );
        }
        if !note.title.is_empty() {
            column = column.push(Text::new(note.title.as_str()));
        }
        if !note.url.is_empty() {
            column = column.push(url);
        }
        if !note.description.is_empty() {
            column = column.push(Text::new(note.description.as_str()));
        }
        if !note.comments.is_empty() {
            column = column.push(Text::new(note.comments.as_str()));
        }

        column = column.push(
            Row::with_children(style::horizontal_rules::<Message>(7))
                .push(delete)
                .push(style::horizontal_rule()),
        );
        iced::pure::widget::Container::new(column)
            .style(style::note(theme))
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
                        iced::pure::widget::qr_code::State::new(self.note.url.as_bytes()).unwrap(),
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

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.view(style::Theme::Light)
    }
}
