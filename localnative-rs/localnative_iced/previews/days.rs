use iced::Sandbox;
use localnative_iced::Chart;

fn main() -> iced::Result {
    Chart::run(localnative_iced::none_flags_settings())
}
