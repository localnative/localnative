use iced::{
    widget::{button, checkbox, column, horizontal_space, radio, row},
    Command, Element,
    Length::Fill,
    Length::Shrink,
};
use iced_aw::{card, modal, number_input};

use crate::{
    config::Config,
    icons::text,
    sidebar::Sidebar,
    tr,
    translate::{self, Language},
};
pub struct Settings {
    pub language_temp: Language,
    pub disable_delete_tip_temp: bool,
    pub limit_temp: u32,
    pub show_modal: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Save,
    Cancel,
    TryFixHost,
    DisableTip(bool),
    LanguageChanged(Language),
    LimitChanged(u32),
    Other,
}
impl Settings {
    pub fn view<'settings, 'underlay: 'settings>(
        &'settings self,
        underlay: Element<'underlay, Message>,
        config: &'underlay Config,
    ) -> Element<'settings, Message> {
        let disable_delete_tip = config.disable_delete_tip;
        let language = config.language;
        let limit = config.limit;
        modal(self.show_modal, underlay, {
            let ok_button = button(text(tr!("ok"))).on_press(Message::Save);
            let cancel_button = button(text(tr!("cancel"))).on_press(Message::Cancel);
            let disable_delete_tip =
                checkbox("", disable_delete_tip, Message::DisableTip).spacing(0);
            let try_fix_host = button(text(tr!("try-fix-host"))).on_press(Message::TryFixHost);
            // TODO: picklist not normaly work with modal
            // let language_selector = pick_list(
            //     &[Language::English, Language::Chinese][..],
            //     Some(language),
            //     Message::LanguageChanged,
            // )
            // .padding(3)
            // .width(Shrink);
            let language_selector = row![
                horizontal_space(Fill),
                radio(
                    tr!("english"),
                    Language::English,
                    Some(language),
                    Message::LanguageChanged,
                )
                .text_shaping(iced_graphics::core::text::Shaping::Advanced),
                radio(
                    tr!("chinese"),
                    Language::Chinese,
                    Some(language),
                    Message::LanguageChanged,
                )
                .text_shaping(iced_graphics::core::text::Shaping::Advanced),
                horizontal_space(Fill)
            ]
            .spacing(30);

            let limit_input = number_input(limit, 1000, Message::LimitChanged)
                .min(5)
                .step(1)
                .padding(0.);

            let body = column![
                row![
                    text(tr!("disable-delete-tip")),
                    horizontal_space(Fill),
                    disable_delete_tip
                ],
                column![
                    text(tr!("lanuage")),
                    horizontal_space(Shrink),
                    language_selector
                ],
                row![text(tr!("limit")), horizontal_space(Fill), limit_input],
                try_fix_host
            ]
            .align_items(iced::Alignment::Center)
            .padding(0)
            .spacing(10);

            card(text(tr!("settings")), body)
                .foot(
                    row![
                        horizontal_space(Fill),
                        ok_button,
                        cancel_button,
                        horizontal_space(Fill),
                    ]
                    .spacing(10)
                    .align_items(iced::Alignment::Center),
                )
                .on_close(Message::Cancel)
                .max_width(400.)
        })
        .on_esc(Message::Cancel)
        .into()
    }
    pub fn update(
        &mut self,
        message: Message,
        config: &mut Config,
        sidebar: &mut Sidebar,
    ) -> Command<crate::Message> {
        match message {
            Message::Save => {
                self.show_modal = false;
                sidebar.settings_is_open = false;
            }
            Message::Cancel => {
                self.show_modal = false;
                sidebar.settings_is_open = false;
                config.disable_delete_tip = self.disable_delete_tip_temp;
                config.language = self.language_temp;
                config.limit = self.limit_temp;
                return Command::perform(
                    translate::init_bundle(config.language),
                    crate::Message::ApplyLanguage,
                );
            }
            Message::DisableTip(flag) => {
                config.disable_delete_tip = flag;
            }
            Message::LanguageChanged(language) => {
                config.language = language;
                return Command::perform(
                    translate::init_bundle(language),
                    crate::Message::ApplyLanguage,
                );
            }
            Message::LimitChanged(limit) => {
                config.limit = limit;
            }
            Message::TryFixHost => {
                return Command::perform(crate::init::WebKind::init_all(), crate::Message::InitHost)
            }
            Message::Other => {}
        }
        Command::none()
    }
}
