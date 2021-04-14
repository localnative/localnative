use iced::{button, Color};

pub struct Symbol;
impl button::StyleSheet for Symbol {
    fn active(&self) -> button::Style {
        button::Style {
            background: None,
            border_color: Color::TRANSPARENT,
            ..Default::default()
        }
    }
}
