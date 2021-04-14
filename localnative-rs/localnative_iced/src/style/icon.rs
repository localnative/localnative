use iced::Svg;
pub struct Icon;

const ICON_SIZE: u16 = 25;
impl Icon {
    pub fn delete() -> Svg {
        Svg::from_path(format!("{}/icon/delete.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
    }
    pub fn reset() -> Svg {
        Svg::from_path(format!("{}/icon/reset.svg", env!("CARGO_MANIFEST_DIR")))
            .width(iced::Length::Units(ICON_SIZE))
            .height(iced::Length::Units(ICON_SIZE))
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
