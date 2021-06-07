use iced::futures::stream::Collect;
// ------impl note start-----
use iced::{button, qr_code, rule, Element};
use iced::{container, Background, Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

pub struct Transparent {
    theme: Theme,
}
impl button::StyleSheet for Transparent {
    fn active(&self) -> button::Style {
        let text_color = match self.theme {
            Theme::Light => Color::BLACK,
            Theme::Dark => Color::WHITE,
        };
        button::Style {
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color,
            ..Default::default()
        }
    }
}
pub fn symbol(theme: Theme) -> Transparent {
    Transparent { theme }
}

pub struct Link {
    theme: Theme,
}
impl button::StyleSheet for Link {
    fn active(&self) -> button::Style {
        let text_color = match self.theme {
            Theme::Light => Color::BLACK,
            Theme::Dark => Color::WHITE,
        };
        button::Style {
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color,
            shadow_offset: Default::default(),
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            text_color: Color::from_rgb8(26, 13, 171),
            ..self.active()
        }
    }
}
pub fn link(theme: Theme) -> Link {
    Link { theme }
}

#[derive(Debug, Clone, Copy)]
pub struct TransparentRule;
impl rule::StyleSheet for TransparentRule {
    fn style(&self) -> rule::Style {
        rule::Style {
            color: Color::TRANSPARENT,
            width: 1,
            radius: 0.0,
            fill_mode: rule::FillMode::Full,
        }
    }
}
pub fn rule() -> iced::Rule {
    iced::Rule::horizontal(0).style(TransparentRule)
}
pub fn rules<'a, Msg: 'a>(n: usize) -> Vec<Element<'a, Msg>> {
    let mut res = Vec::with_capacity(n);
    for _ in 0..n {
        res.push(rule().into());
    }
    res
}
pub struct Note {
    theme: Theme,
}
const LIGHT_NOTE_BG: Color = Color::from_rgb(0.941, 0.972, 1.0);
const DARK_NOTE_BG: Color = Color::from_rgb(0.0784, 0.0863, 0.141);
impl container::StyleSheet for Note {
    fn style(&self) -> container::Style {
        let (tcolor, bg_color, bd_color) = match self.theme {
            Theme::Light => (Color::BLACK, LIGHT_NOTE_BG, Color::from_rgb8(240, 255, 255)),
            Theme::Dark => (Color::WHITE, DARK_NOTE_BG, Color::from_rgb8(20, 36, 36)),
        };
        container::Style {
            text_color: Some(tcolor),
            background: Some(Background::Color(bg_color)),
            border_radius: 30.0,
            border_width: 3.0,
            border_color: bd_color,
        }
    }
}

pub fn note(theme: Theme) -> Note {
    Note { theme }
}

pub struct Tag {
    theme: Theme,
}
impl button::StyleSheet for Tag {
    fn active(&self) -> button::Style {
        let (text_color, bg) = match self.theme {
            Theme::Light => (
                Color::BLACK,
                Some(Background::Color(Color::from_rgb8(255, 182, 193))),
            ),
            Theme::Dark => (
                Color::WHITE,
                Some(Background::Color(Color::from_rgb8(173, 216, 230))),
            ),
        };
        button::Style {
            background: bg,
            border_radius: 10.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color,
            ..Default::default()
        }
    }
}
pub fn tag(theme: Theme) -> Tag {
    Tag { theme }
}
pub fn qr_code(mut qr_code: qr_code::QRCode, theme: Theme) -> qr_code::QRCode {
    let (dark, light) = match theme {
        Theme::Light => (Color::BLACK, LIGHT_NOTE_BG),
        Theme::Dark => (Color::WHITE, DARK_NOTE_BG),
    };
    qr_code.color(dark, light)
}
// --------impl note end------
// --------impl tags start------
pub struct Count {
    theme: Theme
}

impl button::StyleSheet for Count {
    fn active(&self) -> button::Style {
        let text_color = match self.theme {
            Theme::Light => Color::from_rgb(1.0, 0.0, 0.0),
            Theme::Dark =>  Color::from_rgb(0.2, 0.1, 1.0)
        };
        button::Style {
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: text_color,
            ..Default::default()
        }
    }
}
pub fn count(theme: Theme) -> Count {
    Count {
        theme
    }
}
// --------impl tags end------
