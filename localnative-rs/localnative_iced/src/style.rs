use std::ops::Not;

// ------impl note start-----
use iced::{button, qr_code, rule, text_input, Element};
use iced::{container, Background, Color};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

impl Not for Theme {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }
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
pub fn transparent(theme: Theme) -> Transparent {
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
pub fn horizontal_rule() -> iced::Rule {
    iced::Rule::horizontal(0).style(TransparentRule)
}
pub fn horizontal_rules<'a, Msg: 'a>(n: usize) -> Vec<Element<'a, Msg>> {
    let mut res = Vec::with_capacity(n);
    for _ in 0..n {
        res.push(horizontal_rule().into());
    }
    res
}
pub fn vertical_rule() -> iced::Rule {
    iced::Rule::vertical(0).style(TransparentRule)
}
// pub fn vertical_rules<'a, Msg: 'a>(n: usize) -> Vec<Element<'a, Msg>> {
//     let mut res = Vec::with_capacity(n);
//     for _ in 0..n {
//         res.push(vertical_rule().into());
//     }
//     res
// }
pub struct Note {
    theme: Theme,
}
const LIGHT_BG: Color = Color::from_rgb(0.941, 0.972, 1.0);
const DARK_BG: Color = Color::from_rgb(0.0784, 0.0863, 0.141);
impl container::StyleSheet for Note {
    fn style(&self) -> container::Style {
        let (tcolor, bg_color, bd_color) = match self.theme {
            Theme::Light => (Color::BLACK, LIGHT_BG, Color::from_rgb8(240, 255, 255)),
            Theme::Dark => (Color::WHITE, DARK_BG, Color::from_rgb8(20, 36, 36)),
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
pub fn qr_code(qr_code: qr_code::QRCode, theme: Theme) -> qr_code::QRCode {
    let (dark, light) = match theme {
        Theme::Light => (Color::BLACK, LIGHT_BG),
        Theme::Dark => (Color::WHITE, DARK_BG),
    };
    qr_code.color(dark, light)
}
// --------impl note end------
// --------impl tags start------
pub struct Count {
    theme: Theme,
}

impl button::StyleSheet for Count {
    fn active(&self) -> button::Style {
        let text_color = match self.theme {
            Theme::Light => Color::from_rgb(1.0, 0.0, 0.0),
            Theme::Dark => Color::from_rgb(0.2, 0.1, 1.0),
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
pub fn count(theme: Theme) -> Count {
    Count { theme }
}
// --------impl tags end ------
// --------impl search page start------
pub struct Normal {
    theme: Theme,
}

impl container::StyleSheet for Normal {
    fn style(&self) -> container::Style {
        let (tcolor, bg_color) = match self.theme {
            Theme::Light => (Color::BLACK, LIGHT_BG),
            Theme::Dark => (Color::WHITE, DARK_BG),
        };
        container::Style {
            text_color: Some(tcolor),
            background: Some(Background::Color(bg_color)),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

pub struct SearchInput {
    theme: Theme,
}

impl text_input::StyleSheet for SearchInput {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Background::Color(Color::TRANSPARENT),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: Color::TRANSPARENT,
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_color: Color::from_rgb(0.5, 0.5, 0.5),
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> Color {
        match self.theme {
            Theme::Light => Color::from_rgb(0.7, 0.7, 0.7),
            Theme::Dark => todo!(),
        }
    }

    fn value_color(&self) -> Color {
        match self.theme {
            Theme::Light => Color::from_rgb(0.3, 0.3, 0.3),
            Theme::Dark => todo!(),
        }
    }

    fn selection_color(&self) -> Color {
        match self.theme {
            Theme::Light => Color::from_rgb(0.8, 0.8, 1.0),
            Theme::Dark => todo!(),
        }
    }
}

// --------impl search page end------

// -- impl modal style start --

impl iced_aw::style::modal::StyleSheet for Transparent {
    fn active(&self) -> iced_aw::modal::Style {
        iced_aw::style::modal::Style {
            background: Background::Color(Color::TRANSPARENT),
        }
    }
}

// pub struct Warning {
//     theme: Theme,
// }

// pub struct Settings {
//     theme: Theme,
// }

// impl iced_aw::style::card::StyleSheet for Warning {
//     fn active(&self) -> iced_aw::card::Style {
//         let background = match self.theme {
//             Theme::Light => LIGHT_BG,
//             Theme::Dark => DARK_BG,
//         };
//         iced_aw::card::Style {
//             background,
//             border_radius: 0.5,
//             border_width: 2.5,
//             border_color: ,
//             head_background: (),
//             head_text_color: (),
//             body_background: (),
//             body_text_color: (),
//             foot_background: (),
//             foot_text_color: (),
//             close_color: (),
//         }
//     }
// }

// impl iced_aw::style::card::StyleSheet for Settings {
//     fn active(&self) -> iced_aw::card::Style {
//         iced_aw::card::Style {
//             background: (),
//             border_radius: (),
//             border_width: (),
//             border_color: (),
//             head_background: (),
//             head_text_color: (),
//             body_background: (),
//             body_text_color: (),
//             foot_background: (),
//             foot_text_color: (),
//             close_color: (),
//         }
//     }
// }
// -- impl modal style end --
