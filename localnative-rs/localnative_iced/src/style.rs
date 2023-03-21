use iced::{theme, Background, Color};

#[derive(Debug, Clone, Copy)]
pub struct Url;

impl From<Url> for theme::Button {
    fn from(value: Url) -> Self {
        theme::Button::Custom(Box::new(value))
    }
}

impl iced::widget::button::StyleSheet for Url {
    type Style = theme::Theme;

    fn active(&self, style: &Self::Style) -> iced_style::button::Appearance {
        iced_style::button::StyleSheet::active(style, &theme::Button::Text)
    }

    fn hovered(&self, style: &Self::Style) -> iced_style::button::Appearance {
        iced_style::button::Appearance {
            text_color: match style {
                iced::Theme::Light => Color::from_rgb8(29, 28, 229),
                iced::Theme::Dark => Color::from_rgb8(222, 186, 206),
                iced::Theme::Custom(_) => unreachable!(),
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

fn light_tag() -> Color {
    Color::from_rgb8(246, 90, 131)
}

fn dark_tag() -> Color {
    Color::from_rgb8(98, 79, 130)
}

impl iced::widget::button::StyleSheet for Tag {
    type Style = theme::Theme;

    fn active(&self, style: &Self::Style) -> iced_style::button::Appearance {
        iced_style::button::Appearance {
            border_radius: 6.5,
            background: Some(Background::Color(match style {
                iced::Theme::Light => light_tag(),
                iced::Theme::Dark => dark_tag(),
                iced::Theme::Custom(_) => unreachable!(),
            })),
            ..iced_style::button::StyleSheet::active(style, &theme::Button::Primary)
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

impl iced::widget::button::StyleSheet for TagNum {
    type Style = theme::Theme;

    fn active(&self, style: &Self::Style) -> iced_style::button::Appearance {
        iced_style::button::Appearance {
            text_color: match style {
                iced::Theme::Light => light_tag(),
                iced::Theme::Dark => dark_tag(),
                iced::Theme::Custom(_) => unreachable!(),
            },
            ..iced_style::button::StyleSheet::active(style, &theme::Button::Text)
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced_style::button::Appearance {
        iced_style::button::Appearance {
            text_color: match style {
                iced::Theme::Light => Color::from_rgb8(254, 208, 73),
                iced::Theme::Dark => Color::from_rgb8(49, 198, 212),
                iced::Theme::Custom(_) => unreachable!(),
            },
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

    fn appearance(&self, style: &Self::Style) -> iced_style::container::Appearance {
        iced_style::container::Appearance {
            border_radius: 6.6,
            background: Some(Background::Color(match style {
                iced::Theme::Light => Color::from_rgba8(144, 140, 170, 0.15),
                iced::Theme::Dark => Color::from_rgba8(64, 61, 82, 0.15),
                iced::Theme::Custom(_) => unreachable!(),
            })),
            ..style.appearance(&theme::Container::Box)
        }
    }
}
