use iced::Sandbox;
use ln_iced::NoteView;

fn main() -> iced::Result {
    NoteView::run(ln_iced::settings())
}
