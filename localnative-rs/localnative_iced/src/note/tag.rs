use crate::style::icon::Icon;
use iced::{button, text_input, Button, Element, Row, Text, TextInput};

#[derive(Debug, Clone)]
pub enum Message {
    Search(String),
    // 上层处理
    Delete,
    EnterAdd(String),
    CancelAdd,
    // 直接处理
    Enter,
    InputChanged(String),
    Reset,
    Editable,
    Editing,
}
#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub state: State,
}
#[derive(Debug, Clone)]
pub enum State {
    Normal {
        search: button::State,
    },
    // TODO: 有些参数是相互重叠的，所以可以共用，最好添加一个新的State
    Editable {
        temp: String,
        edit: button::State,
        reset: button::State,
        delete: button::State,
    },
    Editing {
        temp: String,
        edit_input: text_input::State,
        reset: button::State,
        delete: button::State,
    },
}

impl Tag {
    pub fn is_editing(&self) -> bool {
        match &self.state {
            State::Editing { .. } => true,
            _ => false,
        }
    }
    pub fn new(name: &str, state: State) -> Self {
        Self {
            name: name.to_string(),
            state,
        }
    }
    pub fn update(&mut self, message: Message) {
        let Tag { name, state } = self;
        match message {
            Message::InputChanged(change) => match state {
                State::Editing { temp, .. } => {
                    *temp = change;
                }
                _ => {}
            },
            Message::Reset => match state {
                State::Editable { temp, .. } | State::Editing { temp, .. } => {
                    *temp = name.clone();
                }
                _ => {}
            },

            Message::Editable => {
                *state = State::Editable {
                    temp: name.clone(),
                    edit: button::State::new(),
                    reset: button::State::new(),
                    delete: button::State::new(),
                };
            }
            Message::Editing => match state {
                State::Editable { temp, .. } => {
                    *state = State::Editing {
                        temp: temp.clone(),
                        edit_input: focused_input(),
                        reset: button::State::new(),
                        delete: button::State::new(),
                    }
                }
                _ => {}
            },
            Message::CancelAdd => match state {
                State::Editing { .. } => {
                    *state = State::Editable {
                        temp: String::new(),
                        edit: button::State::new(),
                        reset: button::State::new(),
                        delete: button::State::new(),
                    };
                }
                _ => unreachable!(),
            },
            Message::Enter => {
                match state {
                    State::Editing { temp, .. } => {
                        *state = State::Editable {
                            temp: temp.clone(),
                            edit: button::State::new(),
                            reset: button::State::new(),
                            delete: button::State::new(),
                        };
                    }
                    _ => {}
                };
            }
            _ => {}
        }
    }
    // TODO:如果为了代码可读性，这里应该单独拎出来作为一个新的结构体，TagAdd的。。
    pub fn add_tag_view(&mut self) -> Element<Message> {
        let Tag { name, state } = self;
        match state {
            State::Editable { edit, .. } => Row::new()
                .push(
                    Button::new(edit, Text::new(name.as_str()).size(TAG_TEXT_SIZE))
                        .on_press(Message::Editing)
                        .style(crate::style::tag::Tag),
                )
                .into(),
            State::Editing {
                temp,
                edit_input,
                delete,
                ..
            } => {
                let input = TextInput::new(
                    edit_input,
                    name.as_str(),
                    temp.as_str(),
                    Message::InputChanged,
                )
                .on_submit(Message::EnterAdd(temp.clone()));
                let cancel_button = Button::new(delete, Icon::cancel())
                    .style(crate::style::symbol::Symbol)
                    .on_press(Message::CancelAdd);

                Row::new().push(input).push(cancel_button).into()
            }
            _ => unreachable!(),
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let Tag { state, name, .. } = self;
        match state {
            State::Normal { search } => {
                Button::new(search, Text::new(name.as_str()).size(TAG_TEXT_SIZE))
                    .on_press(Message::Search(name.clone()))
                    .style(crate::style::tag::Tag)
                    .into()
            }
            State::Editable {
                temp,
                edit,
                reset,
                delete,
            } => {
                let edit = Row::new().push(
                    Button::new(edit, Text::new(temp.as_str()).size(TAG_TEXT_SIZE))
                        .style(crate::style::tag::Tag)
                        .on_press(Message::Editing),
                );
                let delete_button = Button::new(delete, Icon::delete_back())
                    .style(crate::style::symbol::Symbol)
                    .on_press(Message::Delete);
                if name.as_str() != temp.as_str() {
                    edit.push(
                        Button::new(reset, Icon::reset())
                            .on_press(Message::Reset)
                            .style(crate::style::symbol::Symbol),
                    )
                    .push(delete_button)
                } else {
                    edit.push(delete_button)
                }
                .into()
            }
            State::Editing {
                temp,
                edit_input,
                reset,
                delete,
            } => {
                let input = TextInput::new(
                    edit_input,
                    temp.as_str(),
                    temp.as_str(),
                    Message::InputChanged,
                )
                .on_submit(Message::Enter);
                let delete_button = Button::new(delete, Icon::delete_back())
                    .style(crate::style::symbol::Symbol)
                    .on_press(Message::Delete);
                if name.as_str() != temp.as_str() {
                    Row::new()
                        .push(input)
                        .push(
                            Button::new(reset, Icon::reset())
                                .style(crate::style::symbol::Symbol)
                                .on_press(Message::Reset),
                        )
                        .push(delete_button)
                } else {
                    Row::new().push(input).push(delete_button)
                }
                .into()
            }
        }
    }
}

pub fn focused_input() -> text_input::State {
    let mut input = text_input::State::new();
    input.focus();
    input
}
pub const TAG_TEXT_SIZE: u16 = 16;
