use iced::{button, text_input, Button, Element, Row, Text, TextInput};

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
        match message {
            Message::Clear => {
                self.search_text.clear();
            }
            _ => {}
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let State {
            input,
            clear_button,
        } = &mut self.state;
        let input = TextInput::new(
            input,
            "Type your search...",
            self.search_text.as_str(),
            Message::Search,
        )
        .on_submit(Message::Search(self.search_text.clone()))
        .size(35);
        let clear = Button::new(clear_button, Text::new("X").size(35)).on_press(Message::Clear);
        iced::Container::new(Row::new().push(input).spacing(5).push(clear))
            .align_x(iced::Align::Center)
            .align_y(iced::Align::Start)
            .into()
    }
}
