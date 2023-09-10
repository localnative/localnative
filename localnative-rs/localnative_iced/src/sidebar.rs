use iced::{
    widget::{button, column, vertical_space},
    Command, Element, Length,
};

use crate::{
    config::{Config, ThemeType},
    icons::{text, IconItem},
    settings::Settings,
    tr,
};
#[derive(Default)]
pub struct Sidebar {
    pub settings_is_open: bool,
    pub state: State,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
    SearchPage,
    SyncView,
}
impl Default for State {
    fn default() -> Self {
        Self::SearchPage
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    TurnSearchPage,
    TurnSettings,
    TurnSyncView,
    ThemeChanged,
}

impl Sidebar {
    pub const SIDEBAR_ICON_SIZE: u16 = 32;
    pub fn view(&self, theme: &ThemeType) -> Element<Message> {
        let search_page = button(
            column![
                IconItem::Note.into_text().size(Self::SIDEBAR_ICON_SIZE),
                text(tr!("notes"))
            ]
            .align_items(iced::Alignment::Center),
        )
        .style(iced::theme::Button::Text)
        .padding(0)
        .on_press(Message::TurnSearchPage);

        let settings = button(IconItem::Settings.into_text().size(Self::SIDEBAR_ICON_SIZE))
            .style(iced::theme::Button::Text)
            .padding(0)
            .on_press(Message::TurnSettings);

        let sync_view = button(
            column![
                IconItem::Sync.into_text().size(Self::SIDEBAR_ICON_SIZE),
                text(tr!("sync"))
            ]
            .align_items(iced::Alignment::Center),
        )
        .style(iced::theme::Button::Text)
        .padding(0)
        .on_press(Message::TurnSyncView);

        let theme_button = button(
            match theme {
                ThemeType::Light => IconItem::Light,
                ThemeType::Dark => IconItem::Dark,
            }
            .into_text()
            .size(Self::SIDEBAR_ICON_SIZE),
        )
        .style(iced::theme::Button::Text)
        .padding(0)
        .on_press(Message::ThemeChanged);

        column![
            search_page,
            sync_view,
            vertical_space(Length::Fill),
            theme_button,
            settings
        ]
        .align_items(iced::Alignment::Center)
        .into()
    }
    pub fn update(
        &mut self,
        message: Message,
        settings: &mut Settings,
        config: &mut Config,
        theme: &mut ThemeType,
    ) -> Command<crate::Message> {
        match message {
            Message::TurnSearchPage => {
                self.state = State::SearchPage;
            }
            Message::TurnSettings => {
                if self.settings_is_open {
                    self.settings_is_open = false;
                    settings.show_modal = false;

                    config.limit = settings.limit_temp;
                    config.language = settings.language_temp;
                    config.disable_delete_tip = settings.disable_delete_tip_temp;
                } else {
                    self.settings_is_open = true;
                    settings.show_modal = true;
                    settings.limit_temp = config.limit;
                    settings.language_temp = config.language;
                    settings.disable_delete_tip_temp = config.disable_delete_tip;
                }
                return Command::perform(
                    crate::translate::init_bundle(config.language),
                    crate::Message::ApplyLanguage,
                );
            }
            Message::TurnSyncView => {
                self.state = State::SyncView;
            }
            Message::ThemeChanged => {
                config.theme = !config.theme;
                *theme = config.theme;
            }
        }
        Command::none()
    }
}
