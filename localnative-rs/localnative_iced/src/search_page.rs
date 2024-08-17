use iced::widget::{scrollable, Column, Row};
use iced::{
    theme,
    widget::{
        button, column, container, horizontal_space, row, scrollable::Properties, text, text_input,
        vertical_space,
    },
    Command, Element, Pixels,
};
use sqlx::SqlitePool;

use crate::db_operations;
use crate::{config::ThemeType, icons::IconItem, tr, DateView, NoteView, TagView};

#[derive(Default)]
pub struct SearchPage {
    pub notes: Vec<NoteView>,
    pub tags: Vec<TagView>,
    pub days: DateView,
    pub range: Option<(chrono::NaiveDate, chrono::NaiveDate)>,
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
        let tags = self.create_tags_container();
        let days = self.create_days_container();

        let note_page = if self.count > 0 {
            self.create_note_page(days, limit)
        } else {
            self.create_empty_page(days)
        }
        .align_items(iced::Alignment::Center)
        .width(iced::Length::FillPortion(8));

        container(row![note_page, tags]).into()
    }

    fn create_search_bar(&self) -> Row<Message> {
        let search_icon = iced::widget::text_input::Icon {
            font: crate::icons::ICONS,
            code_point: '\u{f0d1}',
            spacing: 8.5,
            side: iced::widget::text_input::Side::Left,
            size: Some(Pixels(22.0)),
        };

        let mut search_bar = row![text_input(&tr!("search"), &self.search_value)
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

        search_bar = search_bar.push(
            button(IconItem::Refresh)
                .style(theme::Button::Text)
                .padding(0)
                .on_press(Message::Refresh),
        );

        search_bar
    }

    fn create_tags_container(&self) -> container::Container<Message> {
        container(scrollable(self.tags.iter().fold(
            iced_aw::Wrap::new().spacing(5.).push(text(tr!("tags"))),
            |tags, tag| tags.push(tag.view().map(Message::Tag)),
        )))
        .width(iced::Length::FillPortion(2))
    }

    fn create_days_container(&self) -> container::Container<Message> {
        container(self.days.view().map(Message::Day))
            .padding(2)
            .height({
                if self.days.is_show {
                    iced::Length::Fixed(256.)
                } else {
                    iced::Length::Fixed(36.5)
                }
            })
    }

    fn create_page_control(&self, limit: u32) -> Row<Message> {
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

        row![
            horizontal_space(),
            pre_button,
            page_info,
            next_button,
            horizontal_space()
        ]
    }

    fn create_note_page<'search_page, 'container: 'search_page>(
        &'search_page self,
        days: container::Container<'container, Message>,
        limit: u32,
    ) -> Column<'search_page, Message> {
        let notes = container(
            scrollable(
                self.notes
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
            .direction(scrollable::Direction::Vertical(
                Properties::new().width(10).scroller_width(10),
            )),
        )
        .height(iced::Length::FillPortion(8));

        column![
            self.create_search_bar(),
            notes,
            self.create_page_control(limit)
        ]
    }

    fn create_empty_page<'search_page, 'container: 'search_page>(
        &'search_page self,
        days: container::Container<'container, Message>,
    ) -> Column<'search_page, Message> {
        let tip = if self.search_value.is_empty() && self.range.is_none() {
            tr!("nothing")
        } else {
            tr!("not-found")
        };
        let tip = container(column![
            vertical_space(),
            row![horizontal_space(), text(tip).size(50), horizontal_space()],
            vertical_space()
        ])
        .height(iced::Length::FillPortion(8));

        column![
            self.create_search_bar(),
            container(column![days, tip]).padding(12.),
            self.create_page_control(10)
        ]
        .align_items(iced::Alignment::Center)
        .width(iced::Length::FillPortion(8))
    }

    pub fn update(
        &mut self,
        message: Message,
        limit: u32,
        pool: &SqlitePool,
        disabel_delete_tip: bool,
        delete_tip: &mut crate::DeleteTip,
    ) -> Command<crate::Message> {
        match message {
            Message::Search => self.handle_search(pool, limit),
            Message::SearchInput(search_value) => {
                self.search_value = search_value;
                self.handle_search(pool, limit)
            }
            Message::Clear => {
                self.search_value.clear();
                self.handle_search(pool, limit)
            }
            Message::Refresh => self.handle_search(pool, limit),
            Message::NextPage => self.handle_next_page(pool, limit),
            Message::PrePage => self.handle_pre_page(pool, limit),
            Message::Note(msg, idx) => {
                self.handle_note_message(msg, idx, pool, limit, disabel_delete_tip, delete_tip)
            }
            Message::Tag(tag_msg) => self.handle_tag_message(tag_msg, pool, limit),
            Message::Day(dm) => self.handle_day_message(dm, pool, limit),
        }
    }

    fn handle_search(&self, pool: &SqlitePool, limit: u32) -> Command<crate::Message> {
        search(
            pool,
            self.search_value.to_owned(),
            limit,
            self.offset,
            self.range,
        )
    }

    fn handle_next_page(&mut self, pool: &SqlitePool, limit: u32) -> Command<crate::Message> {
        let current_count = self.offset + limit;
        if current_count < self.count {
            self.offset = current_count;
            self.handle_search(pool, limit)
        } else {
            Command::none()
        }
    }

    fn handle_pre_page(&mut self, pool: &SqlitePool, limit: u32) -> Command<crate::Message> {
        if self.offset >= limit {
            self.offset -= limit;
            self.handle_search(pool, limit)
        } else if self.offset != 0 {
            self.offset = 0;
            self.handle_search(pool, limit)
        } else {
            Command::none()
        }
    }

    fn handle_note_message(
        &mut self,
        msg: crate::note::Message,
        idx: usize,
        pool: &SqlitePool,
        limit: u32,
        disabel_delete_tip: bool,
        delete_tip: &mut crate::DeleteTip,
    ) -> Command<crate::Message> {
        match msg {
            crate::note::Message::Delete(rowid) => {
                if disabel_delete_tip {
                    Command::perform(
                        db_operations::delete(
                            pool.clone(),
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
                self.handle_search(pool, limit)
            }
            msg => {
                if let Some(note) = self.notes.get_mut(idx) {
                    note.update(msg)
                };
                Command::none()
            }
        }
    }

    fn handle_tag_message(
        &mut self,
        tag_msg: crate::tags::Message,
        pool: &SqlitePool,
        limit: u32,
    ) -> Command<crate::Message> {
        match tag_msg {
            crate::tags::Message::Search(text) => self.search_value = text,
        }
        self.handle_search(pool, limit)
    }

    fn handle_day_message(
        &mut self,
        dm: crate::days::Message,
        pool: &SqlitePool,
        limit: u32,
    ) -> Command<crate::Message> {
        match dm {
            crate::days::Message::Clear => self.handle_search(pool, limit),
            crate::days::Message::Selected { start, end } => {
                self.range = Some((start, end));
                self.handle_search(pool, limit)
            }
            dm => {
                self.days.update(dm);
                Command::none()
            }
        }
    }
}

pub fn search(
    pool: &SqlitePool,
    query: String,
    limit: u32,
    offset: u32,
    range: Option<(chrono::NaiveDate, chrono::NaiveDate)>,
) -> Command<crate::Message> {
    if let Some((from, to)) = range {
        Command::perform(
            db_operations::filter(pool.clone(), query, limit, offset, from, to),
            crate::Message::Receiver,
        )
    } else {
        Command::perform(
            db_operations::select(pool.clone(), query, limit, offset),
            crate::Message::Receiver,
        )
    }
}
