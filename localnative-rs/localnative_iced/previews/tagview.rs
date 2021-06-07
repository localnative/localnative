use iced::Sandbox;
use localnative_iced::TagView;

fn main() -> iced::Result {
    TagView::run(localnative_iced::settings())
}
