use std::{cell::RefCell, collections::BTreeMap, fmt::Debug, iter::once, ops::Deref};

use chrono::{Datelike, NaiveDate, Utc};
use iced::Point;
use iced::{mouse::Cursor, widget::canvas};
use plotters::{
    coord::{
        ranged1d::{ReversibleRanged, ValueFormatter},
        types::{Monthly, RangedCoordi64, Yearly},
        ReverseCoordTranslate,
    },
    prelude::{
        Cartesian2d, CoordTranslate, DiscreteRanged, IntoMonthly, IntoYearly, Ranged, RangedDate,
    },
    style::{Color, FontTransform, IntoFont, ShapeStyle, TextStyle},
};
use plotters_iced::{Chart, DrawingBackend};

use crate::{config::ThemeType, days::Message, tr};

mod constants {
    pub const DAILY: i64 = 400;
    pub const MONTHLY: i64 = 2500;
    pub const MINIMUM: i64 = 66;
    pub const MINIMUM_COUNT: i64 = 12;
}

mod utils {
    use super::*;

    pub fn fold_map<K, F>(
        days: impl IntoIterator<Item = (NaiveDate, i64)>,
        key_func: F,
    ) -> BTreeMap<K, i64>
    where
        F: Fn(NaiveDate) -> K,
        K: Ord,
    {
        days.into_iter()
            .map(|(k, v)| (key_func(k), v))
            .fold(BTreeMap::new(), |mut init, (k, v)| {
                init.entry(k).and_modify(|vv| *vv += v).or_insert(v);
                init
            })
    }

    pub fn max_count<K>(days: &BTreeMap<K, i64>) -> i64 {
        let max_count = days
            .values()
            .max()
            .filter(|s| s.cmp(&&constants::MINIMUM_COUNT).is_gt())
            .copied()
            .unwrap_or(constants::MINIMUM_COUNT);
        max_count
    }

    pub fn calculate_suitable_range(
        max_date: &mut NaiveDate,
        min_date: &mut NaiveDate,
    ) -> chrono::Duration {
        let date_diff = *max_date - *min_date;
        if date_diff.num_days() < constants::MINIMUM {
            *min_date =
                *min_date - chrono::Duration::days((constants::MINIMUM - date_diff.num_days()) / 2);
            *max_date = *max_date
                + chrono::Duration::days((constants::MINIMUM - date_diff.num_days() + 1) / 2);
        }
        *max_date - *min_date
    }
}

#[derive(Debug)]
pub struct DayChart {
    pub view: ChartView,
    pub style: ThemeType,
}

impl Deref for DayChart {
    type Target = ChartView;

    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

#[derive(Clone, Debug)]
pub struct ChartView {
    days: Vec<(NaiveDate, i64)>,
    min_date: NaiveDate,
    max_date: NaiveDate,
    max_count: i64,
    state: ChartState,
}

impl ChartView {
    fn will_draw<'a>(&'a self, state: &'a State) -> &'a Self {
        state.temporary.last().unwrap_or(self)
    }

    pub fn empty() -> Self {
        Self::new(vec![])
    }

    pub fn from_days(days: Vec<crate::days::Day>) -> Self {
        let raw = days
            .into_iter()
            .map(Into::<(NaiveDate, i64)>::into)
            .collect::<Vec<_>>();
        Self::new(raw)
    }

    pub fn new(raw: Vec<(NaiveDate, i64)>) -> Self {
        let days = utils::fold_map(raw.clone(), |k| k);

        let now = Utc::now().date_naive();
        let mut min_date = days.first_key_value().map(|(d, _)| *d).unwrap_or(now);
        let mut max_date = days
            .last_key_value()
            .map(|(d, _)| *d)
            .unwrap_or(now + chrono::Days::new(constants::MINIMUM as u64));
        let date_diff = utils::calculate_suitable_range(&mut max_date, &mut min_date);
        let (state, max_count) = if date_diff.num_days() <= constants::DAILY {
            let max_count = utils::max_count(&days);
            (
                ChartState::Daily(InnerState {
                    spec: RefCell::new(None),
                    map: days,
                }),
                max_count,
            )
        } else if date_diff.num_days() <= constants::MONTHLY {
            let map = utils::fold_map(days, |d| MonthMapKey(d.year(), d.month()));
            let max_count = utils::max_count(&map);
            (
                ChartState::Monthly(InnerState {
                    spec: RefCell::new(None),
                    map,
                }),
                max_count,
            )
        } else {
            let map = utils::fold_map(days, |d| d.year());
            let max_count = utils::max_count(&map);
            (
                ChartState::Yearly(InnerState {
                    spec: RefCell::new(None),
                    map,
                }),
                max_count,
            )
        };

        Self {
            days: raw,
            min_date,
            max_date,
            max_count,
            state,
        }
    }

    #[cfg(feature = "preview")]
    fn new_test() -> Self {
        use rand::Rng;
        use time::{Date, Duration};

        use crate::days::Day;

        fn generate_test_days(start_date: Date, num_days: usize) -> Vec<Day> {
            let mut rng = rand::thread_rng();
            let mut test_days = Vec::new();

            for i in 0..num_days {
                let date = start_date + Duration::days(i as i64);
                let count = rng.gen_range(-10..10);
                if count <= 0 {
                    continue;
                }
                test_days.push(Day { date, count });
            }

            test_days
        }

        let start_date = time::macros::date!(2022 - 01 - 01);
        let num_days = 90;
        let test_days = generate_test_days(start_date, num_days);

        Self::from_days(test_days)
    }

    fn process_chart<DB, X, Data>(
        &self,
        mut builder: plotters_iced::ChartBuilder<DB>,
        state: &State,
        data: Data,
        x_spec: X,
        style: ThemeType,
    ) -> Cartesian2d<X, RangedCoordi64>
    where
        DB: DrawingBackend,
        X: Ranged<ValueType = NaiveDate> + ValueFormatter<NaiveDate> + DiscreteRanged + Clone,
        Cartesian2d<X, RangedCoordi64>: CoordTranslate<From = (NaiveDate, i64)>,
        Data: IntoIterator<Item = (NaiveDate, i64)> + Clone,
        (NaiveDate, i64): Clone,
    {
        let mut chart = builder
            .margin(10.)
            .set_left_and_bottom_label_area_size(10.)
            .right_y_label_area_size(10.)
            .build_cartesian_2d(x_spec, 0..self.max_count)
            .expect("build chart failed");

        chart
            .configure_mesh()
            .bold_line_style(style.line_color().mix(0.1))
            .light_line_style(style.line_color().mix(0.05))
            .axis_style(ShapeStyle::from(style.line_color().mix(0.45)).stroke_width(1))
            .y_labels(10)
            .y_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&style.text_color().mix(0.65))
                    .transform(FontTransform::Rotate90),
            )
            .y_label_formatter(&|y| format!("{}", y))
            .x_label_style(
                ("sans-serif", 15)
                    .into_font()
                    .color(&style.text_color().mix(0.65))
                    .transform(FontTransform::Rotate90),
            )
            .draw()
            .expect("failed to draw chart mesh");

        chart
            .draw_series(
                plotters::series::Histogram::vertical(&chart)
                    .data(data.clone())
                    .style(style.fill_color().mix(0.95).filled())
                    .margin(0),
            )
            .expect("failed to draw chart data");
        let spec = chart.as_coord_spec().clone();

        if let Some([selected_start, selected_end]) = state.reverse_selected(&spec) {
            let selected = [(selected_start.0, self.max_count), (selected_end.0, 0)];
            chart
                .draw_series(once(plotters::prelude::Rectangle::new(
                    selected,
                    style.selection_color().mix(0.3).filled(),
                )))
                .expect("failed to draw selected rect");
        }

        if let Some(pos) = state.cursor_position {
            if let Some((xx, yy)) = spec.reverse_translate((pos.x as i32, pos.y as i32)) {
                let iter: Option<(NaiveDate, i64)> = match &self.state {
                    ChartState::Daily(InnerState { map, .. }) => map.get(&xx).map(|c| (xx, *c)),
                    ChartState::Monthly(InnerState { map, .. }) => map
                        .get(&MonthMapKey(xx.year(), xx.month()))
                        .map(|c| (xx, *c)),
                    ChartState::Yearly(InnerState { map, .. }) => {
                        map.get(&xx.year()).map(|c| (xx, *c))
                    }
                };

                chart
                    .draw_series(plotters::series::LineSeries::new(
                        [(xx, 0), (xx, self.max_count)],
                        style.line_color().mix(0.9).filled(),
                    ))
                    .expect("failed to draw x aim");
                let line_y = iter.unzip().1.unwrap_or(yy);
                chart
                    .draw_series(plotters::series::LineSeries::new(
                        [(self.min_date, line_y), (self.max_date, line_y)],
                        style.line_color().mix(0.9).filled(),
                    ))
                    .expect("failed to draw y aim");
                let text_color = style.text_color();
                chart
                    .draw_series(plotters::series::PointSeries::of_element(
                        iter,
                        5,
                        ShapeStyle::from(&style.fill_color()).filled(),
                        &|coord, _size, _style| {
                            plotters::prelude::EmptyElement::at(coord)
                                + plotters::prelude::Text::new(
                                    self.state.date_text(coord.0),
                                    (0, -30),
                                    TextStyle::from(("sans-serif", 12).into_font())
                                        .color(&text_color),
                                )
                                + plotters::prelude::Text::new(
                                    format!("{}: {:?}", tr!("count"), coord.1),
                                    (0, -15),
                                    TextStyle::from(("sans-serif", 12).into_font())
                                        .color(&text_color),
                                )
                        },
                    ))
                    .expect("failed to draw data and count text");

                if let Some(pending) = state.pending.and_then(|pending_pos| {
                    spec.reverse_translate((pending_pos.x as i32, pending_pos.y as i32))
                }) {
                    chart
                        .draw_series(once(plotters::prelude::Rectangle::new(
                            [pending, (xx, yy)],
                            style.selection_color().mix(0.5).filled(),
                        )))
                        .expect("failed to draw select rect");
                }
            }
        }
        spec
    }

    fn reverse_selected(&self, state: &State) -> Option<[(NaiveDate, i64); 2]> {
        let will_draw = self.will_draw(state);
        match &will_draw.state {
            ChartState::Daily(InnerState { spec, .. }) => spec
                .borrow()
                .as_ref()
                .and_then(|spec| state.reverse_selected(spec)),
            ChartState::Monthly(InnerState { spec, .. }) => spec
                .borrow()
                .as_ref()
                .and_then(|spec| state.reverse_selected(spec)),
            ChartState::Yearly(InnerState { spec, .. }) => spec
                .borrow()
                .as_ref()
                .and_then(|spec| state.reverse_selected(spec)),
        }
        .map(|[(a, b), (c, d)]| {
            if a > c {
                [(c, d), (a, b)]
            } else {
                [(a, b), (c, d)]
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
struct MonthMapKey(i32, u32);

impl Ord for MonthMapKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.0.cmp(&other.0) {
            std::cmp::Ordering::Equal => self.1.cmp(&other.1),
            ordering => ordering,
        }
    }
}

#[derive(Clone, Debug)]
enum ChartState {
    Daily(InnerState<RangedDate<NaiveDate>, NaiveDate>),
    Monthly(InnerState<Monthly<NaiveDate>, MonthMapKey>),
    Yearly(InnerState<Yearly<NaiveDate>, i32>),
}

impl ChartState {
    fn date_text(&self, date: NaiveDate) -> String {
        match self {
            ChartState::Daily { .. } => format!("{}: {:?}", tr!("date"), date),
            ChartState::Monthly { .. } => {
                format!("{}: {:?}-{:?}", tr!("date"), date.year(), date.month())
            }
            ChartState::Yearly { .. } => format!("{}: {:?}", tr!("date"), date.year()),
        }
    }
}

#[derive(Clone)]
struct InnerState<X, K>
where
    X: plotters::prelude::Ranged,
{
    spec: RefCell<Option<Cartesian2d<X, RangedCoordi64>>>,
    map: BTreeMap<K, i64>,
}

impl<X, K: Debug> Debug for InnerState<X, K>
where
    X: plotters::prelude::Ranged,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerState")
            .field("map", &self.map)
            .finish()
    }
}

impl<X, K> Default for InnerState<X, K>
where
    X: plotters::prelude::Ranged,
{
    fn default() -> Self {
        Self {
            spec: RefCell::new(None),
            map: BTreeMap::new(),
        }
    }
}

#[derive(Default)]
pub struct State {
    cursor_position: Option<Point>,
    pending: Option<Point>,
    selected: Option<(Point, Point)>,
    temporary: Vec<ChartView>,
}

impl State {
    fn reverse_selected<X, XT>(
        &self,
        spec: &Cartesian2d<X, RangedCoordi64>,
    ) -> Option<[(XT, i64); 2]>
    where
        X: Ranged<ValueType = XT> + ReversibleRanged,
    {
        self.selected.and_then(|(start, end)| {
            spec.reverse_translate((start.x as i32, start.y as i32))
                .zip(spec.reverse_translate((end.x as i32, end.y as i32)))
                .map(|(s, o)| [s, o])
        })
    }
}

impl Chart<Message> for DayChart {
    type State = State;

    fn build_chart<DB: plotters_iced::DrawingBackend>(
        &self,
        state: &Self::State,
        builder: plotters_iced::ChartBuilder<DB>,
    ) {
        let will_draw = self.will_draw(state);
        let range = will_draw.min_date..will_draw.max_date;
        match &will_draw.state {
            ChartState::Daily(InnerState { spec, .. }) => {
                spec.borrow_mut().replace(will_draw.process_chart(
                    builder,
                    state,
                    will_draw.days.clone(),
                    RangedDate::from(range),
                    self.style,
                ));
            }
            ChartState::Monthly(InnerState { spec, .. }) => {
                spec.borrow_mut().replace(will_draw.process_chart(
                    builder,
                    state,
                    will_draw.days.clone(),
                    range.monthly(),
                    self.style,
                ));
            }
            ChartState::Yearly(InnerState { spec, .. }) => {
                spec.borrow_mut().replace(will_draw.process_chart(
                    builder,
                    state,
                    will_draw.days.clone(),
                    range.yearly(),
                    self.style,
                ));
            }
        }
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: iced::widget::canvas::Event,
        bounds: iced::Rectangle,
        cursor: Cursor,
    ) -> (iced::event::Status, Option<Message>) {
        if let Cursor::Available(point) = cursor {
            state.cursor_position = cursor.position_in(bounds);

            match event {
                canvas::Event::Mouse(mouse) if bounds.contains(point) => match mouse {
                    iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                        state.pending = state.cursor_position;
                    }
                    iced::mouse::Event::ButtonReleased(iced::mouse::Button::Right) => {
                        state.selected.take();
                        return (iced::event::Status::Ignored, Some(Message::Clear));
                    }
                    iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                        let start = state.pending.take();
                        state.selected = state.cursor_position.zip(start);
                        return (
                            iced::event::Status::Ignored,
                            self.reverse_selected(state)
                                .and_then(|[(start, _), (end, _)]| {
                                    time::Date::from_ordinal_date(
                                        start.year(),
                                        start.ordinal() as u16,
                                    )
                                    .ok()
                                    .zip(
                                        time::Date::from_ordinal_date(
                                            end.year(),
                                            end.ordinal() as u16,
                                        )
                                        .ok(),
                                    )
                                    .map(|(start, end)| Message::Selected { start, end })
                                }),
                        );
                    }
                    iced::mouse::Event::WheelScrolled {
                        delta: iced::mouse::ScrollDelta::Lines { y, .. },
                    }
                    | iced::mouse::Event::WheelScrolled {
                        delta: iced::mouse::ScrollDelta::Pixels { y, .. },
                    } => {
                        if y.is_sign_negative() {
                            state.temporary.pop();
                        } else {
                            if let Some([(mut start, _), (mut end, _)]) =
                                self.reverse_selected(state)
                            {
                                utils::calculate_suitable_range(&mut end, &mut start);
                                let selected = start..=end;
                                let new_days = self
                                    .days
                                    .iter()
                                    .filter(|(d, _)| selected.contains(d))
                                    .cloned()
                                    .collect::<Vec<_>>();
                                let new = ChartView::new(new_days);
                                state.temporary.push(new);
                                state.selected.take();
                            }
                        }
                    }
                    _ => (),
                },

                _ => {}
            }
        }

        (iced::event::Status::Ignored, None)
    }
}

#[cfg(feature = "preview")]
pub struct NewChart {
    chart: crate::DateView,
}

#[cfg(feature = "preview")]
impl iced::Sandbox for NewChart {
    type Message = Message;

    fn new() -> Self {
        Self {
            chart: crate::DateView {
                is_show: true,
                chart: ChartView::new_test(),
            },
        }
    }

    fn title(&self) -> String {
        "test chart".into()
    }

    fn update(&mut self, message: Self::Message) {
        self.chart.update(message);
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        self.chart.view()
    }
}
