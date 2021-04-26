use iced::{button, container, rule, Color};

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
impl rule::StyleSheet for Symbol {
    fn style(&self) -> rule::Style {
        rule::Style {
            color: Color::TRANSPARENT,
            width: 0,
            radius: 0.0,
            fill_mode: rule::FillMode::Percent(90.0),
        }
    }
}
