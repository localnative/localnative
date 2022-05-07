use iced::{button, pick_list, Button, Checkbox, Column, Command, Element, Radio, Row, Text};
use iced_aw::{modal, number_input, Card, Modal, NumberInput};

use crate::{
    config::Config,
    sidebar::Sidebar,
    style::{self, Theme},
    tr,
    translate::{self, Language},
};
pub struct Settings {
    pub language_temp: Language,
    pub disable_delete_tip_temp: bool,
    pub limit_temp: u32,
    pub state: modal::State<State>,
}

#[derive(Default)]
pub struct State {
    pub save_button: button::State,
    pub cancel_button: button::State,
    pub try_fix_host: button::State,
    pub limit_input: number_input::State,
    pub language_selector: pick_list::State<Language>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Save,
    Cancel,
    TryFixHost,
    DisableTip(bool),
    LanguageChanged(Language),
    LimitChanged(u32),
    Other,
}
impl Settings {
    pub fn view<'settings, 'underlay: 'settings>(
        &'settings mut self,
        theme: Theme,
        underlay: Element<'underlay, Message>,
        config: &'underlay Config,
    ) -> Element<'settings, Message> {
        let Self { state, .. } = self;
        let disable_delete_tip = config.disable_delete_tip;
        let language = config.language;
        let limit = config.limit;
        Modal::new(state, underlay, move |state| {
            let ok_button =
                Button::new(&mut state.save_button, Text::new(tr!("ok"))).on_press(Message::Save);
            let cancel_button = Button::new(&mut state.cancel_button, Text::new(tr!("cancel")))
                .on_press(Message::Cancel);
            let disable_delete_tip =
                Checkbox::new(disable_delete_tip, "", Message::DisableTip).spacing(0);
            let try_fix_host = Button::new(&mut state.try_fix_host, Text::new(tr!("try-fix-host")))
                .on_press(Message::TryFixHost);
            // TODO: picklist not normaly work with modal
            // let language_selector = PickList::new(
            //     &mut state.language_selector,
            //     &[Language::English, Language::Chinese][..],
            //     Some(language),
            //     Message::LanguageChanged,
            // )
            // .padding(3)
            // .width(iced::Length::Shrink);
            let language_selector = Row::new()
                .push(style::horizontal_rule())
                .push(Radio::new(
                    Language::English,
                    tr!("english"),
                    Some(language),
                    Message::LanguageChanged,
                ))
                .push(Radio::new(
                    Language::Chinese,
                    tr!("chinese"),
                    Some(language),
                    Message::LanguageChanged,
                ))
                .push(style::horizontal_rule())
                .spacing(30);
            let limit_input =
                NumberInput::new(&mut state.limit_input, limit, 1000, Message::LimitChanged)
                    .min(5)
                    .step(1)
                    .padding(0);

            let body = Column::new()
                .push(
                    Row::new()
                        .push(Text::new(tr!("disable-delete-tip")))
                        .push(style::horizontal_rule())
                        .push(disable_delete_tip),
                )
                .push(
                    Column::new()
                        .push(Text::new(tr!("language")))
                        //.push(style::horizontal_rule())
                        .push(language_selector),
                )
                .push(
                    Row::new()
                        .push(Text::new(tr!("limit")))
                        .push(style::horizontal_rule())
                        .push(limit_input),
                )
                .push(try_fix_host)
                .align_items(iced::Alignment::Center)
                .padding(0)
                .spacing(10);
            Element::<'_, Message>::new(
                Card::new(Text::new(tr!("settings")), body)
                    .foot(
                        Row::new()
                            .push(style::horizontal_rule())
                            .push(ok_button)
                            .push(cancel_button)
                            .push(style::horizontal_rule())
                            .spacing(10)
                            .align_items(iced::Alignment::Center),
                    )
                    .on_close(Message::Cancel)
                    .max_width(400),
            )
        })
        .style(style::transparent(theme))
        .on_esc(Message::Cancel)
        .into()
    }
    pub fn update(
        &mut self,
        message: Message,
        config: &mut Config,
        sidebar: &mut Sidebar,
    ) -> Command<crate::Message> {
        match message {
            Message::Save => {
                self.state.show(false);
                sidebar.settings_is_open = false;
            }
            Message::Cancel => {
                self.state.show(false);
                sidebar.settings_is_open = false;
                config.disable_delete_tip = self.disable_delete_tip_temp;
                config.language = self.language_temp;
                config.limit = self.limit_temp;
                return Command::perform(
                    translate::init_bundle(config.language),
                    crate::Message::ApplyLanguage,
                );
            }
            Message::DisableTip(flag) => {
                config.disable_delete_tip = flag;
            }
            Message::LanguageChanged(language) => {
                config.language = language;
                return Command::perform(
                    translate::init_bundle(language),
                    crate::Message::ApplyLanguage,
                );
            }
            Message::LimitChanged(limit) => {
                config.limit = limit;
            }
            Message::TryFixHost => {
                return Command::perform(crate::init::WebKind::init_all(), crate::Message::InitHost)
            }
            Message::Other => {}
        }
        Command::none()
    }
}
