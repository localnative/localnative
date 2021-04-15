use iced::Svg;
pub struct Icon;

const ICON_SIZE: u16 = 25;
impl Icon {
    pub fn qr_code() -> Svg {
        Svg::from_path(format!("{}/icon/qr-code.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn delete_bin() -> Svg {
        Svg::from_path(format!(
            "{}/icon/delete-bin.svg",
            env!("CARGO_MANIFEST_DIR")
        ))
        .width(iced::Length::Units(ICON_SIZE))
        .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn delete_back() -> Svg {
        Svg::from_path(format!(
            "{}/icon/delete-back.svg",
            env!("CARGO_MANIFEST_DIR")
        ))
        .width(iced::Length::Units(ICON_SIZE))
        .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn close() -> Svg {
        Svg::from_path(format!("{}/icon/close.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn list_settings() -> Svg {
        Svg::from_path(format!(
            "{}/icon/list-settings.svg",
            env!("CARGO_MANIFEST_DIR")
        ))
        .width(iced::Length::Units(ICON_SIZE))
        .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn settings() -> Svg {
        Svg::from_path(format!("{}/icon/settings.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn maxmize() -> Svg {
        Svg::from_path(format!("{}/icon/maxmize.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn minmize() -> Svg {
        Svg::from_path(format!("{}/icon/minmize.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn reset() -> Svg {
        Svg::from_path(format!("{}/icon/reset.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn refresh() -> Svg {
        Svg::from_path(format!("{}/icon/refresh.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn search() -> Svg {
        Svg::from_path(format!("{}/icon/search.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(38))
            .height(iced::Length::Units(38))
    }
    pub fn enter() -> Svg {
        Svg::from_path(format!("{}/icon/enter.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn cancel() -> Svg {
        Svg::from_path(format!("{}/icon/cancel.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn edit() -> Svg {
        Svg::from_path(format!("{}/icon/edit.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
}
