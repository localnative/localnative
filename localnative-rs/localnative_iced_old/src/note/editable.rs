use crate::{style::icon::Icon, tr};
use iced::{button, text_input, Button, Column, Element, Row, Text, TextInput};
use std::fmt::Display;

use super::tag::focused_input;

#[derive(Debug)]
pub struct Editables {
    pub title: String,
    pub url: String,
    pub description: String,
    pub comments: String,
    pub state: State,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Editable {
    Title,
    Url,
    Description,
    Comments,
    Menu,
}
impl Display for Editable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Editable::Title => f.write_str(&tr!("add-title")),
            Editable::Url => f.write_str(&tr!("add-url")),
            Editable::Description => f.write_str(&tr!("add-desp")),
            Editable::Comments => f.write_str(&tr!("add-comments")),
            Editable::Menu => f.write_str(&tr!("add-more")),
        }
    }
}
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum State {
    Normal {
        url_button: button::State,
    },
    Edit {
        title_edit: Edit,
        url_edit: Edit,
        description_edit: Edit,
        comments_edit: Edit,
    },
}
#[derive(Debug)]
pub enum Edit {
    Empty,
    Nonempty {
        temp: String,
        edit: button::State,
        reset: button::State,
        delete: button::State,
        state: EditState,
    },
}
#[derive(Debug)]
pub enum EditState {
    Editable,
    Editing { text_bar: text_input::State },
}

impl Edit {
    pub fn is_empty(&self) -> bool {
        match self {
            Edit::Empty => true,
            Edit::Nonempty { .. } => false,
        }
    }
    pub fn temp(&self) -> Option<String> {
        match self {
            Edit::Empty => None,
            Edit::Nonempty { temp, .. } => Some(temp.clone()),
        }
    }
    pub fn new(temp: &str) -> Self {
        if temp.is_empty() {
            Self::Empty
        } else {
            Self::Nonempty {
                temp: temp.to_string(),
                edit: button::State::new(),
                reset: button::State::new(),
                delete: button::State::new(),
                state: EditState::Editable,
            }
        }
    }
    pub fn update(&mut self, message: EditMessage, org: String) {
        match message {
            EditMessage::Delete => {
                // 上层处理
            }
            EditMessage::Edit => {
                if let Edit::Nonempty { state, .. } = self {
                    if let EditState::Editable = state  {
                        *state = EditState::Editing {
                            text_bar: focused_input(),
                        };
                    }
            }}
            ,
            // normal
            EditMessage::Reset => {
                if let Edit::Nonempty { temp, .. }= self {
                    *temp = org;
                }
        },
            // edting => editable
            EditMessage::Enter => {
                if let Edit::Nonempty { state, temp, .. }= self   {
                    if let EditState::Editing { .. } = state {
                        if temp.is_empty() {
                            *self = Edit::Empty;
                        } else {
                            *state = EditState::Editable;
                        }
                    }
                }}
            ,
            // edting
            EditMessage::InputChanged(changed) => {
                if let Edit::Nonempty { temp, state:EditState::Editing { .. }, .. } = self {
                        *temp = changed;
                }
            }
            ,
        }
    }
    pub fn view(&mut self, text: &str, info_text: &str) -> Element<EditMessage> {
        match self {
            Edit::Empty => unreachable!(),
            Edit::Nonempty {
                temp,
                edit,
                reset,
                delete,
                state,
            } => match state {
                EditState::Editable => {
                    let content = Text::new(temp.as_str());
                    let edit_button = Button::new(edit, content)
                        .style(crate::style::symbol::Symbol)
                        .on_press(EditMessage::Edit);
                    let delete_button = Button::new(delete, Icon::delete_back())
                        .style(crate::style::symbol::Symbol)
                        .on_press(EditMessage::Delete);
                    if temp.as_str() != text && !text.is_empty() {
                        let reset_button = Button::new(reset, Icon::reset())
                            .style(crate::style::symbol::Symbol)
                            .on_press(EditMessage::Reset);
                        Row::new()
                            .push(edit_button)
                            .push(reset_button)
                            .push(delete_button)
                    } else {
                        Row::new().push(edit_button).push(delete_button)
                    }
                    .into()
                }
                EditState::Editing { text_bar } => {
                    let input = TextInput::new(
                        text_bar,
                        {
                            if text.is_empty() {
                                info_text
                            } else {
                                text
                            }
                        },
                        {
                            if temp.is_empty() {
                                text
                            } else {
                                temp
                            }
                        },
                        EditMessage::InputChanged,
                    )
                    .on_submit(EditMessage::Enter);
                    let delete_button = Button::new(delete, Icon::delete_back())
                        .style(crate::style::symbol::Symbol)
                        .on_press(EditMessage::Delete);
                    if temp.as_str() != text {
                        let reset = Button::new(reset, Icon::reset())
                            .style(crate::style::symbol::Symbol)
                            .on_press(EditMessage::Reset);
                        Row::new().push(input).push(reset).push(delete_button)
                    } else {
                        Row::new().push(input).push(delete_button)
                    }
                    .into()
                }
            },
        }
    }
}
#[derive(Debug, Clone)]
pub enum EditMessage {
    Edit,
    Delete,
    Reset,
    Enter,
    InputChanged(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    Edit(Editable, EditMessage),
    OpenUrl,
    TurnEdit,
    Cancel,
    Enter,
}
macro_rules! if_empty_then_push {
    ($res:expr,$($need_judge:expr;$need_push:expr),*) => {
        $(
            if $need_judge.is_empty() {
                $res.push($need_push);
            }
        )*
    };
}
macro_rules! if_nonempty_then_push {
    ($res:expr,$($need_judge:expr;$need_push:expr),*) => {
        $(
            if !$need_judge.is_empty() {
                $res.push($need_push);
            }
        )*
    };
}
impl Editables {
    pub fn empty(&self) -> Vec<Editable> {
        let mut res = Vec::with_capacity(5);
        match &self.state {
            State::Normal { .. } => {
                if_empty_then_push!(
                    &mut res,
                    &self.title;Editable::Title,
                    &self.url;Editable::Url,
                    &self.description;Editable::Description,
                    &self.comments;Editable::Comments
                );
            }
            State::Edit {
                title_edit,
                url_edit,
                description_edit,
                comments_edit,
            } => {
                if_empty_then_push!(
                    &mut res,
                    title_edit;Editable::Title,
                    url_edit;Editable::Url,
                    description_edit;Editable::Description,
                    comments_edit;Editable::Comments
                );
            }
        }
        res
    }
    pub fn nonempty(&self) -> Vec<Editable> {
        let mut res = Vec::with_capacity(4);
        match &self.state {
            State::Normal { .. } => {
                if_nonempty_then_push!(
                    &mut res,
                    &self.title;Editable::Title,
                    &self.url;Editable::Url,
                    &self.description;Editable::Description,
                    &self.comments;Editable::Comments
                );
            }
            State::Edit {
                title_edit,
                url_edit,
                description_edit,
                comments_edit,
            } => {
                if_nonempty_then_push!(
                    &mut res,
                    title_edit;Editable::Title,
                    url_edit;Editable::Url,
                    description_edit;Editable::Description,
                    comments_edit;Editable::Comments
                );
            }
        }
        res
    }
    pub fn update(&mut self, message: Message) {
        match &mut self.state {
            State::Edit {
                title_edit,
                url_edit,
                description_edit,
                comments_edit,
            } => match message {
                Message::Edit(edtiable, message) => {
                    let edit = match edtiable {
                        Editable::Title => title_edit,
                        Editable::Url => url_edit,
                        Editable::Description => description_edit,
                        Editable::Comments => comments_edit,
                        _ => unreachable!(),
                    };

                    let org = match edtiable {
                        Editable::Title => self.title.as_str(),
                        Editable::Url => self.url.as_str(),
                        Editable::Description => self.description.as_str(),
                        Editable::Comments => self.comments.as_str(),
                        _ => unreachable!(),
                    };
                    match message {
                        EditMessage::Delete => {
                            *edit = Edit::Empty;
                        }
                        message => edit.update(message, org.to_string()),
                    }
                }
                Message::Cancel => {
                    self.state = State::Normal {
                        url_button: button::State::new(),
                    };
                }
                Message::Enter => {
                    let Editables {
                        title,
                        url,
                        description,
                        comments,
                        ..
                    } = self;
                    if let Some(temp) = title_edit.temp() {
                        *title = temp;
                    } else if !title.is_empty() {
                        title.clear();
                    }
                    if let Some(temp) = url_edit.temp() {
                        *url = temp;
                    } else if !url.is_empty() {
                        url.clear();
                    }
                    if let Some(temp) = description_edit.temp() {
                        *description = temp;
                    } else if !description.is_empty() {
                        description.clear();
                    }
                    if let Some(temp) = comments_edit.temp() {
                        *comments = temp;
                    } else if !comments.is_empty() {
                        comments.clear();
                    }
                    self.state = State::Normal {
                        url_button: button::State::new(),
                    };
                }
                _ => {}
            },
            State::Normal { .. } => match message {
                Message::TurnEdit => {
                    self.state = State::Edit {
                        title_edit: Edit::new(self.title.as_str()),
                        url_edit: Edit::new(self.url.as_str()),
                        description_edit: Edit::new(self.description.as_str()),
                        comments_edit: Edit::new(self.comments.as_str()),
                    };
                }
                Message::OpenUrl => {
                    if let Err(err) = open::that(self.url.as_str()) {
                        log::error!("open url fail:{:?}", err);
                    };
                }
                _ => {}
            },
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let Editables {
            title,
            url,
            description,
            comments,
            state,
        } = self;
        match state {
            State::Normal { url_button } => {
                let mut column = Column::new();
                column = if !title.is_empty() {
                    column.push(Text::new(title.as_str()))
                } else {
                    column
                };
                column = if !url.is_empty() {
                    column.push(
                        Button::new(url_button, Text::new(url.as_str()))
                            .on_press(Message::OpenUrl)
                            .style(crate::style::url::Url),
                    )
                } else {
                    column
                };
                column = if !description.is_empty() {
                    column.push(Text::new(description.as_str()))
                } else {
                    column
                };
                column = if !comments.is_empty() {
                    column.push(Text::new(comments.as_str()))
                } else {
                    column
                };
                column.into()
            }

            State::Edit {
                title_edit,
                url_edit,
                description_edit,
                comments_edit,
            } => {
                let mut column = Column::new();
                column = match title_edit {
                    Edit::Empty => column,
                    edit => column.push(
                        edit.view(title.as_str(), &tr!("add-title-info"))
                            .map(|m| Message::Edit(Editable::Title, m)),
                    ),
                };
                column = match url_edit {
                    Edit::Empty => column,
                    edit => column.push(
                        edit.view(url.as_str(), &tr!("add-url-info"))
                            .map(|m| Message::Edit(Editable::Url, m)),
                    ),
                };
                column = match description_edit {
                    Edit::Empty => column,
                    edit => column.push(
                        edit.view(description.as_str(), &tr!("add-desp-info"))
                            .map(|m| Message::Edit(Editable::Description, m)),
                    ),
                };
                column = match comments_edit {
                    Edit::Empty => column,
                    edit => column.push(
                        edit.view(comments.as_str(), &tr!("add-comments-info"))
                            .map(|m| Message::Edit(Editable::Comments, m)),
                    ),
                };
                column.into()
            }
        }
    }
}
