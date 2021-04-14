pub mod editable;
pub mod tag;

use iced::{
    button, pick_list, qr_code, Align, Button, Column, Container, PickList, QRCode, Row, Text,
};

use iced::Element;

use crate::style::icon::Icon;
use editable::Editables;
use tag::{focused_input, Tag};
#[derive(Debug, Clone)]
pub enum Message {
    TagMessage(usize, tag::Message),
    EditableMessage(editable::Message),
    EnableQrcode,
    DisableQrcode,
    Editable,
    Cancel,
    Enter,
    Delete,
    AddTag(String),
    AddTagMessage(tag::Message),
    AddEdit(editable::Editable),
}

pub type Note = localnative_core::Note;
#[derive(Debug)]
pub struct NoteView {
    pub rowid: i64,
    pub uuid4: String,
    pub editables: Editables,
    // TODO:annotations是图片信息。。
    pub annotations: String,
    pub tags_string: String,
    pub tags: Vec<tag::Tag>,
    pub created_at: String,
    pub is_public: bool,
    view_state: ViewState,
    qr_state: QrState,
}

// TODO: show、hide可以抽象成一个state。
#[derive(Debug)]
pub enum QrState {
    Hide {
        qrcode_show: button::State,
    },
    Show {
        qrcode_hide: button::State,
        qrcode: qr_code::State,
    },
}
#[derive(Debug)]
pub enum ViewState {
    Normal {
        edit: button::State,
        delete: button::State,
    },
    Edit {
        enter: button::State,
        cancel: button::State,
        add_editable: pick_list::State<editable::Editable>,
        add_tag: tag::Tag,
    },
}
impl Into<NoteView> for Note {
    fn into(self) -> NoteView {
        let tags: Vec<Tag> = self
            .tags
            .split(",")
            .into_iter()
            .filter(|name| !name.is_empty())
            .map(|name| {
                Tag::new(
                    name,
                    tag::State::Normal {
                        search: button::State::new(),
                    },
                )
            })
            .collect();
        let editables = Editables {
            title: self.title,
            url: self.url,
            description: self.description,
            comments: self.comments,
            state: editable::State::Normal {
                url_button: button::State::new(),
            },
        };
        NoteView {
            rowid: self.rowid,
            uuid4: self.uuid4,
            editables,
            annotations: self.annotations,
            tags_string: self.tags,
            tags,
            created_at: self.created_at,
            is_public: self.is_public,
            view_state: ViewState::Normal {
                edit: button::State::new(),
                delete: button::State::new(),
            },
            qr_state: QrState::Hide {
                qrcode_show: button::State::new(),
            },
        }
    }
}
impl Into<Note> for &NoteView {
    fn into(self) -> Note {
        let tags = self
            .tags
            .iter()
            .map(|tag| tag.name.as_str())
            .collect::<Vec<&str>>()
            .join(",");
        Note {
            rowid: self.rowid,
            uuid4: self.uuid4.clone(),
            title: self.editables.title.clone(),
            url: self.editables.url.clone(),
            tags,
            description: self.editables.description.clone(),
            comments: self.editables.comments.clone(),
            annotations: self.annotations.clone(),
            created_at: self.created_at.clone(),
            is_public: self.is_public,
        }
    }
}

impl NoteView {
    pub fn old_note(&self) -> Note {
        Note {
            rowid: self.rowid,
            uuid4: self.uuid4.clone(),
            title: self.editables.title.clone(),
            url: self.editables.url.clone(),
            tags: self.tags_string.clone(),
            description: self.editables.description.clone(),
            comments: self.editables.comments.clone(),
            annotations: self.annotations.clone(),
            created_at: self.created_at.clone(),
            is_public: self.is_public,
        }
    }
    pub fn qr_code(&self) -> &[u8] {
        self.editables.url.as_bytes()
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::TagMessage(idx, tm) => match tm {
                tag::Message::Delete => {
                    self.tags.remove(idx);
                }
                tag::Message::Search(_) => {
                    // 顶层处理了，所以这里不用管
                }
                tm => {
                    if let Some(tag) = self.tags.get_mut(idx) {
                        tag.update(tm);
                    }
                }
            },
            Message::EditableMessage(em) => {
                self.editables.update(em);
            }
            Message::EnableQrcode => match self.qr_state {
                QrState::Hide { .. } => {
                    let qrcode = if let Ok(qr_state) = qr_code::State::with_version(
                        self.qr_code(),
                        qr_code::Version::Normal(8),
                        qr_code::ErrorCorrection::Low,
                    ) {
                        qr_state
                    } else {
                        qr_code::State::new(self.qr_code()).unwrap_or(
                            // 如果到了这里，都出错，只能panic了
                            qr_code::State::new("Error in qrcode generation.").unwrap(),
                        )
                    };

                    self.qr_state = QrState::Show {
                        qrcode_hide: button::State::new(),
                        qrcode,
                    };
                }
                _ => {}
            },
            Message::DisableQrcode => match self.qr_state {
                QrState::Show { .. } => {
                    self.qr_state = QrState::Hide {
                        qrcode_show: button::State::new(),
                    };
                }
                _ => {}
            },
            Message::Editable => {
                self.editables.update(editable::Message::TurnEdit);
                self.tags
                    .iter_mut()
                    .for_each(|tag| tag.update(tag::Message::Editable));
                let NoteView { view_state, .. } = self;
                match view_state {
                    ViewState::Normal { .. } => {
                        *view_state = ViewState::Edit {
                            enter: button::State::new(),
                            cancel: button::State::new(),
                            add_editable: pick_list::State::default(),
                            add_tag: tag::Tag::new(
                                "Add new tag",
                                tag::State::Editable {
                                    temp: String::new(),
                                    edit: button::State::new(),
                                    reset: button::State::new(),
                                    delete: button::State::new(),
                                },
                            ),
                        };
                    }
                    _ => {}
                }
            }
            Message::Cancel => {
                self.editables.update(editable::Message::Cancel);
                self.cancel_tags();
                let NoteView { view_state, .. } = self;
                match view_state {
                    ViewState::Edit { .. } => {
                        *view_state = ViewState::Normal {
                            edit: button::State::new(),
                            delete: button::State::new(),
                        };
                    }
                    _ => {}
                }
            }
            Message::Enter => {
                let NoteView {
                    tags,
                    view_state,
                    editables,
                    ..
                } = self;
                match view_state {
                    ViewState::Edit { add_tag, .. } => {
                        let Tag { state, .. } = add_tag;
                        let temp = match state {
                            tag::State::Editing { temp, .. } => Some(temp.clone()),
                            _ => None,
                        };
                        if let Some(temp) = temp {
                            add_tag.update(tag::Message::EnterAdd(temp));
                        }
                    }
                    _ => unreachable!(),
                }
                editables.update(editable::Message::Enter);
                tags.iter_mut()
                    .filter(|tag| tag.is_editing())
                    .for_each(|tag| tag.update(tag::Message::Enter));
            }

            Message::Delete => {
                // 上层处理
            }
            Message::AddTag(tag) => {
                let NoteView {
                    tags, view_state, ..
                } = self;
                if !tag.is_empty() {
                    tag.split(&[' ', ',', '，'][..])
                        .into_iter()
                        .filter(|tag| !tag.is_empty())
                        .for_each(|tag| {
                            tags.push(Tag::new(
                                tag,
                                tag::State::Editable {
                                    temp: tag.to_string(),
                                    edit: button::State::new(),
                                    reset: button::State::new(),
                                    delete: button::State::new(),
                                },
                            ));
                        });
                }
                match view_state {
                    ViewState::Edit { add_tag, .. } => {
                        let Tag { state, .. } = add_tag;
                        match state {
                            tag::State::Editing { .. } => {
                                *state = tag::State::Editable {
                                    temp: String::new(),
                                    edit: button::State::new(),
                                    reset: button::State::new(),
                                    delete: button::State::new(),
                                };
                            }
                            tag::State::Editable { temp, .. } => temp.clear(),
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Message::AddTagMessage(atm) => {
                let NoteView { view_state, .. } = self;
                match view_state {
                    ViewState::Edit { add_tag, .. } => add_tag.update(atm),
                    _ => unreachable!(),
                }
            }
            Message::AddEdit(edit) => {
                let NoteView { editables, .. } = self;
                let Editables { state, .. } = editables;
                let edit = match state {
                    editable::State::Edit {
                        title_edit,
                        url_edit,
                        description_edit,
                        comments_edit,
                    } => match edit {
                        editable::Editable::Title => title_edit,
                        editable::Editable::Url => url_edit,
                        editable::Editable::Description => description_edit,
                        editable::Editable::Comments => comments_edit,
                        editable::Editable::Menu => unreachable!(),
                    },
                    _ => unreachable!(),
                };
                match edit {
                    editable::Edit::Empty => {
                        *edit = editable::Edit::Nonempty {
                            temp: String::new(),
                            edit: button::State::new(),
                            reset: button::State::new(),
                            delete: button::State::new(),
                            state: editable::EditState::Editing {
                                text_bar: focused_input(),
                            },
                        };
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    pub fn new(note: Note, view_state: ViewState, qr_state: QrState) -> Self {
        let tags = note
            .tags
            .split(",")
            .map(|name| {
                let tag_state = match view_state {
                    ViewState::Normal { .. } => tag::State::Normal {
                        search: button::State::new(),
                    },
                    ViewState::Edit { .. } => tag::State::Editable {
                        temp: name.to_string(),
                        edit: button::State::new(),
                        reset: button::State::new(),
                        delete: button::State::new(),
                    },
                };
                Tag::new(name, tag_state)
            })
            .collect::<Vec<Tag>>();
        let editables = Editables {
            title: note.title,
            url: note.url,
            description: note.description,
            comments: note.comments,
            state: editable::State::Normal {
                url_button: button::State::new(),
            },
        };
        Self {
            rowid: note.rowid,
            uuid4: note.uuid4,
            editables,
            tags_string: note.tags,
            annotations: note.annotations,
            tags,
            created_at: note.created_at,
            is_public: note.is_public,
            view_state,
            qr_state,
        }
    }
    pub fn cancel_tags(&mut self) {
        self.tags = self
            .tags_string
            .split(",")
            .filter(|s| !s.is_empty())
            .map(|name| {
                let tag_state = tag::State::Normal {
                    search: button::State::new(),
                };
                Tag::new(name, tag_state)
            })
            .collect::<Vec<Tag>>();
    }
    pub fn view(&mut self) -> Element<Message> {
        let NoteView {
            uuid4,
            editables,
            tags,
            created_at,
            view_state,
            qr_state,
            ..
        } = self;

        let qrcode_button;
        let qrcode = match qr_state {
            QrState::Hide { qrcode_show } => {
                qrcode_button =
                    Button::new(qrcode_show, Text::new("QR")).on_press(Message::EnableQrcode);
                None
            }
            QrState::Show {
                qrcode_hide,
                qrcode,
            } => {
                qrcode_button =
                    Button::new(qrcode_hide, Text::new("QR")).on_press(Message::DisableQrcode);
                Some(QRCode::new(qrcode))
            }
        };
        let info_row = Row::new()
            .push(Text::new(created_at.as_str()).size(15))
            .push(Text::new(uuid4.as_str()).size(15))
            .push(Text::new(format!("rowid {}", self.rowid)).size(15))
            .spacing(5)
            .align_items(Align::Start);
        let mut wrap = tags.iter_mut().enumerate().fold(
            crate::wrap::Wrap {
                line_height: 30,
                spacing: 5,
                padding: 3,
                line_spacing: 5,
                // horizontal_alignment: Align::End,
                // vertical_alignment: Align::End,
                ..Default::default()
            }
            .push(info_row.into())
            .push(qrcode_button.into()),
            |wrap, (idx, tag)| wrap.push(tag.view().map(move |m| Message::TagMessage(idx, m))),
        );

        match view_state {
            ViewState::Normal { edit, delete } => {
                let edit_button = Button::new(edit, Icon::edit())
                    .style(crate::style::symbol::Symbol)
                    .on_press(Message::Editable);
                let delete_button = Button::new(delete, Icon::delete())
                    .style(crate::style::symbol::Symbol)
                    .on_press(Message::Delete);
                let op = Column::new()
                    .align_items(iced::Align::End)
                    .width(iced::Length::Fill)
                    .push(
                        Row::new()
                            .align_items(iced::Align::End)
                            .push(edit_button)
                            .push(delete_button)
                            .padding(20)
                            .spacing(10),
                    );
                let editables_view = editables.view().map(|m| Message::EditableMessage(m));
                let editables_row = Row::new().push(editables_view);
                if let Some(qrcode) = qrcode {
                    Container::new(
                        Column::new()
                            .push(wrap)
                            .push(center_qrcode(qrcode))
                            .push(editables_row)
                            .push(op),
                    )
                } else {
                    Container::new(Column::new().push(wrap).push(editables_row).push(op))
                }
                .into()
            }

            ViewState::Edit {
                enter,
                cancel,
                add_editable,
                add_tag,
            } => {
                let enter_button = Button::new(enter, Icon::enter())
                    .style(crate::style::symbol::Symbol)
                    .on_press(Message::Enter);
                let cancel_button = Button::new(cancel, Icon::cancel())
                    .style(crate::style::symbol::Symbol)
                    .on_press(Message::Cancel);
                let empty = editables.empty();
                let op = if !empty.is_empty() {
                    let add_edit = PickList::new(
                        add_editable,
                        empty,
                        Some(editable::Editable::Menu),
                        Message::AddEdit,
                    );
                    Column::new()
                        .align_items(iced::Align::End)
                        .width(iced::Length::Fill)
                        .push(
                            Row::new()
                                .align_items(iced::Align::End)
                                .push(add_edit)
                                .push(enter_button)
                                .push(cancel_button)
                                .padding(20)
                                .spacing(10),
                        )
                } else {
                    Column::new()
                        .align_items(iced::Align::End)
                        .width(iced::Length::Fill)
                        .push(
                            Row::new()
                                .align_items(iced::Align::End)
                                .push(enter_button)
                                .push(cancel_button)
                                .padding(20)
                                .spacing(10),
                        )
                };
                wrap = wrap.push(add_tag.add_tag_view().map(|atm| match atm {
                    tag::Message::EnterAdd(text) => Message::AddTag(text),
                    tm => Message::AddTagMessage(tm),
                }));
                let editables_view = editables.view().map(|m| Message::EditableMessage(m));

                let editables_row = Row::new()
                    .push(editables_view)
                    .align_items(iced::Align::Start)
                    .width(iced::Length::Fill);
                if let Some(qrcode) = qrcode {
                    Container::new(
                        Column::new()
                            .push(wrap)
                            .push(center_qrcode(qrcode))
                            .push(editables_row)
                            .push(op),
                    )
                } else {
                    Container::new(Column::new().push(wrap).push(editables_row).push(op))
                }
                .into()
            }
        }
    }
}
pub fn center_qrcode(qrcode: QRCode) -> Element<Message> {
    Column::new()
        .push(qrcode)
        .align_items(iced::Align::Center)
        .width(iced::Length::Fill)
        .into()
}
