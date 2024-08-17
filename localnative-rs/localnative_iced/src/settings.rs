use iced::{
    widget::{button, checkbox, column, horizontal_space, row, text, text_input, Space, Text},
    Command, Element,
    Length::{self, Shrink},
};
use iced_aw::{Card, Modal, NumberInput};

use crate::{
    config::{Config, ThemeType},
    sidebar::Sidebar,
    tr,
    translate::{self, Language},
};

pub struct Settings {
    pub language_temp: Language,
    pub disable_delete_tip_temp: bool,
    pub limit_temp: u32,
    pub show_modal: bool,
    pub allowed_origins_temp: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Save,
    Cancel,
    TryFixHost,
    DisableTip(bool),
    LanguageChanged(Language),
    LimitChanged(u32),
    LightThemeChanged(ThemeType),
    DarkThemeChanged(ThemeType),
    AllowedOriginsChanged(String),
    Other,
}

impl Settings {
    pub fn view<'settings, 'underlay: 'settings>(
        &'settings self,
        underlay: Element<'underlay, Message>,
        config: &'underlay Config,
    ) -> Element<'settings, Message> {
        Modal::new(underlay, self.create_modal_content(config))
            .on_esc(Message::Cancel)
            .into()
    }

    fn create_modal_content(&self, config: &Config) -> Option<Element<'_, Message>> {
        if !self.show_modal {
            return None;
        }

        let ok_button = button(text(tr!("ok"))).on_press(Message::Save);
        let cancel_button = button(text(tr!("cancel"))).on_press(Message::Cancel);
        let disable_delete_tip = checkbox("", config.disable_delete_tip)
            .spacing(0)
            .on_toggle(Message::DisableTip);
        let try_fix_host = button(text(tr!("try-fix-host"))).on_press(Message::TryFixHost);
        let language_selector = iced::widget::pick_list(
            &[Language::English, Language::Chinese][..],
            Some(config.language),
            Message::LanguageChanged,
        )
        .padding(3)
        .width(Shrink);
        let light_theme_selector = iced::widget::pick_list(
            &ThemeType::ALL_LIGHT[..],
            Some(config.light_theme),
            Message::LightThemeChanged,
        )
        .padding(3)
        .width(Shrink);
        let dark_theme_selector = iced::widget::pick_list(
            &ThemeType::ALL_DARK[..],
            Some(config.dark_theme),
            Message::DarkThemeChanged,
        )
        .padding(3)
        .width(Shrink);

        let limit_input = NumberInput::new(config.limit, 1000, Message::LimitChanged)
            .min(5)
            .step(1)
            .padding(0.);

        let allowed_origins_input = text_input(
            "Allowed Origin",
            self.allowed_origins_temp.as_deref().unwrap_or(""),
        )
        .on_input(Message::AllowedOriginsChanged)
        .padding(3)
        .width(Length::Fill);

        let body = column![
            row![
                text(tr!("disable-delete-tip")),
                horizontal_space(),
                disable_delete_tip
            ],
            column![
                text(tr!("lanuage")),
                Space::with_width(Shrink),
                language_selector
            ],
            column![
                text(tr!("light-theme")),
                Space::with_width(Shrink),
                light_theme_selector
            ],
            column![
                text(tr!("dark-theme")),
                Space::with_width(Shrink),
                dark_theme_selector
            ],
            row![text(tr!("limit")), horizontal_space(), limit_input],
            row![text(tr!("allowed-origins")), allowed_origins_input],
            try_fix_host
        ]
        .align_items(iced::Alignment::Center)
        .padding(0)
        .spacing(20);

        Some(Element::new(
            Card::new(Text::new(tr!("settings")), body)
                .foot(
                    row![
                        horizontal_space(),
                        ok_button,
                        cancel_button,
                        horizontal_space(),
                    ]
                    .spacing(10)
                    .align_items(iced::Alignment::Center),
                )
                .on_close(Message::Cancel)
                .max_width(400.),
        ))
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
                return Command::perform(
                    crate::init::WebKind::init_all(
                        self.allowed_origins_temp.take().map(|s| vec![s]),
                    ),
                    crate::Message::InitHost,
                )
            }
            Message::Other => {}
            Message::LightThemeChanged(t) => {
                config.light_theme = t;
            }
            Message::DarkThemeChanged(t) => {
                config.dark_theme = t;
            }
            Message::AllowedOriginsChanged(origins) => {
                self.allowed_origins_temp = Some(origins);
            }
        }
        Command::none()
    }
}
