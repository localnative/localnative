use iced::widget::{button, container, horizontal_space, Column, Row};
use iced::Length;
use iced::{theme, Element};

use serde::{Deserialize, Serialize};
use time::Date;

use crate::chart::{ChartView, DayChart};
use crate::icons::IconItem;

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
pub enum Message {
    MaxOrMin,
    Selected { start: time::Date, end: time::Date },
    Clear,
}
#[derive(Debug)]
pub struct DateView {
    pub is_show: bool,
    pub chart: DayChart,
}

impl DateView {
    pub fn new(style: crate::config::ThemeType) -> Self {
        Self {
            is_show: true,
            chart: DayChart {
                view: ChartView::empty(),
                style,
            },
        }
    }
}

impl Default for DateView {
    fn default() -> Self {
        Self {
            is_show: true,
            chart: DayChart {
                view: ChartView::empty(),
                style: crate::config::ThemeType::Light,
            },
        }
    }
}

impl DateView {
    pub fn view(&self) -> Element<Message> {
        let DateView { chart, .. } = self;
        let minimize_or_maximize_button = button(
            if self.is_show {
                IconItem::FilterOff
            } else {
                IconItem::Filter
            }
            .into_text()
            .size(20),
        )
        .style(theme::Button::Text)
        .padding(0)
        .on_press(Message::MaxOrMin);

        let mut ctrl_row = Row::new();

        ctrl_row = ctrl_row.push(horizontal_space(Length::Fill));
        ctrl_row = ctrl_row.push(minimize_or_maximize_button);

        let mut content = Column::new();
        content = content.push(ctrl_row);
        if self.is_show {
            content = content.push(plotters_iced::ChartWidget::new(chart));
        }
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::MaxOrMin => {
                self.is_show = !self.is_show;
            }
            _ => {}
        }
    }
}
