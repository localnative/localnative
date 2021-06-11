mod days;
mod middle_date;
mod note;
mod search_page;
mod style;
mod tags;

use std::{array::IntoIter, sync::Arc};

use iced::{futures::lock::Mutex, Column, Command, Row, Text};
use localnative_core::{exe::get_sqlite_connection, rusqlite::Connection};
use middle_date::MiddleDate;
pub use note::NoteView;
pub use search_page::SearchPage;
use style::Theme;
pub use tags::TagView;

pub enum LocalNative {
    Loading,
    Loaded(Data),
}

pub struct Data {
    search_page: SearchPage,
    conn: Arc<Mutex<Connection>>,
    theme: Theme,
    limit: u32,
}

#[derive(Debug)]
pub enum Message {
    Loading(()),
    SearchPageMessage(search_page::Message),
    NoteView(Vec<NoteView>),
    TagView(Vec<TagView>),
}

impl iced::Application for LocalNative {
    type Executor = iced::executor::Default;

    type Message = Message;

    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            LocalNative::Loading,
            Command::perform(async {}, Message::Loading),
        )
    }

    fn title(&self) -> String {
        "ln-iced".to_owned()
    }

    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        match self {
            LocalNative::Loading => match message {
                Message::Loading(..) => {
                    let conn = Arc::new(Mutex::new(get_sqlite_connection()));
                    let data = Data {
                        search_page: Default::default(),
                        conn,
                        theme: Theme::Light,
                        limit: 10,
                    };

                    *self = LocalNative::Loaded(data);
                    if let LocalNative::Loaded(data) = self {
                        data.search_page
                            .update(search_page::Message::Refresh, data.limit, data.conn.clone())
                            .map(Message::SearchPageMessage)
                    } else {
                        unreachable!()
                    }
                }
                _ => Command::none(),
            },
            LocalNative::Loaded(data) => match message {
                Message::SearchPageMessage(search_page_msg) => match search_page_msg {
                    search_page::Message::Receiver(Some(md)) => {
                        let MiddleDate {
                            tags,
                            notes,
                            count,
                            days,
                        } = md;
                        data.search_page.count = count;
                        Command::batch(IntoIter::new([
                            Command::perform(
                                async move {
                                    let mut tags = tags;
                                    tags.sort_by(|a, b| b.count.cmp(&a.count));
                                    tags.into_iter().map(TagView::from).collect()
                                },
                                Message::TagView,
                            ),
                            Command::perform(
                                async move { notes.into_iter().map(NoteView::from).collect() },
                                Message::NoteView,
                            ),
                        ]))
                    }
                    msg => data
                        .search_page
                        .update(msg, data.limit, data.conn.clone())
                        .map(Message::SearchPageMessage),
                },
                Message::NoteView(notes) => {
                    data.search_page.notes = notes;
                    Command::none()
                }
                Message::TagView(tags) => {
                    data.search_page.tags = tags;
                    Command::none()
                }
                Message::Loading(..) => Command::none(),
            },
        }
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        match self {
            LocalNative::Loading => Column::new()
                .push(style::vertical_rule())
                .push(
                    Row::new()
                        .push(style::horizontal_rule())
                        .push(Text::new("Loading...").size(50))
                        .push(style::horizontal_rule()),
                )
                .push(style::vertical_rule())
                .into(),
            LocalNative::Loaded(data) => {
                let Data { search_page, .. } = data;
                search_page
                    .view(data.theme, data.limit)
                    .map(Message::SearchPageMessage)
            }
        }
    }
}

pub fn settings() -> iced::Settings<()> {
    iced::Settings {
        ..Default::default()
    }
}
