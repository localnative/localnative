use iced::Sandbox;
use localnative_iced::SearchPage;

fn main() -> iced::Result {
    SearchPage::run(localnative_iced::settings())
}
