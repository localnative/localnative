use iced::Sandbox;
use localnative_iced::NoteView;

fn main() -> iced::Result {
    NoteView::run(localnative_iced::none_flags_settings())
}
