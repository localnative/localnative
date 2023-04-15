mod chart;
mod config;
mod days;
mod delete_tip;
mod icons;
mod init;
mod middle_date;
mod note;
mod search_page;
mod settings;
mod sidebar;
mod style;
mod sync;
mod tags;
mod translate;

use std::cmp::Ordering;
use std::sync::Arc;

use chart::ChartView;
#[cfg(feature = "preview")]
pub use chart::NewChart;
use config::{Config, ThemeType};
pub use days::DateView;
use delete_tip::DeleteTip;
use iced::{futures::lock::Mutex, widget::container, Command};
use iced::{
    widget::{column, horizontal_space, row, text, vertical_space},
    Length::Fill,
    Theme,
};
use iced_native::window;
use iced_native::Event;
use iced_native::{command, event::Status};
use localnative_core::rpc::server::Stop;
use localnative_core::{exe::get_sqlite_connection, rusqlite::Connection};
use middle_date::MiddleDate;
pub use note::NoteView;
use once_cell::sync::OnceCell;
pub use search_page::SearchPage;
use sidebar::Sidebar;
pub use tags::TagView;

use crate::sync::SyncView;

pub struct LocalNative {
    config: Config,
    theme: ThemeType,
    state: State,
}
#[allow(clippy::large_enum_variant)]
pub enum State {
    Loading,
    Loaded(Data),
}

pub type Conn = Arc<Mutex<Connection>>;

pub struct Data {
    search_page: SearchPage,
    sidebar: Sidebar,
    delete_tip: DeleteTip,
    settings: settings::Settings,
    sync_view: SyncView,
    conn: Conn,
}

#[derive(Debug)]
pub enum Message {
    Loading(()),
    SearchPageMessage(search_page::Message),
    SidebarMessage(sidebar::Message),
    DeleteTipMessage(delete_tip::Message),
    SyncClientMessage(sync::Message),
    SettingsMessage(settings::Message),
    NoteView(Vec<NoteView>),
    TagView(Vec<TagView>),
    DayView(chart::ChartView),
    RequestClosed,
    ApplyLanguage(Option<()>),
    CloseWindow(Option<()>),
    SyncResult(anyhow::Result<()>),
    SyncOption(Option<()>),
    StartServerResult(std::io::Result<Stop>),
    ServerOption(Option<()>),
    InitHost(()),
    Receiver(Option<MiddleDate>),
}

impl iced::Application for LocalNative {
    type Executor = iced::executor::Default;

    type Message = Message;

    type Flags = Option<Config>;

    type Theme = Theme;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let is_first_open = flags.is_none();
        let config = flags.unwrap_or_default();
        let theme = config.theme;
        let language = config.language;
        (
            LocalNative {
                config,
                state: State::Loading,
                theme: theme.into(),
            },
            Command::batch([
                Command::perform(async {}, Message::Loading),
                Command::perform(translate::init_bundle(language), Message::ApplyLanguage),
                if is_first_open {
                    Command::perform(init::WebKind::init_all(), Message::InitHost)
                } else {
                    Command::none()
                },
            ]),
        )
    }

    fn title(&self) -> String {
        let version = env!("CARGO_PKG_VERSION");
        format!("Local Native {}", version)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        let LocalNative { config, state, .. } = self;
        match state {
            State::Loading => match message {
                Message::Loading(..) => {
                    let conn = Arc::new(Mutex::new(get_sqlite_connection()));

                    let data = Data {
                        search_page: SearchPage::default_with_theme(config.theme),
                        sidebar: Sidebar::default(),
                        delete_tip: DeleteTip {
                            rowid: -1,
                            show_modal: false,
                        },
                        sync_view: SyncView::default(),
                        settings: settings::Settings {
                            disable_delete_tip_temp: config.disable_delete_tip,
                            language_temp: config.language,
                            limit_temp: config.limit,
                            show_modal: false,
                        },
                        conn,
                    };

                    self.state = State::Loaded(data);
                    if let State::Loaded(ref mut data) = self.state {
                        let Data {
                            conn, search_page, ..
                        } = data;

                        Command::perform(
                            MiddleDate::upgrade(
                                conn.clone(),
                                search_page.search_value.clone(),
                                config.limit,
                                search_page.offset,
                            ),
                            Message::Receiver,
                        )
                    } else {
                        unreachable!()
                    }
                }
                Message::InitHost(_) => Command::none(),
                _ => Command::none(),
            },
            State::Loaded(data) => match message {
                Message::Receiver(Some(md)) => {
                    let MiddleDate {
                        tags,
                        notes,
                        count,
                        days,
                    } = md;
                    data.search_page.count = count;
                    // TODO:
                    if data.search_page.offset > count && notes.is_empty() {
                        data.search_page.offset = count.max(config.limit) - config.limit;
                        return search_page::search(
                            data.conn.clone(),
                            data.search_page.search_value.clone(),
                            config.limit,
                            data.search_page.offset,
                            data.search_page.range,
                        );
                    }
                    Command::batch([
                        Command::perform(
                            async move {
                                let mut tags = tags;
                                tags.sort_by(|a, b| {
                                    let ord = b.count.cmp(&a.count);
                                    if ord == Ordering::Equal {
                                        b.name.cmp(&a.name)
                                    } else {
                                        ord
                                    }
                                });

                                tags.into_iter().map(TagView::from).collect()
                            },
                            Message::TagView,
                        ),
                        Command::perform(
                            async move { notes.into_iter().map(NoteView::from).collect() },
                            Message::NoteView,
                        ),
                        {
                            if let Some(days) = days {
                                Command::perform(
                                    async move { ChartView::from_days(days) },
                                    Message::DayView,
                                )
                            } else {
                                Command::none()
                            }
                        },
                    ])
                }
                Message::SearchPageMessage(spmsg) => {
                    let Data {
                        search_page,
                        delete_tip,
                        ..
                    } = data;
                    search_page.update(
                        spmsg,
                        self.config.limit,
                        data.conn.clone(),
                        true,
                        delete_tip,
                    )
                }
                Message::NoteView(notes) => {
                    data.search_page.notes = notes;
                    Command::none()
                }
                Message::TagView(tags) => {
                    data.search_page.tags = tags;
                    Command::none()
                }
                Message::Loading(..) => Command::none(),
                Message::DayView(chart) => {
                    data.search_page.days.chart.view = chart;
                    Command::none()
                }
                Message::ApplyLanguage(..) => Command::none(),
                Message::RequestClosed => {
                    config.date_filter_is_show = data.search_page.days.is_show;
                    let json = serde_json::to_string_pretty(&*config).unwrap();
                    Command::perform(config::save(json), Message::CloseWindow)
                }
                Message::CloseWindow(res) => {
                    if res.is_some() {
                        println!("ok!");
                    }
                    Command::single(command::Action::Window(window::Action::Close))
                }
                Message::SidebarMessage(smsg) => {
                    let Data {
                        sidebar, settings, ..
                    } = data;
                    if matches!(smsg, sidebar::Message::ThemeChanged) {
                        data.search_page.days.chart.style = !data.search_page.days.chart.style;
                    }
                    sidebar.update(smsg, settings, config, &mut self.theme)
                }
                Message::DeleteTipMessage(msg) => {
                    let Data {
                        search_page,
                        delete_tip,
                        conn,
                        ..
                    } = data;
                    match msg {
                        delete_tip::Message::Enter => {
                            delete_tip.show_modal = false;
                            Command::perform(
                                MiddleDate::delete(
                                    conn.clone(),
                                    search_page.search_value.to_string(),
                                    self.config.limit,
                                    search_page.offset,
                                    delete_tip.rowid,
                                ),
                                Message::Receiver,
                            )
                        }
                        delete_tip::Message::SearchPage(spmsg) => search_page.update(
                            spmsg,
                            self.config.limit,
                            conn.clone(),
                            self.config.disable_delete_tip,
                            delete_tip,
                        ),
                        delete_tip::Message::Cancel => {
                            delete_tip.show_modal = false;
                            Command::none()
                        }
                    }
                }
                Message::SyncClientMessage(sync_msg) => {
                    data.sync_view.update(sync_msg, data.conn.clone())
                }
                Message::SyncResult(res) => {
                    if let Err(err) = res {
                        if let Some(io_error) = err.downcast_ref::<std::io::Error>() {
                            data.sync_view.with_sync_state_mut(|state| {
                                *state = sync::SyncState::SyncError(io_error.kind())
                            });
                        }
                        Command::none()
                    } else {
                        data.sync_view
                            .with_sync_state_mut(|state| *state = sync::SyncState::Complete);

                        search_page::search(
                            data.conn.clone(),
                            data.search_page.search_value.clone(),
                            config.limit,
                            data.search_page.offset,
                            data.search_page.range,
                        )
                    }
                }
                Message::SyncOption(opt) => {
                    if opt.is_none() {
                        data.sync_view.with_sync_state_mut(|state| {
                            *state = sync::SyncState::SyncFromFileUnknownError
                        });

                        Command::none()
                    } else {
                        data.sync_view
                            .with_sync_state_mut(|state| *state = sync::SyncState::Complete);
                        search_page::search(
                            data.conn.clone(),
                            data.search_page.search_value.clone(),
                            config.limit,
                            data.search_page.offset,
                            data.search_page.range,
                        )
                    }
                }
                Message::StartServerResult(res) => {
                    match res {
                        Ok(stop) => {
                            let is_none = sync::get_ip()
                                .and_then(|ip| {
                                    let addr = ip
                                        + ":"
                                        + data.sync_view.borrow_port().to_string().as_str();
                                    let state = iced::widget::qr_code::State::new(&addr).ok();
                                    state.map(|state| (addr, state))
                                })
                                .map(|(addr, state)| {
                                    data.sync_view.with_server_state_mut(|state| {
                                        *state = sync::ServerState::Opened
                                    });
                                    data.sync_view.update_server_addr(addr);
                                    data.sync_view
                                        .with_ip_qr_code_mut(|qr_code| *qr_code = state);
                                })
                                .is_none();
                            if is_none {
                                data.sync_view.with_server_state_mut(|state| {
                                    *state = sync::ServerState::Closed
                                });
                                if let Some(cmd) = data.sync_view.with_stop_mut(|ref_mut_stop| {
                                    if let Some(old_stop) = ref_mut_stop.take() {
                                        Some(Command::batch([
                                            Command::perform(
                                                sync::stop_server(old_stop),
                                                Message::ServerOption,
                                            ),
                                            Command::perform(
                                                sync::stop_server(stop),
                                                Message::ServerOption,
                                            ),
                                        ]))
                                    } else {
                                        None
                                    }
                                }) {
                                    return cmd;
                                }
                            } else {
                                if data.sync_view.borrow_stop().is_some() {
                                    if let Some(cmd) =
                                        data.sync_view.with_stop_mut(|ref_mut_stop| {
                                            if let Some(old_stop) = ref_mut_stop.take() {
                                                ref_mut_stop.replace(stop);
                                                Some(Command::perform(
                                                    sync::stop_server(old_stop),
                                                    Message::ServerOption,
                                                ))
                                            } else {
                                                None
                                            }
                                        })
                                    {
                                        return cmd;
                                    }
                                } else {
                                    data.sync_view.with_stop_mut(|ref_mut_stop| {
                                        ref_mut_stop.replace(stop);
                                    });
                                }
                            }
                        }
                        Err(err) => {
                            data.sync_view.with_sync_state_mut(|state| {
                                *state = sync::SyncState::SyncError(err.kind())
                            });
                        }
                    }
                    Command::none()
                }
                Message::ServerOption(opt) => {
                    if opt.is_some() {
                        data.sync_view
                            .with_server_state_mut(|state| *state = sync::ServerState::Closed);
                    } else {
                        data.sync_view
                            .with_server_state_mut(|state| *state = sync::ServerState::Error);
                    }
                    Command::none()
                }
                Message::SettingsMessage(msg) => {
                    let Data {
                        settings, sidebar, ..
                    } = data;
                    settings.update(msg, config, sidebar)
                }
                Message::InitHost(..) => Command::none(),
                Message::Receiver(None) => Command::none(),
            },
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced_native::subscription::events_with(events_handler)
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let LocalNative { config, state, .. } = self;
        match state {
            State::Loading => column![
                vertical_space(Fill),
                row![
                    horizontal_space(Fill),
                    text("Loading...").size(50),
                    horizontal_space(Fill)
                ],
                vertical_space(Fill),
            ]
            .into(),
            State::Loaded(data) => {
                let Data {
                    search_page,
                    sidebar,
                    delete_tip,
                    sync_view: sync_client,
                    settings,
                    ..
                } = data;

                let mut page = match sidebar.state {
                    sidebar::State::SearchPage => {
                        if config.disable_delete_tip {
                            search_page
                                .view(config.limit)
                                .map(Message::SearchPageMessage)
                        } else {
                            delete_tip
                                .view(config.limit, search_page)
                                .map(Message::DeleteTipMessage)
                        }
                    }
                    sidebar::State::SyncView => sync_client.view().map(Message::SyncClientMessage),
                };
                if sidebar.settings_is_open {
                    page = settings
                        .view(page.map(|_| crate::settings::Message::Other), config)
                        .map(Message::SettingsMessage);
                }
                row![
                    container(sidebar.view(&self.theme).map(Message::SidebarMessage))
                        .width(iced::Length::Shrink),
                    container(page)
                ]
                .into()
            }
        }
    }

    fn theme(&self) -> Self::Theme {
        self.theme.into()
    }
}
fn events_handler(event: Event, states: Status) -> Option<Message> {
    if states == Status::Ignored {
        if let Event::Window(window::Event::CloseRequested) = event {
            return Some(Message::RequestClosed);
        }
    }
    None
}
pub fn logo() -> Option<iced::window::Icon> {
    image::load_from_memory_with_format(
        include_bytes!("../../icons/icon.ico"),
        image::ImageFormat::Ico,
    )
    .ok()
    .and_then(|dyn_img| {
        let img = dyn_img.to_rgb8();
        let (width, height) = img.dimensions();
        iced::window::icon::from_rgba(img.into_raw(), width, height).ok()
    })
}

pub fn settings() -> iced::Settings<Option<Config>> {
    iced::Settings {
        default_font: font(),
        exit_on_close_request: false,
        flags: Config::load(),
        window: iced::window::Settings {
            size: (1080, 720),
            icon: logo(),
            ..Default::default()
        },
        ..Default::default()
    }
}
pub fn none_flags_settings() -> iced::Settings<()> {
    iced::Settings {
        default_font: font(),
        exit_on_close_request: false,
        ..Default::default()
    }
}
static FONT: OnceCell<Option<Vec<u8>>> = OnceCell::new();

fn font() -> Option<&'static [u8]> {
    FONT.get_or_init(|| {
        use iced_graphics::font::Family;
        let source = iced_graphics::font::Source::new();

        source
            .load(&[
                Family::Title("PingFang SC".to_owned()),
                Family::Title("Hiragino Sans GB".to_owned()),
                Family::Title("Heiti SC".to_owned()),
                Family::Title("Microsoft YaHei".to_owned()),
                Family::Title("WenQuanYi Micro Hei".to_owned()),
                Family::Title("Microsoft YaHei".to_owned()),
                // TODO:iced 目前没有字体fallback，所以我们只能尽可能选择中英文支持的字体
                Family::Title("Helvetica".to_owned()),
                Family::Title("Tahoma".to_owned()),
                Family::Title("Arial".to_owned()),
                Family::SansSerif,
            ])
            .ok()
    })
    .as_ref()
    .map(|f| f.as_slice())
}

// pub fn handle_notes(notes: Vec<Note>) -> (Vec<NoteView>) {
//     for note in notes {
//         let time = note.created_at;
//     }
//     todo!()
// }
#[inline(always)]
pub fn error_handle(error: impl std::error::Error) {
    eprintln!("{:?}", error);
}
