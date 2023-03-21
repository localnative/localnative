use iced::Sandbox;
use localnative_iced::NewChart;

fn main() -> iced::Result {
    NewChart::run(localnative_iced::none_flags_settings())
}
