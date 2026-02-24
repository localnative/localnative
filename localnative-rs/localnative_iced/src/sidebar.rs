use iced::{
    widget::{button, column, text, vertical_space},
    Command, Element,
};

use crate::{
    config::{Config, ThemeKind},
    icons::IconItem,
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

    pub fn view(&self, theme: &ThemeKind) -> Element<Message> {
        let search_page = self.create_button(IconItem::Note, tr!("notes"), Message::TurnSearchPage);

        let settings = self.create_icon_button(IconItem::Settings, Message::TurnSettings);

        let sync_view = self.create_button(IconItem::Sync, tr!("sync"), Message::TurnSyncView);

        let theme_button = self.create_theme_button(theme);

        column![
            search_page,
            sync_view,
            vertical_space(),
            theme_button,
            settings
        ]
        .align_items(iced::Alignment::Center)
        .into()
    }

    fn create_button(
        &self,
        icon: IconItem,
        label: impl ToString,
        message: Message,
    ) -> button::Button<Message> {
        button(
            column![icon.to_text().size(Self::SIDEBAR_ICON_SIZE), text(label)]
                .align_items(iced::Alignment::Center),
        )
        .style(iced::theme::Button::Text)
        .padding(0)
        .on_press(message)
    }

    fn create_icon_button(&self, icon: IconItem, message: Message) -> button::Button<Message> {
        button(icon.to_text().size(Self::SIDEBAR_ICON_SIZE))
            .style(iced::theme::Button::Text)
            .padding(0)
            .on_press(message)
    }

    fn create_theme_button(&self, theme: &ThemeKind) -> button::Button<Message> {
        let icon = match theme {
            ThemeKind::Dark => IconItem::Dark,
            ThemeKind::Light => IconItem::Light,
        };

        button(icon.to_text().size(Self::SIDEBAR_ICON_SIZE))
            .style(iced::theme::Button::Text)
            .padding(0)
            .on_press(Message::ThemeChanged)
    }

    pub fn update(
        &mut self,
        message: Message,
        settings: &mut Settings,
        config: &mut Config,
    ) -> Command<crate::Message> {
        match message {
            Message::TurnSearchPage => {
                self.state = State::SearchPage;
            }
            Message::TurnSettings => return self.toggle_settings(settings, config),
            Message::TurnSyncView => {
                self.state = State::SyncView;
            }
            Message::ThemeChanged => {
                config.theme_kind = !config.theme_kind;
            }
        }
        Command::none()
    }

    fn toggle_settings(
        &mut self,
        settings: &mut Settings,
        config: &mut Config,
    ) -> Command<crate::Message> {
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
        Command::perform(
            crate::translate::init_bundle(config.language),
            crate::Message::ApplyLanguage,
        )
    }
}
