use iced::{
    button, scrollable, text_input, Button, Column, Command, Container, Element, Row, Scrollable,
    Text, TextInput,
};

use crate::{
    config::Config,
    icons::IconItem,
    middle_date::MiddleDate,
    style::{self, Theme},
    tags::Tag,
    tr, Conn, DateView, NoteView, TagView,
};
#[derive(Default)]
pub struct SearchPage {
    pub notes: Vec<NoteView>,
    pub tags: Vec<TagView>,
    pub days: DateView,
    pub range: Option<(time::Date, time::Date)>,
    pub search_value: String,
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
    NoteMessage(crate::note::Message, usize),
    TagMessage(crate::tags::Message),
    DayMessage(crate::days::Message),
    Search,
    SearchInput(String),
    Clear,
    Refresh,
    NextPage,
    PrePage,
}
impl SearchPage {
    pub fn from_config(config: &Config) -> Self {
        Self {
            days: DateView::new(config),
            ..Default::default()
        }
    }
    pub fn new(count: u32, notes: Vec<NoteView>, tags: Vec<TagView>, days: DateView) -> Self {
        Self {
            notes,
            tags,
            days,
            count,
            ..Default::default()
        }
    }
    pub fn view(&mut self, theme: Theme, limit: u32) -> Element<Message> {
        let Self {
            notes,
            tags,
            days,
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
        let mut search_bar = Row::new().push(IconItem::Search).push(
            TextInput::new(
                input_state,
                &tr!("search"),
                &search_value,
                Message::SearchInput,
            )
            .on_submit(Message::Search),
        );
        if !self.search_value.is_empty() {
            search_bar = search_bar.push(
                Button::new(clear_button, IconItem::Clear)
                    .style(style::transparent(theme))
                    .padding(0)
                    .on_press(Message::Clear),
            );
        }
        let refresh_button = Button::new(refresh_button, IconItem::Refresh)
            .padding(0)
            .style(style::transparent(theme))
            .on_press(Message::Refresh);
        search_bar = search_bar.push(refresh_button);
        let tags = Scrollable::new(tags_scrollable)
            .push(Container::new(tags.iter_mut().fold(
                iced_aw::Wrap::new().spacing(5).push(Text::new(tr!("tags"))),
                |tags, tag| tags.push(tag.view(theme).map(Message::TagMessage)),
            )))
            .width(iced::Length::FillPortion(2));
        let days = Container::new(days.view(theme).map(Message::DayMessage))
            .height(iced::Length::Shrink)
            .padding(2)
            .max_height(240);
        let next_button = Button::new(next_button, IconItem::Next)
            .style(style::transparent(theme))
            .padding(0)
            .on_press(Message::NextPage);
        let pre_button = Button::new(pre_button, IconItem::Pre)
            .style(style::transparent(theme))
            .padding(0)
            .on_press(Message::PrePage);
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
        let note_page = if self.count > 0 {
            let notes = Container::new(
                notes.iter_mut().enumerate().fold(
                    Scrollable::new(notes_scrollable)
                        .padding(8)
                        .scroller_width(5)
                        .push(days),
                    |notes, (idx, note_view)| {
                        notes.push(
                            note_view
                                .view(theme)
                                .map(move |note_msg| Message::NoteMessage(note_msg, idx)),
                        )
                    },
                ),
            )
            .height(iced::Length::FillPortion(8));

            Column::new().push(search_bar).push(notes).push(page_ctrl)
        } else {
            let tip = if self.search_value.is_empty() && self.range.is_none() {
                tr!("nothing")
            } else {
                tr!("not-found")
            };
            let tip = Container::new(
                Column::new()
                    .push(style::vertical_rule())
                    .push(
                        Row::new()
                            .push(style::horizontal_rule())
                            .push(Text::new(tip).size(50))
                            .push(style::horizontal_rule()),
                    )
                    .push(style::vertical_rule()),
            )
            .height(iced::Length::FillPortion(8));
            Column::new()
                .push(search_bar)
                .push(Column::new().padding(8).push(days).push(tip))
                .push(page_ctrl)
        }
        .align_items(iced::Align::Center)
        .width(iced::Length::FillPortion(8));
        Container::new(Row::new().push(note_page).push(tags)).into()
    }
    pub fn update(
        &mut self,
        message: Message,
        limit: u32,
        conn: Conn,
        disabel_delete_tip: bool,
        delete_tip: &mut crate::DeleteTip,
    ) -> Command<crate::Message> {
        match message {
            Message::Search => search(
                conn,
                self.search_value.to_owned(),
                limit,
                self.offset,
                self.range,
            ),
            Message::SearchInput(search_value) => {
                self.search_value = search_value;
                search(
                    conn,
                    self.search_value.to_owned(),
                    limit,
                    self.offset,
                    self.range,
                )
            }
            Message::Clear => {
                self.search_value.clear();
                search(
                    conn,
                    self.search_value.to_owned(),
                    limit,
                    self.offset,
                    self.range,
                )
            }
            Message::Refresh => search(
                conn,
                self.search_value.to_owned(),
                limit,
                self.offset,
                self.range,
            ),
            Message::NextPage => {
                let current_count = self.offset + limit;
                if current_count < self.count {
                    self.offset = current_count;
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                } else {
                    Command::none()
                }
            }
            Message::PrePage => {
                if self.offset >= limit {
                    self.offset -= limit;
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                } else if self.offset != 0 {
                    self.offset = 0;
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                } else {
                    Command::none()
                }
            }
            Message::NoteMessage(msg, idx) => match msg {
                crate::note::Message::Delete(rowid) => {
                    if disabel_delete_tip {
                        Command::perform(
                            MiddleDate::delete(
                                conn,
                                self.search_value.to_string(),
                                limit,
                                self.offset,
                                rowid,
                            ),
                            crate::Message::Receiver,
                        )
                    } else {
                        delete_tip.tip_state.show(true);
                        delete_tip.rowid = rowid;
                        Command::none()
                    }
                }
                crate::note::Message::Search(s) => {
                    self.search_value = s;
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                }
                msg => {
                    if let Some(note) = self.notes.get_mut(idx) {
                        note.update(msg)
                    };
                    Command::none()
                }
            },
            Message::TagMessage(tag_msg) => {
                match tag_msg {
                    crate::tags::Message::Search(text) => self.search_value = text,
                }
                search(
                    conn,
                    self.search_value.to_owned(),
                    limit,
                    self.offset,
                    self.range,
                )
            }
            Message::DayMessage(dm) => match dm {
                crate::days::Message::DayOrMonth => {
                    self.days.day_or_month();
                    self.days.clear_cache_and_convert_selected_range();
                    Command::none()
                }

                crate::days::Message::ChartMsg(crate::days::ChartMsg::ClearRange) => {
                    self.days.clear_cahce();
                    self.days.chart.selected.take();
                    self.range.take();
                    if self.days.is_full {
                        match self.days.chart.level {
                            crate::days::ChartLevel::Day => {
                                self.days.chart.full_days = self.days.full_days;
                                self.days.chart.last_day = self.days.last_day;
                            }
                            crate::days::ChartLevel::Month => {
                                self.days.chart.full_months = self.days.full_months;
                                self.days.chart.last_month = self.days.last_month;
                            }
                        }
                    }
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                }
                crate::days::Message::ChartMsg(crate::days::ChartMsg::FilterSearch(selected)) => {
                    self.days.clear_cahce();
                    if matches!(self.days.chart.level, crate::days::ChartLevel::Month) {
                        let day_selected = selected
                            .months_to_days(self.days.chart.base_day, &self.days.chart.base_month);
                        self.days.preview_chart(day_selected);
                        self.days.preview_chart_update(day_selected);
                    }

                    let range = self.days.get_range(selected);
                    self.range = Some(range);
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                }
                crate::days::Message::PreviewChartMsg(crate::days::ChartMsg::FilterSearch(
                    selected,
                )) => {
                    self.days.preview_chart(selected);
                    self.days.preview_chart_update(selected);
                    let range = selected.get_days_range(self.days.preview_chart.base_day);
                    self.range = Some(range);
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                }
                dm => {
                    self.days.update(dm);
                    Command::none()
                }
            },
        }
    }
}

pub fn search(
    conn: Conn,
    query: String,
    limit: u32,
    offset: u32,
    range: Option<(time::Date, time::Date)>,
) -> Command<crate::Message> {
    if let Some((from, to)) = range {
        Command::perform(
            MiddleDate::from_filter(conn, query, limit, offset, from, to),
            crate::Message::Receiver,
        )
    } else {
        Command::perform(
            MiddleDate::from_select(conn, query, limit, offset),
            crate::Message::Receiver,
        )
    }
}

#[cfg(feature = "preview")]
impl iced::Sandbox for SearchPage {
    type Message = Message;

    fn new() -> Self {
        let count = 20;
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
        let days = DateView::default();
        Self {
            notes,
            tags,
            offset: 0,
            days,
            count,
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        "search page preview".to_owned()
    }

    fn update(&mut self, _message: Self::Message) {
        // TODO:
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.view(Theme::Light, 10)
    }
}
