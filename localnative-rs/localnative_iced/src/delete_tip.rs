use iced::{button, Button, Command, Element, Row, Text};
use iced_aw::{modal, Card, Modal};

use crate::{
    icons::IconItem,
    style::{self, Theme},
    tr, Conn,
};

pub struct DeleteTip {
    pub rowid: i64,
    pub tip_state: modal::State<TipState>,
}
#[derive(Default)]
pub struct TipState {
    pub ok_button: button::State,
    pub cancel_button: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    Enter,
    Cancel,
    SearchPageMessage(crate::search_page::Message),
}
impl DeleteTip {
    pub fn view<'tip, 'page: 'tip>(
        &'tip mut self,
        theme: Theme,
        limit: u32,
        search_page: &'page mut crate::SearchPage,
    ) -> Element<'tip, Message> {
        let underlay = search_page
            .view(theme, limit)
            .map(Message::SearchPageMessage);
        let Self { tip_state, .. } = self;

        Modal::new(tip_state, underlay, |state| {
            let ok_button =
                Button::new(&mut state.ok_button, Text::new(tr!("ok"))).on_press(Message::Enter);
            let cancel_button = Button::new(&mut state.cancel_button, Text::new(tr!("cancel")))
                .on_press(Message::Cancel);
            Card::new(
                Row::new()
                    .push(
                        Text::new(iced_aw::Icon::ExclamationDiamond)
                            .font(iced_aw::ICON_FONT)
                            .color(iced::Color::from_rgba(1.0, 0.0, 0.0, 0.7)),
                    )
                    .push(Text::new(tr!("delete-tip"))),
                Text::new(tr!("delete-tip-content")),
            )
            .foot(
                Row::new()
                    .push(style::horizontal_rule())
                    .push(ok_button)
                    .push(cancel_button)
                    .push(style::horizontal_rule())
                    .spacing(10),
            )
            .on_close(Message::Cancel)
            .max_width(300)
            .into()
        })
        .style(style::transparent(theme))
        .on_esc(Message::Cancel)
        .backdrop(Message::Cancel)
        .into()
    }
}
