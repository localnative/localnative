use iced::{button, Color};

pub struct Url;

impl button::StyleSheet for Url {
    fn active(&self) -> button::Style {
        button::Style {
            background: None,
            border_color: Color::TRANSPARENT,
            text_color: Color::BLACK,
            ..Default::default()
        }
    }
    fn hovered(&self) -> button::Style {
        button::Style {
            text_color: Color::from_rgb8(26, 13, 171),
            ..self.active()
        }
    }
}
