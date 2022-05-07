use iced::{button, Button, Column, Command, Element, Text};

use crate::{
    config::Config,
    icons::IconItem,
    settings::Settings,
    style::{self, Theme},
    tr,
};
#[derive(Default)]
pub struct Sidebar {
    search_page: button::State,
    settings: button::State,
    sync_view: button::State,
    theme: button::State,
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

#[derive(Debug, Clone)]
pub enum Message {
    TurnSearchPage,
    TurnSettings,
    TurnSyncView,
    ThemeChanged,
}

impl Sidebar {
    pub const SIDEBAR_ICON_SIZE: u16 = 32;
    pub fn view(&mut self, theme: Theme) -> Element<Message> {
        let Self {
            search_page,
            settings,
            sync_view,
            theme: theme_state,
            ..
        } = self;
        let search_page = Button::new(
            search_page,
            Column::new()
                .push(IconItem::Note.into_text().size(Self::SIDEBAR_ICON_SIZE))
                .push(Text::new(tr!("notes")))
                .align_items(iced::Alignment::Center),
        )
        .style(style::transparent(theme))
        .padding(0)
        .on_press(Message::TurnSearchPage);
        let settings = Button::new(
            settings,
            IconItem::Settings.into_text().size(Self::SIDEBAR_ICON_SIZE),
        )
        .padding(0)
        .style(style::transparent(theme))
        .on_press(Message::TurnSettings);

        let sync_view = Button::new(
            sync_view,
            Column::new()
                .push(IconItem::Sync.into_text().size(Self::SIDEBAR_ICON_SIZE))
                .push(Text::new(tr!("sync")))
                .align_items(iced::Alignment::Center),
        )
        .padding(0)
        .style(style::transparent(theme))
        .on_press(Message::TurnSyncView);
        let theme_button = Button::new(
            theme_state,
            match theme {
                Theme::Light => IconItem::Light,
                Theme::Dark => IconItem::Dark,
            }
            .into_text()
            .size(Self::SIDEBAR_ICON_SIZE),
        )
        .padding(0)
        .style(style::transparent(theme))
        .on_press(Message::ThemeChanged);

        Column::new()
            .push(search_page)
            .push(sync_view)
            .push(style::vertical_rule())
            .push(theme_button)
            .push(settings)
            .align_items(iced::Alignment::Center)
            .into()
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
            Message::TurnSettings => {
                if self.settings_is_open {
                    self.settings_is_open = false;
                    settings.state.show(false);
                    config.limit = settings.limit_temp;
                    config.language = settings.language_temp;
                    config.disable_delete_tip = settings.disable_delete_tip_temp;
                } else {
                    self.settings_is_open = true;
                    settings.state.show(true);
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
            }
        }
        Command::none()
    }
}
