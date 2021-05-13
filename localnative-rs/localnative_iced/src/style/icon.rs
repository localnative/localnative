use iced::Svg;
pub struct Icon;

const ICON_SIZE: u16 = 25;
impl Icon {
    pub fn dark() -> Svg {
        Svg::from_path("./icons/dark.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn light() -> Svg {
        Svg::from_path("./icons/light.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn logo() -> anyhow::Result<Vec<u8>> {
        let ico_buffer = include_bytes!("../../../icons/icon.ico");
        Ok(image::load_from_memory(ico_buffer)?
            .into_rgba8()
            .as_raw()
            .clone())
    }
    pub fn qr_code() -> Svg {
        Svg::from_path("./icons/qr-code.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn delete_bin() -> Svg {
        Svg::from_path("./icons/delete-bin.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn delete_back() -> Svg {
        Svg::from_path("./icons/delete-back.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn close() -> Svg {
        Svg::from_path("./icons/close.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn list_settings() -> Svg {
        Svg::from_path("./icons/list-settings.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn settings() -> Svg {
        Svg::from_path("./icons/settings.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn maxmize() -> Svg {
        Svg::from_path("./icons/maxmize.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn minmize() -> Svg {
        Svg::from_path("./icons/minmize.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn reset() -> Svg {
        Svg::from_path("./icons/reset.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn refresh() -> Svg {
        Svg::from_path("./icons/refresh.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn search() -> Svg {
        Svg::from_path("./icons/search.svg")
            .width(iced::Length::Units(38))
            .height(iced::Length::Units(38))
    }
    pub fn enter() -> Svg {
        Svg::from_path("./icons/enter.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn cancel() -> Svg {
        Svg::from_path("./icons/cancel.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn edit() -> Svg {
        Svg::from_path("./icons/edit.svg")
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
}
