use std::sync::Arc;

use iced::{
    button, futures::lock::Mutex, scrollable, text_input, Button, Column, Command, Container,
    Element, Row, Scrollable, Text, TextInput,
};
use localnative_core::rusqlite::Connection;

use crate::{
    middle_date::MiddleDate,
    style::{self, Theme},
    tags::Tag,
    NoteView, TagView,
};
#[derive(Default)]
pub struct SearchPage {
    pub notes: Vec<NoteView>,
    pub tags: Vec<TagView>,
    search_value: String,
    pub offset: u32,
    pub count: u32,
    input_state: text_input::State,
    clear_button: button::State,
    refresh_button: button::State,
    notes_scrollable: scrollable::State,
    tags_scrollable: scrollable::State,
    next_button: button::State,
    pre_button: button::State,
}
#[derive(Debug, Clone)]
pub enum Message {
    Receiver(Option<MiddleDate>),
    NoteMessage(crate::note::Message, usize),
    TagMessage(crate::tags::Message),
    Search,
    SearchInput(String),
    Clear,
    Refresh,
    NextPage,
    PrePage,
}
impl SearchPage {
    pub fn new(count: u32, notes: Vec<NoteView>, tags: Vec<TagView>) -> Self {
        Self {
            count,
            notes,
            tags,
            ..Default::default()
        }
    }
    pub fn view(&mut self, theme: Theme, limit: u32) -> Element<Message> {
        let Self {
            notes,
            tags,
            search_value,
            input_state,
            clear_button,
            refresh_button,
            notes_scrollable,
            tags_scrollable,
            next_button,
            pre_button,
            ..
        } = self;
        let mut search_bar = Row::new().push(
            TextInput::new(
                input_state,
                "Type your search...",
                &search_value,
                Message::SearchInput,
            )
            .on_submit(Message::Search),
        );
        if !self.search_value.is_empty() {
            search_bar =
                search_bar.push(Button::new(clear_button, Text::new("X")).on_press(Message::Clear));
        }
        let refresh_button = Button::new(refresh_button, Text::new("O")).on_press(Message::Refresh);
        search_bar = search_bar.push(refresh_button);
        let tags = Scrollable::new(tags_scrollable)
            .push(Container::new(tags.iter_mut().fold(
                iced_aw::Wrap::new().spacing(5).push(Text::new("tags:")),
                |tags, tag| tags.push(tag.view(theme).map(Message::TagMessage)),
            )))
            .width(iced::Length::FillPortion(2));
        let note_page = if self.count > 0 {
            let notes = Container::new(notes.iter_mut().enumerate().fold(
                Scrollable::new(notes_scrollable).padding(10),
                |notes, (idx, note_view)| {
                    notes.push(
                        note_view
                            .view(theme)
                            .map(move |note_msg| Message::NoteMessage(note_msg, idx)),
                    )
                },
            ))
            .height(iced::Length::Fill);
            let next_button = Button::new(next_button, Text::new("->")).on_press(Message::NextPage);
            let pre_button = Button::new(pre_button, Text::new("<-")).on_press(Message::PrePage);
            let page_info = Text::new(format!(
                "{}-{}/{}",
                self.offset + 1,
                (self.offset + limit).min(self.count),
                self.count
            ));
            let page_ctrl = Row::new()
                .push(style::horizontal_rule())
                .push(pre_button)
                .push(page_info)
                .push(next_button)
                .push(style::horizontal_rule());

            Column::new().push(search_bar).push(notes).push(page_ctrl)
        } else {
            let tip = if self.search_value.is_empty() {
                "Not Created"
            } else {
                "Not Founded"
            };
            Column::new()
                .push(search_bar)
                .push(style::vertical_rule())
                .push(Text::new(tip).size(50))
                .push(style::vertical_rule())
        }
        .align_items(iced::Align::Center)
        .width(iced::Length::FillPortion(8));
        Container::new(Row::new().push(note_page).push(tags)).into()
    }
    pub fn update(
        &mut self,
        message: Message,
        limit: u32,
        conn: Arc<Mutex<Connection>>,
    ) -> Command<Message> {
        match message {
            Message::Search => search(conn, self.search_value.to_owned(), limit, self.offset),
            Message::SearchInput(search_value) => {
                self.search_value = search_value;
                search(conn, self.search_value.to_owned(), limit, self.offset)
            }
            Message::Clear => {
                self.search_value.clear();
                search(conn, self.search_value.to_owned(), limit, self.offset)
            }
            Message::Refresh => search(conn, self.search_value.to_owned(), limit, self.offset),
            Message::NextPage => {
                let current_count = self.offset + limit;
                if current_count < self.count {
                    self.offset = current_count;
                    search(conn, self.search_value.to_owned(), limit, self.offset)
                } else {
                    Command::none()
                }
            }
            Message::PrePage => {
                if self.offset >= limit {
                    self.offset -= limit;
                    search(conn, self.search_value.to_owned(), limit, self.offset)
                } else if self.offset != 0 {
                    self.offset = 0;
                    search(conn, self.search_value.to_owned(), limit, self.offset)
                } else {
                    Command::none()
                }
            }
            Message::NoteMessage(msg, idx) => match msg {
                crate::note::Message::Delete(rowid) => Command::perform(
                    MiddleDate::delete(
                        conn,
                        self.search_value.to_string(),
                        limit,
                        self.offset,
                        rowid,
                    ),
                    Message::Receiver,
                ),
                crate::note::Message::Search(s) => {
                    self.search_value = s;
                    search(conn, self.search_value.to_owned(), limit, self.offset)
                }
                msg => {
                    if let Some(note) = self.notes.get_mut(idx) {
                        note.update(msg)
                    };
                    Command::none()
                }
            },
            // 上层处理
            Message::Receiver(_) => Command::none(),
            Message::TagMessage(tag_msg) => {
                match tag_msg {
                    crate::tags::Message::Search(text) => self.search_value = text,
                }
                search(conn, self.search_value.to_owned(), limit, self.offset)
            }
        }
    }
}

fn search(
    conn: Arc<Mutex<Connection>>,
    query: String,
    limit: u32,
    offset: u32,
) -> Command<Message> {
    Command::perform(
        MiddleDate::from_select(conn, query, limit, offset),
        Message::Receiver,
    )
}

#[cfg(feature = "preview")]
impl iced::Sandbox for SearchPage {
    type Message = Message;

    fn new() -> Self {
        let count = 0;
        let mut notes = Vec::with_capacity(count as usize);
        for _ in 0..count {
            notes.push(NoteView::new());
        }
        let tags = vec![
            Tag {
                name: "testtag".to_owned(),
                count: 16,
            };
            50
        ]
        .into_iter()
        .map(TagView::from)
        .collect();
        Self {
            notes,
            tags,
            offset: 0,
            count,
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        "search page preview".to_owned()
    }

    fn update(&mut self, message: Self::Message) {
        let conn = localnative_core::exe::get_sqlite_connection();
        self.update(message, 10, Arc::new(Mutex::new(conn)));
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.view(Theme::Light, 10)
    }
}
