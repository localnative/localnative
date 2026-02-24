use iced::{
    theme,
    widget::{button, container},
    Background, Border,
};

mod themes {
    use iced::Color;

    pub struct ThemeColors {
        pub active_background: Color,
        pub hovered_background: Color,
        pub text_color: Color,
        pub border_color: Color,
    }

    pub fn get_theme_colors(theme: &iced::Theme) -> ThemeColors {
        match theme {
            iced::Theme::Light => ThemeColors {
                active_background: Color::from_rgb(0.95, 0.95, 0.95),
                hovered_background: Color::from_rgb(0.9, 0.9, 0.9),
                text_color: Color::BLACK,
                border_color: Color::from_rgb(0.8, 0.8, 0.8),
            },
            iced::Theme::Dark => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            iced::Theme::Dracula => ThemeColors {
                active_background: Color::from_rgb(0.22, 0.23, 0.29),
                hovered_background: Color::from_rgb(0.2, 0.2, 0.25),
                text_color: Color::from_rgb(0.97, 0.97, 0.95),
                border_color: Color::from_rgb(0.25, 0.25, 0.3),
            },
            iced::Theme::Nord => ThemeColors {
                active_background: Color::from_rgb(0.85, 0.87, 0.9),
                hovered_background: Color::from_rgb(0.8, 0.82, 0.85),
                text_color: Color::from_rgb(0.2, 0.25, 0.3),
                border_color: Color::from_rgb(0.75, 0.77, 0.8),
            },
            iced::Theme::SolarizedLight => ThemeColors {
                active_background: Color::from_rgb(0.99, 0.96, 0.89),
                hovered_background: Color::from_rgb(0.95, 0.92, 0.85),
                text_color: Color::from_rgb(0.4, 0.48, 0.51),
                border_color: Color::from_rgb(0.9, 0.87, 0.8),
            },
            iced::Theme::SolarizedDark => ThemeColors {
                active_background: Color::from_rgb(0.0, 0.17, 0.21),
                hovered_background: Color::from_rgb(0.05, 0.22, 0.26),
                text_color: Color::from_rgb(0.58, 0.63, 0.63),
                border_color: Color::from_rgb(0.1, 0.15, 0.18),
            },
            iced::Theme::GruvboxLight => ThemeColors {
                active_background: Color::from_rgb(0.99, 0.96, 0.89),
                hovered_background: Color::from_rgb(0.95, 0.92, 0.85),
                text_color: Color::from_rgb(0.4, 0.48, 0.51),
                border_color: Color::from_rgb(0.9, 0.87, 0.8),
            },
            iced::Theme::GruvboxDark => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            iced::Theme::CatppuccinLatte => ThemeColors {
                active_background: Color::from_rgb(0.99, 0.96, 0.89),
                hovered_background: Color::from_rgb(0.95, 0.92, 0.85),
                text_color: Color::from_rgb(0.4, 0.48, 0.51),
                border_color: Color::from_rgb(0.9, 0.87, 0.8),
            },
            iced::Theme::CatppuccinFrappe => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            iced::Theme::CatppuccinMacchiato => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            iced::Theme::CatppuccinMocha => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            iced::Theme::TokyoNight => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            iced::Theme::TokyoNightStorm => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            iced::Theme::TokyoNightLight => ThemeColors {
                active_background: Color::from_rgb(0.99, 0.96, 0.89),
                hovered_background: Color::from_rgb(0.95, 0.92, 0.85),
                text_color: Color::from_rgb(0.4, 0.48, 0.51),
                border_color: Color::from_rgb(0.9, 0.87, 0.8),
            },
            iced::Theme::KanagawaWave => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            iced::Theme::KanagawaDragon => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            iced::Theme::KanagawaLotus => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            iced::Theme::Moonfly => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            iced::Theme::Nightfly => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            iced::Theme::Oxocarbon => ThemeColors {
                active_background: Color::from_rgb(0.2, 0.2, 0.2),
                hovered_background: Color::from_rgb(0.15, 0.15, 0.15),
                text_color: Color::WHITE,
                border_color: Color::from_rgb(0.3, 0.3, 0.3),
            },
            _ => ThemeColors {
                active_background: Color::from_rgb(0.9, 0.9, 0.9),
                hovered_background: Color::from_rgb(0.85, 0.85, 0.85),
                text_color: Color::BLACK,
                border_color: Color::from_rgb(0.8, 0.8, 0.8),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Url;

impl From<Url> for theme::Button {
    fn from(value: Url) -> Self {
        theme::Button::Custom(Box::new(value))
    }
}

impl button::StyleSheet for Url {
    type Style = theme::Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let colors = themes::get_theme_colors(style);
        button::Appearance {
            background: Some(Background::Color(colors.active_background)),
            text_color: colors.text_color,
            border: Border::with_radius(6.5),
            ..button::StyleSheet::active(style, &theme::Button::Text)
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let colors = themes::get_theme_colors(style);
        button::Appearance {
            background: Some(Background::Color(colors.hovered_background)),
            text_color: colors.text_color,
            border: Border {
                color: colors.border_color,
                radius: 6.5.into(),
                ..Default::default()
            },
            ..self.active(style)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tag;

impl From<Tag> for theme::Button {
    fn from(value: Tag) -> Self {
        theme::Button::Custom(Box::new(value))
    }
}

impl button::StyleSheet for Tag {
    type Style = theme::Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let colors = themes::get_theme_colors(style);
        button::Appearance {
            background: Some(Background::Color(colors.active_background)),
            text_color: colors.text_color,
            border: Border::with_radius(6.5),
            ..button::StyleSheet::active(style, &theme::Button::Primary)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TagNum;

impl From<TagNum> for theme::Button {
    fn from(value: TagNum) -> Self {
        theme::Button::Custom(Box::new(value))
    }
}

impl button::StyleSheet for TagNum {
    type Style = theme::Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let colors = themes::get_theme_colors(style);
        button::Appearance {
            background: Some(Background::Color(colors.active_background)),
            text_color: colors.text_color,
            border: Border::with_radius(4.0),
            ..button::StyleSheet::active(style, &theme::Button::Secondary)
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let colors = themes::get_theme_colors(style);
        button::Appearance {
            background: Some(Background::Color(colors.hovered_background)),
            text_color: colors.text_color,
            border: Border::with_radius(4.0),
            ..self.active(style)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SimpleBox;

impl From<SimpleBox> for theme::Container {
    fn from(value: SimpleBox) -> Self {
        theme::Container::Custom(Box::new(value))
    }
}

impl iced::widget::container::StyleSheet for SimpleBox {
    type Style = theme::Theme;

    fn appearance(&self, style: &Self::Style) -> iced::widget::container::Appearance {
        let colors = themes::get_theme_colors(style);
        container::Appearance {
            border: Border::with_radius(6.6),
            background: Some(Background::Color(colors.active_background)),
            text_color: Some(colors.text_color),
            ..style.appearance(&theme::Container::Box)
        }
    }
}
