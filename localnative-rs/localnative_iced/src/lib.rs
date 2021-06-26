mod days;
mod middle_date;
mod note;
mod search_page;
mod style;
mod tags;

use std::sync::Arc;

pub use days::Chart;
pub use days::DateView;
use days::HandleDays;
use iced::{futures::lock::Mutex, Column, Command, Row, Text, Vector};
use localnative_core::{exe::get_sqlite_connection, rusqlite::Connection, Note};
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
    DayView(HandleDays),
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

                        Command::batch([
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
                            {
                                if let Some(days) = days {
                                    Command::perform(
                                        async move { days::Day::handle_days(days) },
                                        Message::DayView,
                                    )
                                } else {
                                    Command::none()
                                }
                            },
                        ])
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
                Message::DayView(HandleDays {
                    days,
                    months,
                    max_day_count,
                    max_month_count,
                    full_days,
                    full_months,
                    last_day,
                    last_month,
                }) => {
                    data.search_page.days.days = days;
                    data.search_page.days.months = months;
                    data.search_page.days.full_days = full_days;
                    data.search_page.days.full_months = full_months;
                    data.search_page.days.last_day = last_day;
                    data.search_page.days.last_month = last_month;
                    data.search_page.days.chart.last_day = last_day;
                    data.search_page.days.chart.last_month = last_month;
                    data.search_page.days.chart.max_day_count = max_day_count;
                    data.search_page.days.chart.max_month_count = max_month_count;
                    if data.search_page.range.is_none() {
                        data.search_page.days.align();
                    }
                    data.search_page.days.clear_cahce();
                    Command::none()
                }
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

pub fn handle_notes(notes: Vec<Note>) -> (Vec<NoteView>) {
    for note in notes {
        let time = note.created_at;
    }
    todo!()
}
