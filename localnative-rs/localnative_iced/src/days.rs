use iced::widget::{button, container, Column, Row, Space};
use iced::Element;
use iced::Length;

use crate::chart::{ChartView, DayChart};
use crate::icons::IconItem;

#[derive(Debug, Clone)]
pub enum Message {
    MaxOrMin,
    Selected {
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    },
    Clear,
}

#[derive(Debug)]
pub struct DateView {
    pub is_show: bool,
    pub chart: DayChart,
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
    pub fn new(style: crate::config::ThemeType) -> Self {
        Self {
            is_show: true,
            chart: DayChart {
                view: ChartView::empty(),
                style,
            },
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let minimize_or_maximize_button = self.create_minimize_or_maximize_button();
        let ctrl_row = self.create_control_row(minimize_or_maximize_button);
        let content = self.create_content(ctrl_row);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn create_minimize_or_maximize_button(&self) -> button::Button<'_, Message> {
        button(if self.is_show {
            IconItem::FilterOff
        } else {
            IconItem::Filter
        })
        .style(button::text)
        .padding(0)
        .on_press(Message::MaxOrMin)
    }

    fn create_control_row<'self_lifetime, 'button_lifetime: 'self_lifetime>(
        &'self_lifetime self,
        button: button::Button<'button_lifetime, Message>,
    ) -> Row<'self_lifetime, Message> {
        let mut ctrl_row = Row::new();
        ctrl_row = ctrl_row.push(Space::new().width(iced::Length::Fill));
        ctrl_row = ctrl_row.push(button);
        ctrl_row
    }

    fn create_content<'self_lifetime, 'row_lifetime: 'self_lifetime>(
        &'self_lifetime self,
        ctrl_row: Row<'row_lifetime, Message>,
    ) -> Column<'self_lifetime, Message> {
        let mut content = Column::new();
        content = content.push(ctrl_row);
        if self.is_show {
            content = content.push(iced::Element::from(crate::plotters_bridge::ChartWidget::new(&self.chart)));
        }
        content
    }

    pub fn update(&mut self, message: Message) {
        if let Message::MaxOrMin = message {
            self.is_show = !self.is_show;
        }
    }
}
