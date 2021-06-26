use std::convert::TryFrom;

use iced::{
    button,
    canvas::{self, Cache, Cursor, Fill, Frame, Geometry, Program},
    mouse, Button, Canvas, Color, Column, Container, Element, Point, Rectangle, Row, Size, Text,
    Vector,
};

use iced_aw::number_input;
use serde::{Deserialize, Serialize};
use time::{macros::date, Date, Duration, Month};

use crate::style::{self, Theme};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Day {
    #[serde(rename = "k")]
    pub date: Date,
    #[serde(rename = "v")]
    pub count: i64,
}

impl Default for Day {
    fn default() -> Self {
        Self {
            date: time::macros::date!(2021 - 06 - 24),
            count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonthView {
    pub month: Month,
    pub year: i32,
    pub count: i64,
}

pub struct DateChart<'a> {
    pub days: &'a [Day],
    pub months: &'a [MonthView],
    pub cache: &'a Cache,
    pub selected_cache: &'a Cache,
    pub selected: Option<&'a Selected>,
    pub pending: Option<Point>,
    pub maximal_day_count: i64,
    pub maximal_month_count: i64,
    pub day_uw: f32,
    pub month_uw: f32,
    pub full_days: Option<i64>,
    pub full_months: Option<i32>,
    pub last_day: Option<i64>,
    pub last_month: Option<i32>,
    pub level: ChartLevel,
    pub style: Style,
    pub translation: Vector,
    pub base_day: Date,
    pub base_month: &'a MonthView,
}

impl<'a> DateChart<'a> {
    fn init(
        &self,
        size: Size,
        translation: &mut Vector,
        last_day: Option<i64>,
        last_month: Option<i32>,
        selected: &Option<&Selected>,
    ) -> f32 {
        match self.level {
            ChartLevel::Day => {
                if let Some(num) = self.full_days {
                    let uw = (size.width as i64 / (num + 3)).max(1).min(50);
                    if let Some(s) = selected {
                        *translation = Vector::new(
                            ((s.end.max(1) - 1) * uw as usize).max(0).min(10000) as f32,
                            0.0,
                        )
                    } else {
                        last_day.map(|last| {
                            *translation =
                                Vector::new(((last - 1).max(0) * uw).max(0).min(10000) as f32, 0.0)
                        });
                    }
                    uw as f32
                } else {
                    self.day_uw
                }
            }
            ChartLevel::Month => {
                if let Some(num) = self.full_months {
                    let uw = (size.width as i32 / (num + 3)).max(1).min(50);
                    if let Some(s) = selected {
                        *translation = Vector::new(
                            ((s.end.max(1) - 1) * uw as usize).max(0).min(10000) as f32,
                            0.0,
                        )
                    } else {
                        last_month.map(|last| {
                            *translation =
                                Vector::new(((last - 1) * uw).max(0).min(10000) as f32, 0.0)
                        });
                    }
                    uw as f32
                } else {
                    self.month_uw
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ChartLevel {
    Day,
    Month,
}

impl Default for ChartLevel {
    fn default() -> Self {
        Self::Day
    }
}

pub struct Style {
    fill_color: Color,
    font_size: f32,
    big_font_size: f32,
}
pub fn month_to_num(month: Month) -> i32 {
    match month {
        Month::January => 1,
        Month::February => 2,
        Month::March => 3,
        Month::April => 4,
        Month::May => 5,
        Month::June => 6,
        Month::July => 7,
        Month::August => 8,
        Month::September => 9,
        Month::October => 10,
        Month::November => 11,
        Month::December => 12,
    }
}
pub fn base_month() -> MonthView {
    let today = time::OffsetDateTime::now_utc().date();
    let tomorrow = today.next_day().unwrap();
    let tomorrow_month = tomorrow.month();
    if tomorrow_month != today.month() {
        MonthView {
            month: tomorrow_month,
            year: tomorrow.year(),
            count: 0,
        }
    } else {
        let year = if tomorrow_month == Month::December {
            tomorrow.year() + 1
        } else {
            tomorrow.year()
        };
        MonthView {
            month: tomorrow_month.next(),
            year,
            count: 0,
        }
    }
}
pub fn base_day() -> Date {
    time::OffsetDateTime::now_utc().date().next_day().unwrap()
}
impl Day {
    pub fn draw_all_day(
        days: &[Day],
        frame: &mut Frame,
        max_count: i64,
        fill_color: Color,
        font_size: f32,
        big_font_size: f32,
        uw: f32,
        translation: f32,
        base_day: Date,
        skip_text: bool,
    ) {
        // 绘制文本
        let Size { width, height } = frame.size();
        let num = (width / uw).max(0.0).min(10000.0) as i64;
        let overflow_num = (translation / uw).max(0.0).min(10000.0) as i64;
        let rightmost_day = base_day - Duration::days(overflow_num + 1);

        let mut date_pointer = rightmost_day;
        let day_text_y = height - font_size;
        let gt9_offset = 0.5 * (uw - font_size);
        let lt9_offset = 0.5 * (uw - 0.5 * font_size);

        for idx in 1..=num {
            let day = date_pointer.day();
            let x = width - idx as f32 * uw - translation;
            if !skip_text {
                if idx == num || (date_pointer.month() == Month::January && date_pointer.day() == 1)
                {
                    let year = canvas::Text {
                        content: date_pointer.year().to_string(),
                        position: Point::new(x, 0.0),
                        size: big_font_size,
                        ..Default::default()
                    };
                    frame.fill_text(year);
                }
                if day == 1 || idx == num {
                    let month = canvas::Text {
                        content: date_pointer.month().to_string(),
                        position: Point::new(x, big_font_size),
                        size: big_font_size,
                        ..Default::default()
                    };
                    frame.fill_text(month);
                }
                if uw >= font_size * 1.8 {
                    let day_text_offset = if day > 9 { gt9_offset } else { lt9_offset };
                    let position = Point::new(x + day_text_offset, day_text_y);
                    let day = canvas::Text {
                        content: day.to_string(),
                        position,
                        size: font_size,
                        ..Default::default()
                    };
                    frame.fill_text(day);
                }
            }
            if let Some(pd) = date_pointer.previous_day() {
                date_pointer = pd;
            }
        }
        // 绘制方块
        let uh = height / max_count as f32;
        for day in days {
            if day.date > rightmost_day {
                break;
            }
            if day.date < date_pointer {
                continue;
            }

            let num = (base_day - day.date).whole_days().max(0).min(10000) as f32;
            let x = width - num * uw;
            let dh = uh * day.count as f32;
            if uw >= font_size * 1.8 && (height - dh < day_text_y - font_size) {
                let count_offset = if day.count > 9 {
                    gt9_offset
                } else {
                    lt9_offset
                };
                let count = canvas::Text {
                    content: day.count.to_string(),
                    position: Point::new(x + count_offset, height - dh),
                    size: font_size,
                    ..Default::default()
                };
                frame.fill_text(count);
            }

            frame.fill_rectangle(
                Point::new(x, height - dh),
                Size::new(uw, dh),
                Fill::from(fill_color),
            );
        }
    }

    pub fn handle_days(days: Vec<Day>) -> HandleDays {
        let len = days.len();

        if len == 0 {
            return HandleDays::default();
        }

        let mut monthviews = Vec::new();
        let mut max_day_count = 0;
        let mut max_month_count = 0;
        let mut monthcount_temp = 0;
        let (mut pre_year, mut pre_month, _) = days.first().unwrap().date.to_calendar_date();

        for day in days.iter() {
            if day.date.year() != pre_year || day.date.month() != pre_month {
                if monthcount_temp != 0 {
                    max_month_count = max_month_count.max(monthcount_temp);
                    monthviews.push(MonthView {
                        month: pre_month,
                        year: pre_year,
                        count: monthcount_temp,
                    });
                    monthcount_temp = day.count;
                } else {
                    monthcount_temp += day.count;
                }
                pre_year = day.date.year();
                pre_month = day.date.month();
            } else {
                monthcount_temp += day.count;
            }

            max_day_count = max_day_count.max(day.count);
        }
        if monthcount_temp != 0 {
            max_month_count = max_month_count.max(monthcount_temp);

            monthviews.push(MonthView {
                month: pre_month,
                year: pre_year,
                count: monthcount_temp,
            });
        }
        let full_days = days.last().and_then(|last| {
            days.first()
                .map(|first| (last.date - first.date).whole_days())
        });
        let full_months = monthviews
            .last()
            .and_then(|last| monthviews.first().map(|first| months_num(first, last)));
        let last_day = days.last().map(|day| (base_day() - day.date).whole_days());
        let last_month = monthviews
            .last()
            .map(|month| months_num(month, &base_month()));
        HandleDays {
            days,
            months: monthviews,
            max_day_count,
            max_month_count,
            full_days,
            full_months,
            last_day,
            last_month,
        }
    }
}
#[derive(Debug, Default, Clone)]
pub struct HandleDays {
    pub days: Vec<Day>,
    pub months: Vec<MonthView>,
    pub max_day_count: i64,
    pub max_month_count: i64,
    pub full_days: Option<i64>,
    pub full_months: Option<i32>,
    pub last_day: Option<i64>,
    pub last_month: Option<i32>,
}
impl MonthView {
    pub fn from_date(date: Date, count: i64) -> Self {
        Self {
            year: date.year(),
            month: date.month(),
            count,
        }
    }
    pub fn draw_all_month(
        months: &[MonthView],
        frame: &mut Frame,
        max_count: i64,
        fill_color: Color,
        font_size: f32,
        big_font_size: f32,
        uw: f32,
        translation: f32,
        base_month: &MonthView,
    ) {
        // 绘制文本
        let Size { width, height } = frame.size();
        let num = (width / uw).min(10000.0).max(0.0) as i64;
        let MonthView {
            mut month,
            mut year,
            ..
        } = base_month;
        let mut offset = (translation / uw) as i64;

        year -= offset as i32 / 12;
        offset = offset % 12;

        if offset != 0 {
            let mut m_num = month_to_num(month) - offset as i32;
            if m_num <= 0 {
                m_num += 12;
                year -= 1;
            }
            month = match m_num {
                1 => Month::January,
                2 => Month::February,
                3 => Month::March,
                4 => Month::April,
                5 => Month::May,
                6 => Month::June,
                7 => Month::July,
                8 => Month::August,
                9 => Month::September,
                10 => Month::October,
                11 => Month::November,
                12 => Month::December,
                _ => unreachable!(),
            };
        }

        let rightmost_month = (year, month);

        let mut date_pointer = rightmost_month;

        let month_text_y = height - font_size;
        let gt9_offset = 0.5 * (uw - font_size);
        let lt9_offset = 0.5 * (uw - 0.5 * font_size);

        for idx in 1..=num {
            let month = date_pointer;
            let x = width - idx as f32 * uw - translation;
            if idx == num || date_pointer.1 == Month::January {
                let year = canvas::Text {
                    content: date_pointer.0.to_string(),
                    position: Point::new(x, 0.0),
                    size: big_font_size,
                    ..Default::default()
                };
                frame.fill_text(year);
            }

            let content = month_to_num(month.1);
            if uw >= font_size * 1.8 {
                let month_text_offset = if content > 9 { gt9_offset } else { lt9_offset };
                let position = Point::new(x + month_text_offset, month_text_y);

                let month = canvas::Text {
                    content: content.to_string(),
                    position,
                    size: font_size,
                    ..Default::default()
                };

                frame.fill_text(month);
            }

            if let Month::January = date_pointer.1 {
                date_pointer = (date_pointer.0 - 1, Month::December);
            } else {
                date_pointer.1 = date_pointer.1.previous();
            }
        }
        // 绘制方块
        let uh = height / max_count as f32;
        let rm_num = month_to_num(rightmost_month.1);
        let dm_num = month_to_num(date_pointer.1);
        let MonthView {
            month: tm,
            year: ty,
            ..
        } = base_month;
        let tm_num = month_to_num(*tm);
        for month in months {
            let cy = month.year;
            let cm = month.month;
            let cm_num = month_to_num(cm);
            if cy > rightmost_month.0 || (cy == rightmost_month.0 && cm_num > rm_num) {
                break;
            }

            if (cy == date_pointer.0 && cm_num < dm_num) || (cy < date_pointer.0) {
                continue;
            }
            let num = (*ty - cy - 1) * 12 + 13 + tm_num - cm_num;
            let x = width - num as f32 * uw;

            let dh = uh * month.count as f32;
            let content = month.count.to_string();
            if font_size * content.len() as f32 <= uw && height - dh < month_text_y - font_size {
                let count_offset = if month.count > 9 {
                    gt9_offset
                } else {
                    lt9_offset
                };
                let count = canvas::Text {
                    content,
                    position: Point::new(x + count_offset, height - dh),
                    size: font_size,
                    ..Default::default()
                };
                frame.fill_text(count);
            }
            frame.fill_rectangle(
                Point::new(x, height - dh),
                Size::new(uw, dh),
                Fill::from(fill_color),
            );
        }
    }
}

pub fn months_num(start: &MonthView, end: &MonthView) -> i32 {
    (end.year - start.year - 1) * 12 + 13 + month_to_num(end.month) - month_to_num(start.month)
}

#[derive(Clone, Debug)]
pub enum ChartMsg {
    ZoomLevel,
    ReduceLevel,
    Refresh,
    ClearRange,
    EnterRange,
    AddSelcted(Selected),
    FilterSearch(Selected),
    Scroll(f32),
}

impl<'a> Program<ChartMsg> for DateChart<'a> {
    fn update(
        &mut self,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (canvas::event::Status, Option<ChartMsg>) {
        if let Some(cursor_position) = cursor.position_in(&bounds) {
            match event {
                canvas::Event::Mouse(me) => match me {
                    mouse::Event::ButtonReleased(mouse::Button::Right) => {
                        println!("🖱: {:?}", cursor_position);
                        return (canvas::event::Status::Captured, Some(ChartMsg::ClearRange));
                    }
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        self.pending.replace(cursor_position);
                    }
                    mouse::Event::ButtonPressed(mouse::Button::Middle) => {
                        if self.selected.is_some() {
                            return (canvas::event::Status::Captured, Some(ChartMsg::EnterRange));
                        }
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        let pending = self.pending.take();
                        let size = bounds.size();
                        let width = size.width;
                        if let Some(pd) = pending {
                            let mut translation = self.translation;
                            let uw = self.init(
                                size,
                                &mut translation,
                                self.last_day,
                                self.last_month,
                                &self.selected,
                            );
                            let fix = translation.x;

                            let a = (width - pd.x + fix) / uw;
                            let b = (width - cursor_position.x + fix) / uw;
                            let (start, end) = if a > b {
                                (a.ceil() as usize, b.floor() as usize)
                            } else {
                                (b.ceil() as usize, a.floor() as usize)
                            };

                            return (
                                canvas::event::Status::Captured,
                                Some(ChartMsg::FilterSearch(Selected { start, end })),
                            );
                        }
                    }
                    mouse::Event::WheelScrolled { delta } => match delta {
                        mouse::ScrollDelta::Lines { y, .. } => {
                            return (canvas::event::Status::Captured, Some(ChartMsg::Scroll(y)));
                        }
                        mouse::ScrollDelta::Pixels { y, .. } => {
                            return (canvas::event::Status::Captured, Some(ChartMsg::Scroll(y)));
                        }
                    },
                    _ => {}
                },
                canvas::Event::Keyboard(_) => {}
            };
        }
        (canvas::event::Status::Ignored, None)
    }

    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let size = bounds.size();
        let mut translation = self.translation;
        let uw = self.init(
            size,
            &mut translation,
            self.last_day,
            self.last_month,
            &self.selected,
        );
        let mut res = vec![];
        match &self.level {
            ChartLevel::Day => {
                let days = &self.days[..];
                let rects = self.cache.draw(size, |frame| {
                    frame.translate(translation);
                    Day::draw_all_day(
                        days,
                        frame,
                        self.maximal_day_count,
                        self.style.fill_color,
                        self.style.font_size,
                        self.style.big_font_size,
                        uw,
                        translation.x,
                        self.base_day,
                        false,
                    );
                });
                res.push(rects);
            }
            ChartLevel::Month => {
                let months = &self.months[..];

                let rects = self.cache.draw(size, |frame| {
                    frame.translate(translation);
                    MonthView::draw_all_month(
                        months,
                        frame,
                        self.maximal_month_count,
                        self.style.fill_color,
                        self.style.font_size,
                        self.style.big_font_size,
                        uw,
                        translation.x,
                        self.base_month,
                    );
                });

                let mut sub_frame = Frame::new(Size {
                    width: size.width / 2.0,
                    height: size.height / 2.0,
                });

                if let Some(s) = self.selected {
                    let day_range = s.months_to_days(self.base_day, self.base_month);
                    let day_uw = (size.width
                        / (day_range.start as f32 - day_range.end as f32 + 3.0) as f32)
                        .max(1.0)
                        .min(50.0);
                    let day_translation = Vector::new((day_range.end - 1) as f32 * day_uw, 0.0);
                    sub_frame.translate(day_translation);
                    let days = &self.days[..];
                    Day::draw_all_day(
                        days,
                        &mut sub_frame,
                        self.maximal_day_count,
                        self.style.fill_color,
                        self.style.font_size,
                        self.style.big_font_size,
                        day_uw,
                        day_translation.x,
                        self.base_day,
                        true,
                    );
                }
                res.push(rects);
                res.push(sub_frame.into_geometry());
            }
        };

        if let Some(selected) = self.selected.map(|selected| {
            self.selected_cache.draw(size, |frame| {
                frame.translate(translation);
                selected.draw(frame, uw)
            })
        }) {
            res.push(selected);
        }

        if let Some(pending) = self.pending.map(|pending| {
            let mut frame = Frame::new(size);
            if let Some(cursor_position) = cursor.position_in(&bounds) {
                let top_left = Point::new(
                    cursor_position.x.min(pending.x),
                    cursor_position.y.min(pending.y),
                );
                let size = Size::new(
                    (cursor_position.x - pending.x).abs(),
                    (cursor_position.y - pending.y).abs(),
                );
                frame.fill_rectangle(
                    top_left,
                    size,
                    Fill::from(Color::from_rgba(0.1, 0.5, 0.8, 0.2)),
                );
            }
            frame.into_geometry()
        }) {
            res.push(pending);
        }

        res
    }

    fn mouse_interaction(
        &self,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> iced_native::mouse::Interaction {
        if self.pending.is_some() {
            return iced_native::mouse::Interaction::Pointer;
        }
        iced_native::mouse::Interaction::default()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Selected {
    start: usize,
    end: usize,
}

impl Selected {
    pub fn get_months_range(&self, base_month: &MonthView) -> (Date, Date) {
        (
            get_offset_date(self.start, 1, base_month),
            get_offset_date(self.end, 31, base_month),
        )
    }
    pub fn get_days_range(&self, base_day: Date) -> (Date, Date) {
        (
            base_day - Duration::days(self.start as i64 + 1),
            base_day - Duration::days(self.end as i64 + 1),
        )
    }
    pub fn days_to_months(self, base_day: Date, base_month: &MonthView) -> Self {
        let (start, end) = self.get_days_range(base_day);
        let start = months_num(&MonthView::from_date(start, 0), base_month);
        let end = months_num(&MonthView::from_date(end, 0), base_month);
        Self {
            start: start.max(1) as usize - 1,
            end: end.max(1) as usize - 1,
        }
    }
    pub fn months_to_days(self, base_day: Date, base_month: &MonthView) -> Self {
        let (start, end) = self.get_months_range(base_month);
        let start = (base_day - start).whole_days() as usize;
        let end = (base_day - end).whole_days() as usize;
        Self {
            start: start.max(1) - 1,
            end: end.max(1) - 1,
        }
    }
    pub fn draw(&self, frame: &mut Frame, uw: f32) {
        let Size { width, height } = frame.size();
        if self.start == self.end {
            frame.fill_rectangle(
                Point::new(width - self.start as f32 * uw - uw, 0.0),
                Size::new(uw, height),
                Fill::from(Color::from_rgba(0.8, 0.0, 0.0, 0.3)),
            );
        } else {
            let rw = (self.start as f32 - self.end as f32) * uw;
            frame.fill_rectangle(
                Point::new(width - self.start as f32 * uw, 0.0),
                Size::new(rw, height),
                Fill::from(Color::from_rgba(0.8, 0.0, 0.0, 0.3)),
            );
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    MaxOrMin,
    FullOrAdjustable,
    DayOrMonth,
    ChartMsg(ChartMsg),
    UwChange(f32),
}
#[derive(Debug)]
pub struct DateView {
    pub days: Vec<Day>,
    pub months: Vec<MonthView>,
    pub is_show: bool,
    pub chart: Chart,
    pub full_days: Option<i64>,
    pub full_months: Option<i32>,
    pub last_day: Option<i64>,
    pub last_month: Option<i32>,
    close: button::State,
    full_or_adjustable: button::State,
    max_or_min: button::State,
    day_or_month: button::State,
    uw_input: number_input::State,
    pub is_full: bool,
}

impl Default for DateView {
    fn default() -> Self {
        Self {
            is_show: true,
            is_full: true,
            chart: Chart::new(),
            close: button::State::new(),
            full_or_adjustable: button::State::new(),
            max_or_min: button::State::new(),
            day_or_month: button::State::new(),
            uw_input: number_input::State::new(),
            days: Vec::new(),
            months: Vec::new(),
            full_days: None,
            full_months: None,
            last_day: None,
            last_month: None,
        }
    }
}

impl DateView {
    pub fn view(&mut self, theme: Theme) -> Element<Message> {
        let DateView {
            close,
            full_or_adjustable,
            max_or_min,
            day_or_month,
            chart,
            days,
            months,
            uw_input,
            ..
        } = self;
        let close_button = Button::new(close, Text::new("X")).on_press(Message::Close);
        let minimize_or_maximize_button = Button::new(
            max_or_min,
            Text::new(if self.is_show { "min" } else { "max" }),
        )
        .on_press(Message::MaxOrMin);

        let mut ctrl_row = Row::new();

        if !self.is_full {
            let uw_input = iced_aw::NumberInput::new(uw_input, chart.uw(), 30.0, Message::UwChange)
                .min(1.0)
                .step(0.1);
            ctrl_row = ctrl_row.push(uw_input);
        }
        ctrl_row = ctrl_row.push(style::horizontal_rule());
        if self.is_show {
            let full_or_adjustable_button = Button::new(
                full_or_adjustable,
                Text::new(if self.is_full { "adjustable" } else { "full" }),
            )
            .on_press(Message::FullOrAdjustable);
            let day_or_month_button =
                Button::new(day_or_month, Text::new("changing view")).on_press(Message::DayOrMonth);
            ctrl_row = ctrl_row
                .push(day_or_month_button)
                .push(full_or_adjustable_button);
        }
        ctrl_row = ctrl_row
            .push(minimize_or_maximize_button)
            .push(close_button);

        let mut content = Column::new();
        content = content.push(ctrl_row);
        if self.is_show {
            let chart = chart.chart_view(&*days, &*months).map(Message::ChartMsg);
            content = content.push(chart);
        }
        Container::new(content).into()
    }
    pub fn clear_cache_and_convert_selected_range(&mut self) {
        if let Some(s) = self.chart.selected {
            match self.chart.level {
                crate::days::ChartLevel::Day => self
                    .chart
                    .selected
                    .replace(s.months_to_days(self.chart.base_day, &self.chart.base_month)),
                crate::days::ChartLevel::Month => self
                    .chart
                    .selected
                    .replace(s.days_to_months(self.chart.base_day, &self.chart.base_month)),
            };
        }
        // self.chart.selected.take();
        self.clear_cahce();
    }
    pub fn clear_cahce(&mut self) {
        self.chart.cache.clear();
        self.chart.selected_cache.clear();
    }
    pub fn get_range(&mut self, selected: Selected) -> (Date, Date) {
        self.chart.selected.replace(selected);
        match self.chart.level {
            ChartLevel::Day => selected.get_days_range(self.chart.base_day),
            ChartLevel::Month => selected.get_months_range(&self.chart.base_month),
        }
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Close => {
                // TODO: 上层处理
            }
            Message::MaxOrMin => {
                self.is_show = !self.is_show;
            }
            Message::FullOrAdjustable => {
                self.is_full = !self.is_full;
                if self.is_full {
                    self.chart.full_days = self.full_days;
                    self.chart.full_months = self.full_months;
                } else {
                    self.chart.full_days.take();
                    self.chart.full_months.take();
                    self.align();
                }
                self.clear_cahce();
                //self.align();
            }
            Message::ChartMsg(cm) => match cm {
                ChartMsg::EnterRange => {
                    self.day_or_month();
                    self.clear_cache_and_convert_selected_range();
                }
                ChartMsg::ClearRange => {
                    self.chart.selected.take();
                    self.clear_cahce();
                }
                ChartMsg::Scroll(x) => {
                    // 向下是负数
                    if self.is_full {
                        if let Some(s) = self.chart.selected {
                            if x > 0.0 {
                                let val = s.start - s.end;
                                match self.chart.level {
                                    ChartLevel::Day => {
                                        self.chart.full_days = Some(val as i64);
                                        self.chart.last_day = Some(s.end as i64);
                                    }
                                    ChartLevel::Month => {
                                        self.chart.full_months = Some(val as i32);
                                        self.chart.last_month = Some(s.end as i32);
                                    }
                                }
                            } else {
                                match self.chart.level {
                                    ChartLevel::Day => {
                                        let fd = self.full_days.and_then(|original| {
                                            self.chart.full_days.map(|now| {
                                                let new = now - x.floor() as i64;
                                                if new > original {
                                                    original
                                                } else {
                                                    new
                                                }
                                            })
                                        });
                                        let ld = self.last_day.and_then(|original| {
                                            self.chart.last_day.map(|now| {
                                                let new = now + x.floor() as i64;
                                                if new < original {
                                                    original
                                                } else {
                                                    new
                                                }
                                            })
                                        });
                                        self.chart.full_days = fd;
                                        self.chart.last_day = ld;
                                    }
                                    ChartLevel::Month => {
                                        let fm = self.full_months.and_then(|original| {
                                            self.chart.full_months.map(|now| {
                                                let new = now - x.floor() as i32;
                                                if new > original {
                                                    original
                                                } else {
                                                    new
                                                }
                                            })
                                        });
                                        let lm = self.last_month.and_then(|original| {
                                            self.chart.last_month.map(|now| {
                                                let new = now + x.floor() as i32;
                                                if new < original {
                                                    original
                                                } else {
                                                    new
                                                }
                                            })
                                        });
                                        self.chart.full_months = fm;
                                        self.chart.last_month = lm;
                                    }
                                }
                            }
                        }
                    } else {
                        let new_x = x * self.chart.uw() + self.chart.translation.x;
                        if new_x < 0.0 {
                            self.chart.translation = Vector::new(0.0, 0.0);
                        } else {
                            self.chart.translation = Vector::new(new_x, 0.0);
                        }
                    }
                    self.chart.cache.clear();
                    self.chart.selected_cache.clear();
                }
                _ => {}
            },
            Message::DayOrMonth => {
                // 上层处理
            }
            Message::UwChange(uw) => {
                self.set_uw(uw);
                self.clear_cahce();
                self.align();
            }
        }
    }
    pub fn day_or_month(&mut self) {
        match self.chart.level {
            ChartLevel::Day => {
                self.chart.level = ChartLevel::Month;
            }
            ChartLevel::Month => {
                self.chart.level = ChartLevel::Day;
            }
        }
        self.align();
    }
    pub fn align(&mut self) {
        match self.chart.level {
            ChartLevel::Day => {
                self.chart.last_day.map(|last| {
                    self.chart.translation =
                        Vector::new((last as f32 - 1.0) * self.chart.day_uw, 0.0)
                });
            }
            ChartLevel::Month => {
                self.chart.last_month.map(|last| {
                    self.chart.translation =
                        Vector::new((last - 1) as f32 * self.chart.month_uw, 0.0)
                });
            }
        }
    }
    pub fn set_uw(&mut self, uw: f32) {
        match self.chart.level {
            ChartLevel::Day => {
                self.chart.day_uw = uw;
            }
            ChartLevel::Month => {
                self.chart.month_uw = uw;
            }
        }
    }
}

#[inline]
fn get_offset_date(mut offset: usize, mut day: u8, base_month: &MonthView) -> Date {
    let MonthView {
        mut month,
        mut year,
        ..
    } = base_month;

    year -= offset as i32 / 12;
    offset = offset % 12;

    if offset != 0 {
        let mut m_num = month_to_num(month) - offset as i32;
        if m_num <= 0 {
            m_num += 12;
            year -= 1;
        }
        month = match m_num {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => unreachable!(),
        };
    }

    let mut res = Date::from_calendar_date(year, month, day);

    while res.is_err() && day > 0 {
        day -= 1;
        res = Date::from_calendar_date(year, month, day);
    }

    res.unwrap()
}
#[derive(Debug)]
pub struct Chart {
    pub cache: Cache,
    pub selected_cache: Cache,
    pub selected: Option<Selected>,
    pub full_days: Option<i64>,
    pub full_months: Option<i32>,
    pub last_day: Option<i64>,
    pub last_month: Option<i32>,
    pub max_day_count: i64,
    pub max_month_count: i64,
    pub day_uw: f32,
    pub month_uw: f32,
    pub level: ChartLevel,
    pub translation: Vector,
    pub base_day: Date,
    pub base_month: MonthView,
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

impl Chart {
    pub fn uw(&self) -> f32 {
        match self.level {
            ChartLevel::Day => self.day_uw,
            ChartLevel::Month => self.month_uw,
        }
    }
    pub fn new() -> Self {
        Self {
            cache: Cache::new(),
            selected_cache: Cache::new(),
            selected: None,
            max_day_count: 10,
            max_month_count: 10,
            level: ChartLevel::Day,
            translation: Vector::new(0.0, 0.0),
            day_uw: 17.0,
            month_uw: 17.0,
            full_days: None,
            full_months: None,
            last_day: None,
            last_month: None,
            base_day: base_day(),
            base_month: base_month(),
        }
    }
    pub fn chart_view<'a>(
        &'a self,
        days: &'a [Day],
        months: &'a [MonthView],
    ) -> Element<'a, ChartMsg> {
        Canvas::new(DateChart {
            days,
            months,
            cache: &self.cache,
            selected_cache: &self.selected_cache,
            selected: self.selected.as_ref(),
            pending: None,
            maximal_day_count: self.max_day_count,
            maximal_month_count: self.max_month_count,
            style: Style {
                fill_color: Color::from_rgba(0.0, 0.0, 0.8, 0.5),
                font_size: self.uw() * 2.0 / 3.0,
                big_font_size: 17.0,
            },
            level: self.level,
            translation: self.translation,
            day_uw: self.day_uw,
            month_uw: self.month_uw,
            base_day: self.base_day,
            base_month: &self.base_month,
            full_days: self.full_days,
            full_months: self.full_months,
            last_day: self.last_day,
            last_month: self.last_month,
        })
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
    }
}
#[cfg(feature = "preview")]
impl iced::Sandbox for Chart {
    type Message = ChartMsg;

    fn new() -> Self {
        Self::new()
    }

    fn title(&self) -> String {
        "chart".to_owned()
    }

    fn update(&mut self, message: Self::Message) {
        // todo!()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.chart_view(&[], &[])
    }
}
