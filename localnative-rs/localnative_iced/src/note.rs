use iced::{
    theme,
    widget::{button, column, container, horizontal_space, row, text, Column, QRCode, Row, Space},
    Element,
    Length::FillPortion,
};
use iced_aw::direction::Horizontal;
use localnative_core::Note;

use crate::icons::IconItem;

#[derive(Debug)]
pub struct NoteView {
    note: Note,
    tags: Vec<Tag>,
    qrcode: Option<iced::widget::qr_code::Data>,
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
        let qrcode_widget = self.qrcode.as_ref().map(QRCode::new);
        let url_button = self.create_url_button();
        let delete_button = self.create_delete_button();
        let qrcode_button = self.create_qrcode_button();
        let row = self.create_info_row(qrcode_button);
        let wrap = self.create_tag_buttons(row);
        let mut column = self.create_content_column(wrap, qrcode_widget);
        column = self.add_note_details(column, url_button);
        column = self.add_delete_button(column, delete_button);

        container(column)
            .padding(1)
            .style(crate::style::SimpleBox)
            .into()
    }

    fn create_url_button(&self) -> button::Button<Message> {
        button(text(&self.note.url))
            .style(crate::style::Url)
            .padding(0)
            .on_press(Message::OpenUrl)
    }

    fn create_delete_button(&self) -> button::Button<Message> {
        button(IconItem::Delete)
            .style(theme::Button::Text)
            .on_press(Message::Delete(self.note.rowid))
    }

    fn create_qrcode_button(&self) -> button::Button<Message> {
        button(IconItem::QRCode)
            .style(theme::Button::Text)
            .padding(0)
            .on_press(Message::QRCode)
    }

    fn create_info_row<'note_view, 'qrcode_button: 'note_view>(
        &'note_view self,
        qrcode_button: button::Button<'qrcode_button, Message>,
    ) -> Row<'note_view, Message> {
        row![
            text(&self.note.created_at),
            text(&self.note.uuid4),
            text(format!("rowid {}", self.note.rowid)),
            qrcode_button
        ]
        .spacing(5)
    }

    fn create_tag_buttons<'note_view, 'row: 'note_view>(
        &'note_view self,
        row: Row<'row, Message>,
    ) -> iced_aw::Wrap<'note_view, Message, Horizontal> {
        self.tags
            .iter()
            .fold(iced_aw::Wrap::new().spacing(5.).push(row), |wrap, tag| {
                let tag_button = button(text(&tag.name))
                    .style(crate::style::Tag)
                    .padding(0)
                    .on_press(Message::Search(tag.name.to_owned()));
                wrap.push(tag_button)
            })
    }

    fn create_content_column<'note_view, 'wrap: 'note_view, 'qrcode: 'note_view>(
        &'note_view self,
        wrap: iced_aw::Wrap<'wrap, Message, Horizontal>,
        qrcode_widget: Option<QRCode<'qrcode>>,
    ) -> Column<'note_view, Message> {
        let mut column = column![wrap];
        if let Some(qrcode) = qrcode_widget {
            column = column.push(row![horizontal_space(), qrcode, horizontal_space()]);
        }
        column
    }

    fn add_note_details<'note_view, 'column: 'note_view, 'button: 'column>(
        &'note_view self,
        mut column: Column<'column, Message>,
        url_button: button::Button<'button, Message>,
    ) -> Column<'note_view, Message> {
        if !self.note.title.is_empty() {
            column = column.push(text(&self.note.title));
        }
        if !self.note.url.is_empty() {
            column = column.push(url_button);
        }
        if !self.note.description.is_empty() {
            column = column.push(text(&self.note.description));
        }
        if !self.note.comments.is_empty() {
            column = column.push(text(&self.note.comments));
        }
        column
    }

    fn add_delete_button<'note_view, 'column: 'note_view, 'button: 'note_view>(
        &'note_view self,
        column: Column<'column, Message>,
        delete_button: button::Button<'button, Message>,
    ) -> Column<'note_view, Message> {
        column.push(row![
            Space::with_width(FillPortion(12)),
            delete_button,
            Space::with_width(FillPortion(1))
        ])
    }

    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::OpenUrl => self.open_url(),
            Message::Delete(_) => println!("delete"),
            Message::QRCode => self.toggle_qrcode(),
            Message::Search(tag) => println!("search tag: {}", tag),
        }
    }

    fn open_url(&self) {
        if let Err(err) = open::that(&self.note.url) {
            println!("open url fail:{:?}", err);
        }
    }

    fn toggle_qrcode(&mut self) {
        match self.qrcode {
            Some(_) => self.qrcode.take(),
            None => self
                .qrcode
                .replace(iced::widget::qr_code::Data::new(self.note.url.as_bytes()).unwrap()),
        };
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
