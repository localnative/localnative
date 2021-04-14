use iced::{button, Button, Element, Row, Text};

#[derive(Debug)]
pub struct PageBar {
    pub page_num: u32,
    pub offset: u32,
    pub count: u32,
    pub state: State,
}

impl Default for PageBar {
    fn default() -> Self {
        Self {
            page_num: 1,
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
                if self.page_num > 1 {
                    self.page_num -= 1;
                    self.offset -= limit;
                    return crate::Message::NeedUpdate;
                }
            }
            Message::Next => {
                let max = self.count / limit + if self.count % limit != 0 { 1 } else { 0 };
                if self.page_num < max {
                    self.page_num += 1;
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
        let pre = Button::new(pre_button, Text::new("<-")).on_press(Message::Pre);
        let next = Button::new(next_button, Text::new("->")).on_press(Message::Next);
        let page = Text::new(format!(
            "{}/{}",
            self.page_num,
            self.count / limit + if self.count % limit != 0 { 1 } else { 0 }
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
