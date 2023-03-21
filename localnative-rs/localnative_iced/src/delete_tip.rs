use iced::{
    widget::{button, horizontal_space, row, text},
    Element,
};
use iced_aw::{Card, Modal};

use crate::tr;

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
        limit: u32,
        search_page: &'page crate::SearchPage,
    ) -> Element<'tip, Message> {
        let underlay = search_page.view(limit).map(Message::SearchPage);

        Modal::new(self.show_modal, underlay, || {
            let ok_button = button(text(tr!("ok"))).on_press(Message::Enter);
            let cancel_button = button(text(tr!("cancel"))).on_press(Message::Cancel);
            Card::new(
                // TODO: make a head
                row!(text("⚠️")),
                row!(
                    text(iced_aw::Icon::ExclamationDiamond).font(iced_aw::ICON_FONT),
                    text(tr!("delete-tip")),
                    text(tr!("delete-tip-content"))
                ),
            )
            .foot(
                row!(
                    horizontal_space(iced::Length::Fill),
                    ok_button,
                    cancel_button,
                    horizontal_space(iced::Length::Fill),
                )
                .spacing(10),
            )
            .on_close(Message::Cancel)
            .max_width(300.)
            .into()
        })
        .on_esc(Message::Cancel)
        .backdrop(Message::Cancel)
        .into()
    }
}
