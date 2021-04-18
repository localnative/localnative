use iced::{button, Button, Element, Row, Text};

#[derive(Debug)]
pub struct PageBar {
    pub offset: u32,
    pub count: u32,
    pub state: State,
}

impl Default for PageBar {
    fn default() -> Self {
        Self {
            offset: 0,
            count: 1,
            state: State::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct State {
    pre_button: button::State,
    next_button: button::State,
}
#[derive(Debug, Clone)]
pub enum Message {
    Pre,
    Next,
}

impl PageBar {
    pub fn update(&mut self, message: Message, limit: u32) -> crate::Message {
        match message {
            Message::Pre => {
                if self.offset > 0 {
                    self.offset -= limit;
                    return crate::Message::NeedUpdate;
                }
            }
            Message::Next => {
                if self.offset < self.count {
                    self.offset += limit;
                    return crate::Message::NeedUpdate;
                }
            }
        }
        crate::Message::Ignore
    }
    pub fn view(&mut self, limit: u32) -> Element<Message> {
        let State {
            pre_button,
            next_button,
        } = &mut self.state;
        let pre = Button::new(pre_button, Text::new("<-"))
            .on_press(Message::Pre)
            .style(crate::style::symbol::Symbol);
        let next = Button::new(next_button, Text::new("->"))
            .on_press(Message::Next)
            .style(crate::style::symbol::Symbol);
        let page = Text::new(format!(
            "{}/{}",
            (self.offset + limit).min(self.count),
            self.count
        ));

        Row::new()
            .push(pre)
            .push(page)
            .push(next)
            .spacing(20)
            .align_items(iced::Align::Center)
            .into()
    }
}
