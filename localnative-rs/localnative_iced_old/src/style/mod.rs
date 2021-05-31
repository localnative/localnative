#[allow(dead_code)]
pub mod icon;

pub mod qr_code;
pub mod symbol;
pub mod tag;
pub mod url;

use std::ops::Neg;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}
impl Neg for Theme {
    type Output = Theme;

    fn neg(self) -> Self::Output {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Dark => write!(f, "dark"),
            Theme::Light => write!(f, "light"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {}
