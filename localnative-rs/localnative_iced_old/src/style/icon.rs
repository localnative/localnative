use iced::{HorizontalAlignment, Text, VerticalAlignment};

use crate::ICONS;

pub struct Icon;

const ICON_SIZE: u16 = 25;
fn icon(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .font(ICONS)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Top)
        .size(ICON_SIZE)
}
impl Icon {
    pub fn dark() -> Text {
        icon('\u{E90E}')
    }
    pub fn light() -> Text {
        icon('\u{E909}')
    }
    pub fn logo() -> anyhow::Result<Vec<u8>> {
        let ico_buffer = include_bytes!("../../../icons/icon.ico");
        Ok(image::load_from_memory(ico_buffer)?
            .into_rgba8()
            .as_raw()
            .clone())
    }
    pub fn qr_code() -> Text {
        icon('\u{E905}')
    }
    pub fn delete_bin() -> Text {
        icon('\u{E90C}')
    }
    pub fn delete_back() -> Text {
        icon('\u{E90D}')
    }
    pub fn close(size: u16) -> Text {
        icon('\u{E90F}').size(size)
    }
    pub fn list_settings() -> Text {
        icon('\u{E908}')
    }
    pub fn settings() -> Text {
        icon('\u{E901}')
    }
    pub fn maxmize() -> Text {
        icon('\u{E907}')
    }
    pub fn minmize() -> Text {
        icon('\u{E906}')
    }
    pub fn reset() -> Text {
        icon('\u{E903}')
    }
    pub fn refresh() -> Text {
        icon('\u{E904}')
    }
    pub fn search() -> Text {
        icon('\u{E902}')
    }
    pub fn enter() -> Text {
        icon('\u{E90A}')
    }
    pub fn cancel() -> Text {
        icon('\u{E910}')
    }
    pub fn edit() -> Text {
        icon('\u{E90B}')
    }
}
