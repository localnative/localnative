use iced::{
    widget::{button, center, container, mouse_area, opaque, row, stack, text, Space},
    Element,
};
use iced_aw::Card;

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

        if self.show_modal {
            let card = self.create_card();
            stack![
                underlay,
                opaque(
                    mouse_area(center(opaque(card)).style(|_theme| {
                        container::Style {
                            background: Some(
                                iced::Color {
                                    a: 0.8,
                                    ..iced::Color::BLACK
                                }
                                .into(),
                            ),
                            ..container::Style::default()
                        }
                    }))
                    .on_press(Message::Cancel)
                )
            ]
            .into()
        } else {
            underlay
        }
    }

    fn create_card(&self) -> Element<'_, Message> {
        let ok_button = button(text(tr!("ok"))).on_press(Message::Enter);
        let cancel_button = button(text(tr!("cancel"))).on_press(Message::Cancel);

        Card::new(
            row!(text("⚠️")),
            row!(
                text("⚠ "),
                text(tr!("delete-tip")),
                text(tr!("delete-tip-content"))
            ),
        )
        .foot(
            row!(
                Space::new().width(iced::Length::Fill),
                ok_button,
                cancel_button,
                Space::new().width(iced::Length::Fill),
            )
            .spacing(10),
        )
        .on_close(Message::Cancel)
        .max_width(300.)
        .into()
    }
}
