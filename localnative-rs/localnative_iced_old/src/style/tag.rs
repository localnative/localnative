use iced::{button, Color};

pub struct Tag;
impl button::StyleSheet for Tag {
    fn active(&self) -> button::Style {
        button::Style {
            border_radius: 10.0,
            background: Color::from_rgb8(255, 235, 205).into(),
            ..Default::default()
        }
    }
}
