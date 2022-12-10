use iced::pure::{button, row, text, Element};
use iced_aw::pure::{Card, Modal};

use crate::{
    style::{self, Theme},
    tr,
};

pub struct DeleteTip {
    pub rowid: i64,
    pub show_modal: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Enter,
    Cancel,
    SearchPage(crate::search_page::Message),
}
impl DeleteTip {
    pub fn view<'tip, 'page: 'tip>(
        &'tip self,
        theme: Theme,
        limit: u32,
        search_page: &'page crate::SearchPage,
    ) -> Element<'tip, Message> {
        let underlay = search_page.view(theme, limit).map(Message::SearchPage);

        Modal::new(self.show_modal, underlay, || {
            let ok_button = button(text(tr!("ok"))).on_press(Message::Enter);
            let cancel_button = button(text(tr!("cancel"))).on_press(Message::Cancel);
            Card::new(
                row()
                    .push(
                        text(iced_aw::Icon::ExclamationDiamond)
                            .font(iced_aw::ICON_FONT)
                            .color(iced::Color::from_rgba(1.0, 0.0, 0.0, 0.7)),
                    )
                    .push(text(tr!("delete-tip"))),
                text(tr!("delete-tip-content")),
            )
            .foot(
                row()
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
