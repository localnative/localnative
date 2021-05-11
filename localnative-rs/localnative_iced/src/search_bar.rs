use iced::{button, text_input, Button, Element, Row, TextInput};

use crate::{style, tr};

#[derive(Debug, Default)]
pub struct SearchBar {
    pub search_text: String,
    pub state: State,
}

#[derive(Debug, Default)]
pub struct State {
    input: text_input::State,
    clear_button: button::State,
}
#[derive(Debug, Clone)]
pub enum Message {
    Search(String),
    Clear,
}
impl SearchBar {
    pub fn update(&mut self, message: Message) {
        if let Message::Clear = message {
            self.search_text.clear();
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let State {
            input,
            clear_button,
        } = &mut self.state;
        let input = TextInput::new(
            input,
            &tr!("search"),
            self.search_text.as_str(),
            Message::Search,
        )
        .on_submit(Message::Search(self.search_text.clone()))
        .size(35);
        let clear = Button::new(clear_button, style::icon::Icon::close())
            .on_press(Message::Clear)
            .style(style::symbol::Symbol);
        iced::Container::new(
            Row::new()
                .padding(10)
                .push(style::icon::Icon::search())
                .push(input)
                .push(clear),
        )
        .align_x(iced::Align::Center)
        .align_y(iced::Align::Start)
        .into()
    }
}
