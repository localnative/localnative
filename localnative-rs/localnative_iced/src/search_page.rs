use iced::{
    theme,
    widget::{
        button, column, container, horizontal_space, row, scrollable, scrollable::Properties, text,
        text_input, vertical_space,
    },
    Command, Element,
    Length::Fill,
};

use crate::{
    config::ThemeType, icons::IconItem, middle_date::MiddleDate, tr, Conn, DateView, NoteView,
    TagView,
};

#[cfg(feature = "preview")]
use crate::tags::Tag;

#[derive(Default)]
pub struct SearchPage {
    pub notes: Vec<NoteView>,
    pub tags: Vec<TagView>,
    pub days: DateView,
    pub range: Option<(time::Date, time::Date)>,
    pub search_value: String,
    pub offset: u32,
    pub count: u32,
}
#[derive(Debug, Clone)]
pub enum Message {
    Note(crate::note::Message, usize),
    Tag(crate::tags::Message),
    Day(crate::days::Message),
    Search,
    SearchInput(String),
    Clear,
    Refresh,
    NextPage,
    PrePage,
}
impl SearchPage {
    pub fn default_with_theme(theme: ThemeType) -> Self {
        Self {
            days: DateView::new(theme),
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
    pub fn view(&self, limit: u32) -> Element<Message> {
        let Self {
            notes,
            tags,
            days,
            search_value,
            ..
        } = self;
        let search_icon = iced::widget::text_input::Icon {
            font: crate::icons::ICONS,
            code_point: '\u{f0d1}',
            spacing: 8.5,
            side: iced::widget::text_input::Side::Left,
            size: Some(22.0),
        };

        let mut search_bar = row![text_input(&tr!("search"), search_value)
            .icon(search_icon)
            .on_input(Message::SearchInput)
            .on_submit(Message::Search)];

        if !self.search_value.is_empty() {
            search_bar = search_bar.push(
                button(IconItem::Clear)
                    .style(theme::Button::Text)
                    .padding(0)
                    .on_press(Message::Clear),
            );
        }
        let refresh_button = button(IconItem::Refresh)
            .style(theme::Button::Text)
            .padding(0)
            .on_press(Message::Refresh);

        search_bar = search_bar.push(refresh_button);

        let tags = container(scrollable(tags.iter().fold(
            iced_aw::Wrap::new().spacing(5.).push(text(tr!("tags"))),
            |tags, tag| tags.push(tag.view().map(Message::Tag)),
        )))
        .width(iced::Length::FillPortion(2));

        let days = container(days.view().map(Message::Day)).padding(2).height({
            if days.is_show {
                iced::Length::Fixed(256.)
            } else {
                iced::Length::Shrink
            }
        });
        let next_button = button(IconItem::Next)
            .style(theme::Button::Text)
            .padding(0)
            .on_press(Message::NextPage);
        let pre_button = button(IconItem::Pre)
            .style(theme::Button::Text)
            .padding(0)
            .on_press(Message::PrePage);
        let page_info = text(format!(
            "{}-{}/{}",
            self.offset + 1,
            (self.offset + limit).min(self.count),
            self.count
        ));
        let page_ctrl = row![
            horizontal_space(Fill),
            pre_button,
            page_info,
            next_button,
            horizontal_space(Fill)
        ];

        let note_page = if self.count > 0 {
            let notes = container(
                scrollable(
                    notes
                        .iter()
                        .enumerate()
                        .fold(
                            iced_aw::Wrap::new_vertical().spacing(5.).push(days),
                            |notes, (idx, note_view)| {
                                notes.push(
                                    note_view
                                        .view()
                                        .map(move |note_msg| Message::Note(note_msg, idx)),
                                )
                            },
                        )
                        .padding(12.),
                )
                .vertical_scroll(Properties::new().width(10).scroller_width(10)),
            )
            .height(iced::Length::FillPortion(8));

            column![search_bar, notes, page_ctrl]
        } else {
            let tip = if self.search_value.is_empty() && self.range.is_none() {
                tr!("nothing")
            } else {
                tr!("not-found")
            };
            let tip = container(column![
                vertical_space(Fill),
                row![
                    horizontal_space(Fill),
                    text(tip).size(50),
                    horizontal_space(Fill)
                ],
                vertical_space(Fill)
            ])
            .height(iced::Length::FillPortion(8));
            column![
                search_bar,
                container(column![days, tip]).padding(12.),
                page_ctrl
            ]
        }
        .align_items(iced::Alignment::Center)
        .width(iced::Length::FillPortion(8));

        container(row![note_page, tags]).into()
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
            Message::Note(msg, idx) => match msg {
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
                        delete_tip.show_modal = true;
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
            Message::Tag(tag_msg) => {
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
            Message::Day(dm) => match dm {
                crate::days::Message::Clear => search(
                    conn,
                    self.search_value.to_owned(),
                    limit,
                    self.offset,
                    self.range,
                ),
                crate::days::Message::Selected { start, end } => {
                    self.range = Some((start, end));
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

    fn view(&self) -> Element<'_, Self::Message> {
        self.view(10)
    }
}
