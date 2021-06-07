mod note;
mod search_bar;
mod style;
mod tags;
use iced::Command;
pub use note::NoteView;
pub use tags::TagView;

pub enum LocalNative {
    Loading,
    Loaded(Data),
}

pub struct Data {}

#[derive(Debug)]
pub enum Message {
    NoteMessage(note::Message),
    TagsMessage(tags::Message),
}

impl iced::Application for LocalNative {
    type Executor = iced::executor::Default;

    type Message = Message;

    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (LocalNative::Loading, Command::none())
    }

    fn title(&self) -> String {
        "ln-iced".to_owned()
    }

    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        iced::Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        iced::Text::new("ln-iced").into()
    }
}

pub fn settings() -> iced::Settings<()> {
    iced::Settings {
        ..Default::default()
    }
}
