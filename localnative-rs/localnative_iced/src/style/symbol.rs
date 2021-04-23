use iced::{button, container, Color};

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
impl container::StyleSheet for Symbol {
    fn style(&self) -> container::Style {
        container::Style {
            background: None,
            border_color: Color::TRANSPARENT,
            ..Default::default()
        }
    }
}
