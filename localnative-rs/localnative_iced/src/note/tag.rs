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
    Edit {
        temp: String,
        reset: button::State,
        delete: button::State,
        edit: button::State,
        state: EditState,
    },
}
#[derive(Debug, Clone)]
pub enum EditState {
    Able,
    Editing(text_input::State),
}

impl Tag {
    pub fn is_editing(&self) -> bool {
        matches!(
            &self.state,
            State::Edit {
                state: EditState::Editing { .. },
                ..
            }
        )
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
            Message::InputChanged(change) => {
                if let State::Edit { temp, .. } = state {
                    *temp = change;
                }
            }
            Message::Reset => {
                if let State::Edit { temp, .. } = state {
                    *temp = name.clone();
                }
            }

            Message::Editable => {
                if let State::Normal { .. } = state {
                    *state = State::Edit {
                        temp: name.clone(),
                        state: EditState::Able,
                        edit: button::State::new(),
                        reset: button::State::new(),
                        delete: button::State::new(),
                    };
                }
            }
            Message::Editing => {
                if let State::Edit { state, .. } = state {
                    if let EditState::Able { .. } = state {
                        *state = EditState::Editing(focused_input());
                    }
                }
            }

            Message::CancelAdd => {
                if let State::Edit { temp, state, .. } = state {
                    if let EditState::Editing(..) = state {
                        temp.clear();
                        *state = EditState::Able;
                    }
                }
            }
            Message::Enter => {
                if let State::Edit {
                    state: edit_state,
                    temp,
                    ..
                } = state
                {
                    match edit_state {
                        EditState::Able => {
                            if name != temp {
                                *name = temp.clone();
                                log::info!("{}", name);
                            }
                            *state = State::Normal {
                                search: button::State::new(),
                            };
                        }
                        EditState::Editing(_) => {
                            *temp = temp.replace(&[',', '，', ' '][..], "");
                            *edit_state = EditState::Able;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    pub fn add_tag_view(&mut self) -> Element<Message> {
        let Tag { name, state } = self;
        match state {
            State::Edit {
                temp,
                delete,
                edit,
                state,
                ..
            } => {
                if let EditState::Editing(edit_input) = state {
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
                } else {
                    Row::new()
                        .push(
                            Button::new(edit, Text::new(name.as_str()).size(TAG_TEXT_SIZE))
                                .on_press(Message::Editing)
                                .style(crate::style::tag::Tag),
                        )
                        .into()
                }
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
            State::Edit {
                temp,
                edit,
                reset,
                delete,
                state,
            } => {
                let delete_button = Button::new(delete, Icon::delete_back())
                    .style(crate::style::symbol::Symbol)
                    .on_press(Message::Delete);
                if let EditState::Editing(edit_input) = state {
                    let input = TextInput::new(
                        edit_input,
                        temp.as_str(),
                        temp.as_str(),
                        Message::InputChanged,
                    )
                    .on_submit(Message::Enter);
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
                } else {
                    let edit = Row::new().push(
                        Button::new(edit, Text::new(temp.as_str()).size(TAG_TEXT_SIZE))
                            .style(crate::style::tag::Tag)
                            .on_press(Message::Editing),
                    );
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
            }
        }
    }
}

pub fn focused_input() -> text_input::State {
    let mut input = text_input::State::focused();
    input.move_cursor_to_end();
    input
}
pub const TAG_TEXT_SIZE: u16 = 16;
