use iced::{
    button,
    canvas::{self, Cache, Cursor, Fill, Frame, Geometry, Program},
    mouse, Button, Canvas, Color, Column, Container, Element, Point, Rectangle, Row, Size, Text,
    Vector,
};

use iced_aw::{date_picker, number_input, DatePicker};
use serde::{Deserialize, Serialize};
use time::{Date, Duration, Month};

use crate::{
    config::Config,
    icons::IconItem,
    style::{self, Theme},
    tr,
};

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
                        *translation =
                            Vector::new(((s.end as i64 - 1) * uw).max(0).min(10000) as f32, 0.0)
                    } else if let Some(last) = last_day {
                        *translation =
                            Vector::new(((last - 1).max(0) * uw).max(0).min(10000) as f32, 0.0);
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
                    } else if let Some(last) = last_month {
                        *translation = Vector::new(((last - 1) * uw).max(0).min(10000) as f32, 0.0);
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
pub fn num_to_month(num: i32) -> Month {
    match num {
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
    #[allow(clippy::too_many_arguments)]
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

            if idx == num || (date_pointer.month() == Month::January && date_pointer.day() == 1) {
                let year = canvas::Text {
                    content: date_pointer.year().to_string(),
                    position: Point::new(x, 0.0),
                    size: big_font_size,
                    ..Default::default()
                };
                frame.fill_text(year);
            }
            if day == 1 && uw > 1.0 {
                let month = canvas::Text {
                    content: date_pointer.month().to_string(),
                    position: Point::new(x, big_font_size),
                    size: big_font_size,
                    ..Default::default()
                };
                frame.fill_text(month);
            }
            if uw >= font_size * 1.5 {
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
            if uw >= font_size * 1.5 && (height - dh < day_text_y - font_size) {
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
    #[allow(clippy::too_many_arguments)]
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
        offset %= 12;

        if offset != 0 {
            let mut m_num = month_to_num(month) - offset as i32;
            if m_num <= 0 {
                m_num += 12;
                year -= 1;
            }
            month = num_to_month(m_num);
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
            if uw >= font_size * 1.5 {
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
            if font_size * 1.5 <= uw && height - dh < month_text_y - font_size {
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
    ClearRange,
    EnterRange,
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
                let days = self.days;
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
                    );
                });
                res.push(rects);
            }
            ChartLevel::Month => {
                let months = self.months;
                let rects = self
                    .cache
                    .draw(Size::new(size.width, size.height), |frame| {
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

                res.push(rects);
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
            get_offset_date(self.start, 31, base_month),
            get_offset_date(self.end, 31, base_month),
        )
    }
    pub fn get_days_range(&self, base_day: Date) -> (Date, Date) {
        (
            base_day - Duration::days(self.start as i64 + 1),
            base_day - Duration::days(self.end as i64 + 1),
        )
    }
    pub fn date_into_days_selected(&mut self, start: Date, end: Date, base_day: Date) {
        self.start = (base_day - start).whole_days().max(1) as usize - 1;
        self.end = (base_day - end).whole_days().max(1) as usize - 1;
    }
    pub fn date_into_months_selected(
        &mut self,
        start: Date,
        end: Date,
        base_month: &MonthView,
        base_day: Date,
    ) {
        let ds = Selected {
            start: (base_day - start).whole_days().max(1) as usize - 1,
            end: (base_day - end).whole_days().max(1) as usize - 1,
        };
        *self = ds.days_to_months(base_day, base_month);
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
        let start = (base_day - start).whole_days();
        let end = (base_day - end).whole_days();
        Self {
            start: start.max(1) as usize - 1,
            end: end.max(1) as usize - 1,
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
    MaxOrMin,
    FullOrAdjustable,
    DayOrMonth,
    ChartMsg(ChartMsg),
    PreviewChartMsg(ChartMsg),
    UwChange(f32),
    CancelStartPick,
    CancelEndPick,
    PickStart,
    PickEnd,
    PickStartRes(date_picker::Date),
    PickEndRes(date_picker::Date),
}
#[derive(Debug)]
pub struct DateView {
    pub days: Vec<Day>,
    pub months: Vec<MonthView>,
    pub is_show: bool,
    pub chart: Chart,
    pub preview_chart: Chart,
    pub full_days: Option<i64>,
    pub full_months: Option<i32>,
    pub last_day: Option<i64>,
    pub last_month: Option<i32>,
    full_or_adjustable: button::State,
    max_or_min: button::State,
    day_or_month: button::State,
    clear_button: button::State,
    uw_input: number_input::State,
    start_date_picker: date_picker::State,
    end_date_picker: date_picker::State,
    pick_start: button::State,
    pick_end: button::State,
    pub is_full: bool,
}

impl Default for DateView {
    fn default() -> Self {
        Self {
            is_show: true,
            is_full: true,
            chart: Chart::new(),
            preview_chart: Chart {
                fill_color: Color::from_rgba(0.8, 0.2, 0.3, 0.6),
                ..Chart::new()
            },
            full_or_adjustable: button::State::new(),
            max_or_min: button::State::new(),
            day_or_month: button::State::new(),
            clear_button: button::State::new(),
            uw_input: number_input::State::new(),
            start_date_picker: date_picker::State::now(),
            end_date_picker: date_picker::State::now(),
            pick_start: button::State::new(),
            pick_end: button::State::new(),
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
    pub fn new(config: &Config) -> Self {
        Self {
            is_show: config.date_filter_is_show,
            is_full: config.date_mode_is_full,
            chart: Chart {
                day_uw: config.day_uw,
                month_uw: config.month_uw,
                ..Chart::new()
            },
            preview_chart: Chart {
                fill_color: Color::from_rgba(0.8, 0.2, 0.3, 0.6),
                ..Chart::new()
            },
            full_or_adjustable: button::State::new(),
            max_or_min: button::State::new(),
            day_or_month: button::State::new(),
            clear_button: button::State::new(),
            uw_input: number_input::State::new(),
            start_date_picker: date_picker::State::now(),
            end_date_picker: date_picker::State::now(),
            pick_start: button::State::new(),
            pick_end: button::State::new(),
            days: Vec::new(),
            months: Vec::new(),
            full_days: None,
            full_months: None,
            last_day: None,
            last_month: None,
        }
    }
    pub fn update_from_handle_days(&mut self, handle_days: HandleDays) {
        let HandleDays {
            days,
            months,
            max_day_count,
            max_month_count,
            full_days,
            full_months,
            last_day,
            last_month,
        } = handle_days;
        self.days = days;
        self.months = months;
        self.full_days = full_days;
        self.full_months = full_months;
        self.last_day = last_day;
        self.last_month = last_month;
        self.chart.last_day = last_day;
        self.chart.last_month = last_month;
        self.chart.max_day_count = max_day_count;
        self.chart.max_month_count = max_month_count;
    }
    pub fn view(&mut self, theme: Theme) -> Element<Message> {
        let range = self.get_current_range();

        let DateView {
            full_or_adjustable,
            max_or_min,
            day_or_month,
            chart,
            preview_chart,
            days,
            months,
            uw_input,
            clear_button,
            start_date_picker,
            end_date_picker,
            pick_start,
            pick_end,
            ..
        } = self;

        let minimize_or_maximize_button = Button::new(
            max_or_min,
            if self.is_show {
                IconItem::FilterOff
            } else {
                IconItem::Filter
            },
        )
        .padding(0)
        .style(style::transparent(theme))
        .on_press(Message::MaxOrMin);

        let mut ctrl_row = Row::new();

        if let Some(range_info) = range.map(move |(start_date, end_date)| {
            let (start_year, start_month, start_day) = start_date.to_calendar_date();
            start_date_picker.set_date(
                start_year,
                month_to_num(start_month) as u32,
                start_day as u32,
            );
            let start_picker = DatePicker::new(
                start_date_picker,
                Button::new(
                    pick_start,
                    Row::new()
                        .push(IconItem::Date)
                        .push(Text::new(start_date.to_string())),
                )
                .style(style::transparent(theme))
                .padding(0)
                .on_press(Message::PickStart),
                Message::CancelStartPick,
                Message::PickStartRes,
            );
            let (end_year, end_month, end_day) = end_date.to_calendar_date();
            end_date_picker.set_date(end_year, month_to_num(end_month) as u32, end_day as u32);
            let end_picker = DatePicker::new(
                end_date_picker,
                Button::new(
                    pick_end,
                    Row::new()
                        .push(IconItem::Date)
                        .push(Text::new(end_date.to_string())),
                )
                .style(style::transparent(theme))
                .padding(0)
                .on_press(Message::PickEnd),
                Message::CancelEndPick,
                Message::PickEndRes,
            );
            let clear_button = Button::new(clear_button, IconItem::Clear)
                .style(style::transparent(theme))
                .padding(0)
                .on_press(Message::ChartMsg(ChartMsg::ClearRange));
            Row::new()
                .push(Text::new(tr!("range")))
                .push(start_picker)
                .push(Text::new(tr!("to")))
                .push(end_picker)
                .push(clear_button)
        }) {
            ctrl_row = ctrl_row.push(range_info);
        };
        if !self.is_full {
            let uw_input = iced_aw::NumberInput::new(uw_input, chart.uw(), 30.0, Message::UwChange)
                .min(1.0)
                .padding(0)
                .step(0.1);
            ctrl_row = ctrl_row.push(Text::new(tr!("uw"))).push(uw_input);
        }
        ctrl_row = ctrl_row.push(style::horizontal_rule());
        if self.is_show {
            let full_or_adjustable_button = Button::new(
                full_or_adjustable,
                if self.is_full {
                    IconItem::FullExit
                } else {
                    IconItem::Full
                },
            )
            .padding(0)
            .style(style::transparent(theme))
            .on_press(Message::FullOrAdjustable);

            let day_or_month_button = Button::new(day_or_month, {
                match chart.level {
                    ChartLevel::Day => IconItem::DayTime,
                    ChartLevel::Month => IconItem::MonthTime,
                }
            })
            .padding(0)
            .style(style::transparent(theme))
            .on_press(Message::DayOrMonth);
            ctrl_row = ctrl_row
                .push(day_or_month_button)
                .push(full_or_adjustable_button);
        }
        ctrl_row = ctrl_row.push(minimize_or_maximize_button);

        let mut content = Column::new();
        content = content.push(ctrl_row);
        if self.is_show {
            if chart.selected.is_some() && matches!(chart.level, ChartLevel::Month) {
                let preview_chart_view = preview_chart
                    .chart_view(&*days, &*months)
                    .map(Message::PreviewChartMsg);
                content = content.push(preview_chart_view);
            }
            let chart_view = chart.chart_view(&*days, &*months).map(Message::ChartMsg);
            content = content.push(chart_view);
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
    pub fn preview_chart(&mut self, selected: Selected) {
        self.preview_chart.selected.replace(selected);
        self.preview_chart.selected_cache.clear();
        self.preview_chart.cache.clear();
    }
    pub fn preview_chart_update(&mut self, selected: Selected) {
        self.preview_chart
            .full_days
            .replace(selected.start as i64 - selected.end as i64);
        self.preview_chart.last_day.replace(selected.end as i64);
    }
    pub fn clear_preview_chart(&mut self) {
        self.preview_chart.selected.take();
        self.preview_chart.selected_cache.clear();
        self.preview_chart.cache.clear();
    }
    pub fn clear_chart(&mut self) {
        self.chart.selected.take();
        self.chart.selected_cache.clear();
        self.chart.cache.clear();
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
    pub fn get_current_range(&self) -> Option<(Date, Date)> {
        self.chart.selected.map(|s| match self.chart.level {
            ChartLevel::Day => s.get_days_range(self.chart.base_day),
            ChartLevel::Month => {
                if let Some(ds) = self.preview_chart.selected {
                    ds.get_days_range(self.preview_chart.base_day)
                } else {
                    s.get_months_range(&self.chart.base_month)
                }
            }
        })
    }
    pub fn update(&mut self, message: Message) {
        match message {
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
                    self.clear_preview_chart();
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
            Message::PreviewChartMsg(cm) => match cm {
                ChartMsg::ClearRange => {
                    self.preview_chart.selected.take();
                    self.preview_chart.selected_cache.clear();
                    self.preview_chart.cache.clear();
                }
                ChartMsg::EnterRange => {
                    // 不用处理
                }
                ChartMsg::FilterSearch(_) => {
                    // 上层处理
                }
                ChartMsg::Scroll(_) => {
                    // 不用处理
                }
            },
            Message::DayOrMonth => {
                // 上层处理
            }
            Message::UwChange(uw) => {
                self.set_uw(uw);
                self.clear_cahce();
                self.align();
            }
            Message::CancelStartPick => {
                self.start_date_picker.show(false);
            }
            Message::CancelEndPick => {
                self.end_date_picker.show(false);
            }
            Message::PickStart => {
                self.start_date_picker.show(true);
            }
            Message::PickEnd => {
                self.end_date_picker.show(true);
            }
            Message::PickStartRes(date) => {
                self.start_date_picker.show(false);
                let start_date = Date::from_calendar_date(
                    date.year,
                    num_to_month(date.month as i32),
                    date.day as u8,
                )
                .unwrap();
                let Self {
                    chart,
                    preview_chart,
                    ..
                } = self;
                chart.set_range_from_start(preview_chart, start_date);
                self.clear_cahce();
                self.preview_chart.cache.clear();
                self.preview_chart.selected_cache.clear();
            }
            Message::PickEndRes(date) => {
                self.end_date_picker.show(false);
                let end_date = Date::from_calendar_date(
                    date.year,
                    num_to_month(date.month as i32),
                    date.day as u8,
                )
                .unwrap();
                let Self {
                    chart,
                    preview_chart,
                    ..
                } = self;
                chart.set_range_from_end(preview_chart, end_date);
                self.clear_cahce();
                self.preview_chart.cache.clear();
                self.preview_chart.selected_cache.clear();
            }
        }
    }
    pub fn day_or_month(&mut self) {
        match self.chart.level {
            ChartLevel::Day => {
                self.chart.level = ChartLevel::Month;
                self.clear_preview_chart();
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
                self.chart.day_align();
            }
            ChartLevel::Month => {
                self.chart.month_align();
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
    offset %= 12;

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

#[inline]
pub fn date_into_usize(date: Date, base_day: Date) -> usize {
    if date > base_day {
        return 0;
    }
    (base_day - date).whole_days() as usize - 1
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
    pub fill_color: Color,
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

impl Chart {
    pub fn set_range_from_start(&mut self, preview_chart: &mut Chart, start: Date) {
        match self.level {
            ChartLevel::Day => {
                let mut start = date_into_usize(start, self.base_day);
                let ds = if let Some(ds) = self.selected {
                    if start < ds.end {
                        start = ds.end + 1;
                    }
                    Selected { start, end: ds.end }
                } else {
                    Selected { start, end: 0 }
                };
                self.full_days.replace(ds.start as i64 - ds.end as i64);
                self.last_day.replace(ds.end as i64);
                self.selected.replace(ds);
            }
            ChartLevel::Month => {
                let ds = if let Some(ds) = self.selected {
                    let mut ds = ds.months_to_days(self.base_day, &self.base_month);
                    ds.start = date_into_usize(start, self.base_day).max(ds.end + 1);
                    ds
                } else {
                    Selected {
                        start: date_into_usize(start, self.base_day).max(1),
                        end: 0,
                    }
                };
                preview_chart
                    .full_days
                    .replace(ds.start as i64 - ds.end as i64);
                preview_chart.last_day.replace(ds.end as i64);
                preview_chart.selected.replace(ds);
                let ms = ds.days_to_months(self.base_day, &self.base_month);
                self.selected.replace(ms);
                self.full_months.replace(ms.start as i32 - ms.end as i32);
                self.last_month.replace(ms.end as i32);
            }
        }
    }
    pub fn set_range_from_end(&mut self, preview_chart: &mut Chart, end: Date) {
        match self.level {
            ChartLevel::Day => {
                let mut end = date_into_usize(end, self.base_day);
                let ds = if let Some(ds) = self.selected {
                    if end > ds.start {
                        end = ds.start - 1;
                    }
                    Selected {
                        start: ds.start,
                        end,
                    }
                } else {
                    Selected {
                        start: end + 5,
                        end,
                    }
                };
                self.full_days.replace((ds.start - ds.end) as i64);
                self.last_day.replace(ds.end as i64);
                self.selected.replace(ds);
            }
            ChartLevel::Month => {
                let ds = if let Some(ds) = self.selected {
                    let mut ds = ds.months_to_days(self.base_day, &self.base_month);
                    ds.end = date_into_usize(end, self.base_day).min(ds.start.max(1) - 1);
                    ds
                } else {
                    let end = date_into_usize(end, self.base_day);
                    Selected {
                        start: end + 5,
                        end,
                    }
                };
                preview_chart.full_days.replace((ds.start - ds.end) as i64);
                preview_chart.last_day.replace(ds.end as i64);
                preview_chart.selected.replace(ds);
                let ms = ds.days_to_months(self.base_day, &self.base_month);
                self.selected.replace(ms);
                self.full_months.replace((ms.start - ms.end) as i32);
                self.last_month.replace(ms.end as i32);
            }
        }
    }
    pub fn uw(&self) -> f32 {
        match self.level {
            ChartLevel::Day => self.day_uw,
            ChartLevel::Month => self.month_uw,
        }
    }
    pub fn day_align(&mut self) {
        if let Some(last) = self.last_day {
            self.translation = Vector::new((last as f32 - 1.0) * self.day_uw, 0.0);
        }
    }
    pub fn month_align(&mut self) {
        if let Some(last) = self.last_month {
            self.translation = Vector::new((last - 1) as f32 * self.month_uw, 0.0);
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
            fill_color: Color::from_rgba(0.0, 0.0, 0.8, 0.5),
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
                fill_color: self.fill_color,
                font_size: (self.uw() * 2.0 / 3.0).max(10.0),
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

    fn update(&mut self, _message: Self::Message) {
        // todo!()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.chart_view(&[], &[])
    }
}
