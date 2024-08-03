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
        let modal_content = self.create_modal_content();

        Modal::new(underlay, modal_content)
            .on_esc(Message::Cancel)
            .backdrop(Message::Cancel)
            .into()
    }

    fn create_modal_content(&self) -> Option<Card<Message>> {
        if self.show_modal {
            Some(self.create_card())
        } else {
            None
        }
    }

    fn create_card(&self) -> Card<Message> {
        let ok_button = button(text(tr!("ok"))).on_press(Message::Enter);
        let cancel_button = button(text(tr!("cancel"))).on_press(Message::Cancel);

        Card::new(
            row!(text("⚠️")),
            row!(
                text(iced_aw::Bootstrap::ExclamationDiamond).font(iced_aw::BOOTSTRAP_FONT),
                text(tr!("delete-tip")),
                text(tr!("delete-tip-content"))
            ),
        )
        .foot(
            row!(
                horizontal_space(),
                ok_button,
                cancel_button,
                horizontal_space(),
            )
            .spacing(10),
        )
        .on_close(Message::Cancel)
        .max_width(300.)
    }
}
