mod chart;
mod config;
mod days;
mod db_operations;
mod delete_tip;
mod icons;
mod init;
mod note;
mod search_page;
mod settings;
mod sidebar;
mod style;
mod sync;
mod tags;
mod translate;

use std::cmp::Ordering;

use chart::ChartView;
#[cfg(feature = "preview")]
pub use chart::NewChart;
use config::Config;
pub use days::DateView;
use delete_tip::DeleteTip;
use iced::{event, window, Font, Size};
use iced::{widget::container, Command};
use iced::{
    widget::{column, horizontal_space, row, text, vertical_space},
    Theme,
};
use localnative_core::db::{init_db, models::QueryResult, DbError};
pub use note::NoteView;
pub use search_page::SearchPage;
use sidebar::Sidebar;
use sqlx::SqlitePool;
pub use tags::TagView;
use tokio_util::sync::CancellationToken;

use crate::sync::SyncView;

pub struct LocalNative {
    config: Config,
    state: State,
}

pub enum State {
    Loading(String),
    Loaded(Data),
}

pub struct Data {
    search_page: SearchPage,
    sidebar: Sidebar,
    delete_tip: DeleteTip,
    settings: settings::Settings,
    sync_view: SyncView,
    pool: SqlitePool,
}

impl Data {
    fn new(config: &Config, pool: SqlitePool) -> Self {
        Self {
            search_page: SearchPage::default_with_theme(config.theme()),
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
            pool,
        }
    }

    fn handle_receiver_message(&mut self, res: QueryResult, config: &Config) -> Command<Message> {
        let QueryResult {
            count,
            notes,
            days,
            tags,
        } = res;
        self.search_page.count = count;
        if self.search_page.offset > count && notes.is_empty() {
            self.search_page.offset = count.max(config.limit) - config.limit;
            return search_page::search(
                &self.pool,
                self.search_page.search_value.clone(),
                config.limit,
                self.search_page.offset,
                self.search_page.range,
            );
        }
        Command::batch([
            Command::perform(
                async move {
                    let mut tags = tags;
                    tags.sort_by(|a, b| {
                        let ord = b.count.cmp(&a.count);
                        if ord == Ordering::Equal {
                            b.tag.cmp(&a.tag)
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
            { Command::perform(async move { ChartView::from_days(days) }, Message::DayView) },
        ])
    }

    fn handle_search_page_message(
        &mut self,
        spmsg: search_page::Message,
        config: &Config,
    ) -> Command<Message> {
        let search_page = &mut self.search_page;
        let delete_tip = &mut self.delete_tip;
        search_page.update(spmsg, config.limit, &self.pool, true, delete_tip)
    }

    fn handle_note_view_message(&mut self, notes: Vec<NoteView>) -> Command<Message> {
        self.search_page.notes = notes;
        Command::none()
    }

    fn handle_tag_view_message(&mut self, tags: Vec<TagView>) -> Command<Message> {
        self.search_page.tags = tags;
        Command::none()
    }

    fn handle_day_view_message(&mut self, chart: ChartView) -> Command<Message> {
        self.search_page.days.chart.view = chart;
        Command::none()
    }

    fn handle_request_closed_message(&mut self, config: &mut Config) -> Command<Message> {
        config.date_filter_is_show = self.search_page.days.is_show;
        let json = serde_json::to_string_pretty(&config).unwrap();
        Command::perform(config::save(json), Message::CloseWindow)
    }

    fn handle_close_window_message(&self, res: Option<()>) -> Command<Message> {
        if res.is_some() {
            println!("ok!");
        }
        window::close(window::Id::MAIN)
    }

    fn handle_sidebar_message(
        &mut self,
        smsg: sidebar::Message,
        config: &mut Config,
    ) -> Command<Message> {
        let sidebar = &mut self.sidebar;
        let settings = &mut self.settings;
        if matches!(smsg, sidebar::Message::ThemeChanged) {
            self.search_page.days.chart.style = config.theme();
        }
        sidebar.update(smsg, settings, config)
    }

    fn handle_delete_tip_message(
        &mut self,
        msg: delete_tip::Message,
        config: &Config,
    ) -> Command<Message> {
        let search_page = &mut self.search_page;
        let delete_tip = &mut self.delete_tip;
        let pool = &self.pool;
        match msg {
            delete_tip::Message::Enter => {
                delete_tip.show_modal = false;
                Command::perform(
                    db_operations::delete(
                        pool.clone(),
                        search_page.search_value.to_string(),
                        config.limit,
                        search_page.offset,
                        delete_tip.rowid,
                    ),
                    Message::Receiver,
                )
            }
            delete_tip::Message::SearchPage(spmsg) => search_page.update(
                spmsg,
                config.limit,
                pool,
                config.disable_delete_tip,
                delete_tip,
            ),
            delete_tip::Message::Cancel => {
                delete_tip.show_modal = false;
                Command::none()
            }
        }
    }

    fn handle_sync_client_message(&mut self, sync_msg: sync::Message) -> Command<Message> {
        self.sync_view.update(sync_msg, self.pool.clone())
    }

    fn handle_sync_result_message(
        &mut self,
        res: anyhow::Result<()>,
        config: &Config,
    ) -> Command<Message> {
        if let Err(err) = res {
            if let Some(io_error) = err.downcast_ref::<std::io::Error>() {
                self.sync_view.with_sync_state_mut(|state| {
                    *state = sync::SyncState::SyncError(io_error.to_string())
                });
            }
            Command::none()
        } else {
            self.sync_view
                .with_sync_state_mut(|state| *state = sync::SyncState::Complete);

            search_page::search(
                &self.pool,
                self.search_page.search_value.clone(),
                config.limit,
                self.search_page.offset,
                self.search_page.range,
            )
        }
    }

    fn handle_sync_option_message(&mut self, opt: Option<()>, config: &Config) -> Command<Message> {
        if opt.is_none() {
            self.sync_view
                .with_sync_state_mut(|state| *state = sync::SyncState::SyncFromFileUnknownError);

            Command::none()
        } else {
            self.sync_view
                .with_sync_state_mut(|state| *state = sync::SyncState::Complete);
            search_page::search(
                &self.pool,
                self.search_page.search_value.clone(),
                config.limit,
                self.search_page.offset,
                self.search_page.range,
            )
        }
    }

    fn handle_start_server_result_message(
        &mut self,
        res: anyhow::Result<CancellationToken>,
    ) -> Command<Message> {
        match res {
            Ok(stop) => {
                let is_none = sync::get_ip()
                    .and_then(|ip| {
                        let addr = ip + ":" + self.sync_view.borrow_port().to_string().as_str();
                        let state = iced::widget::qr_code::Data::new(&addr).ok();
                        state.map(|state| (addr, state))
                    })
                    .map(|(addr, state)| {
                        self.sync_view
                            .with_server_state_mut(|state| *state = sync::ServerState::Opened);
                        self.sync_view.update_server_addr(addr);
                        self.sync_view
                            .with_ip_qr_code_mut(|qr_code| *qr_code = state);
                    })
                    .is_none();
                if is_none {
                    self.sync_view
                        .with_server_state_mut(|state| *state = sync::ServerState::Closed);
                    if let Some(cmd) = self.sync_view.with_stop_mut(|ref_mut_stop| {
                        if let Some(old_stop) = ref_mut_stop.take() {
                            Some(Command::batch([
                                Command::perform(
                                    sync::stop_server(old_stop),
                                    Message::ServerOption,
                                ),
                                Command::perform(sync::stop_server(stop), Message::ServerOption),
                            ]))
                        } else {
                            None
                        }
                    }) {
                        return cmd;
                    }
                } else {
                    if self.sync_view.borrow_stop().is_some() {
                        if let Some(cmd) = self.sync_view.with_stop_mut(|ref_mut_stop| {
                            if let Some(old_stop) = ref_mut_stop.take() {
                                ref_mut_stop.replace(stop);
                                Some(Command::perform(
                                    sync::stop_server(old_stop),
                                    Message::ServerOption,
                                ))
                            } else {
                                None
                            }
                        }) {
                            return cmd;
                        }
                    } else {
                        self.sync_view.with_stop_mut(|ref_mut_stop| {
                            ref_mut_stop.replace(stop);
                        });
                    }
                }
            }
            Err(err) => {
                self.sync_view.with_sync_state_mut(|state| {
                    *state = sync::SyncState::SyncError(err.to_string())
                });
            }
        }
        Command::none()
    }

    fn handle_server_option_message(&mut self, opt: Option<()>) -> Command<Message> {
        if opt.is_some() {
            self.sync_view
                .with_server_state_mut(|state| *state = sync::ServerState::Closed);
        } else {
            self.sync_view
                .with_server_state_mut(|state| *state = sync::ServerState::Error);
        }
        Command::none()
    }

    fn handle_settings_message(
        &mut self,
        msg: settings::Message,
        config: &mut Config,
    ) -> Command<Message> {
        let settings = &mut self.settings;
        let sidebar = &mut self.sidebar;
        settings.update(msg, config, sidebar)
    }

    fn handle_load_font_message(&self, res: Result<(), iced::font::Error>) -> Command<Message> {
        match res {
            Ok(_) => println!("Font loaded successfully!"),
            Err(e) => eprintln!("Failed to load font: {:?}", e),
        }
        Command::none()
    }

    fn handle_loaded_state(&mut self, config: &mut Config, message: Message) -> Command<Message> {
        match message {
            Message::Receiver(Some(md)) => self.handle_receiver_message(md, &*config),
            Message::SearchPageMessage(spmsg) => self.handle_search_page_message(spmsg, &*config),
            Message::NoteView(notes) => self.handle_note_view_message(notes),
            Message::TagView(tags) => self.handle_tag_view_message(tags),
            Message::Loading(..) => Command::none(),
            Message::DayView(chart) => self.handle_day_view_message(chart),
            Message::ApplyLanguage(..) => Command::none(),
            Message::RequestClosed => self.handle_request_closed_message(config),
            Message::CloseWindow(res) => self.handle_close_window_message(res),
            Message::SidebarMessage(smsg) => self.handle_sidebar_message(smsg, config),
            Message::DeleteTipMessage(msg) => self.handle_delete_tip_message(msg, &*config),
            Message::SyncClientMessage(sync_msg) => self.handle_sync_client_message(sync_msg),
            Message::SyncResult(res) => self.handle_sync_result_message(res, &*config),
            Message::SyncOption(opt) => self.handle_sync_option_message(opt, &*config),
            Message::StartServerResult(res) => self.handle_start_server_result_message(res),
            Message::ServerOption(opt) => self.handle_server_option_message(opt),
            Message::SettingsMessage(msg) => self.handle_settings_message(msg, config),
            Message::InitHost(..) => Command::none(),
            Message::Receiver(None) => Command::none(),
            Message::LoadFont(res) => self.handle_load_font_message(res),
            Message::InitDatabase(_) => Command::none(),
        }
    }
}

#[derive(Debug)]
pub enum Message {
    Loading(SqlitePool),
    InitDatabase(Result<SqlitePool, DbError>),
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
    StartServerResult(anyhow::Result<CancellationToken>),
    ServerOption(Option<()>),
    InitHost(()),
    Receiver(Option<QueryResult>),
    LoadFont(Result<(), iced::font::Error>),
}

impl iced::Application for LocalNative {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = Option<Config>;
    type Theme = Theme;

    fn new(flags: Option<Config>) -> (Self, Command<Self::Message>) {
        let is_first_open = flags.is_none();
        let config = flags.unwrap_or_default();
        let language = config.language;

        (
            LocalNative {
                config,
                state: State::Loading(String::new()),
            },
            Command::batch([
                iced::font::load(include_bytes!("../fonts/icons.ttf")).map(Message::LoadFont),
                Command::perform(init_db(), Message::InitDatabase),
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
        match &mut self.state {
            State::Loading(..) => self.handle_loading_state(message),
            State::Loaded(data) => data.handle_loaded_state(&mut self.config, message),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        match &self.state {
            State::Loading(info) => self.loading_view(info),
            State::Loaded(data) => self.loaded_view(data),
        }
    }

    fn theme(&self) -> Self::Theme {
        self.config.theme().into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        event::listen_raw(|event, status| match (event, status) {
            (iced::Event::Window(_, window::Event::CloseRequested), event::Status::Ignored) => {
                Some(Message::RequestClosed)
            }
            _ => None,
        })
    }
}

impl LocalNative {
    fn handle_loading_state(&mut self, message: Message) -> Command<Message> {
        let State::Loading(ref mut info) = self.state else {
            return Command::none();
        };
        match message {
            Message::InitDatabase(Ok(pool)) => {
                Command::perform(async {}, move |_| Message::Loading(pool.clone()))
            }
            Message::InitDatabase(Err(err)) => {
                *info = err.to_string();
                Command::none()
            }
            Message::Loading(pool) => {
                let data = Data::new(&self.config, pool);
                self.state = State::Loaded(data);
                if let State::Loaded(ref mut data) = self.state {
                    let search_page = &data.search_page;
                    let pool = data.pool.clone();
                    Command::perform(
                        db_operations::upgrade(
                            pool,
                            search_page.search_value.clone(),
                            self.config.limit,
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
        }
    }

    fn loading_view(&self, info: &String) -> iced::Element<'_, Message> {
        column![
            vertical_space(),
            row![
                horizontal_space(),
                if info.is_empty() {
                    text("Loading...")
                } else {
                    text(info)
                }
                .size(50),
                horizontal_space()
            ],
            vertical_space(),
        ]
        .into()
    }

    fn loaded_view<'view, 'data: 'view>(
        &'view self,
        data: &'data Data,
    ) -> iced::Element<'view, Message> {
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
                if self.config.disable_delete_tip {
                    search_page
                        .view(self.config.limit)
                        .map(Message::SearchPageMessage)
                } else {
                    delete_tip
                        .view(self.config.limit, search_page)
                        .map(Message::DeleteTipMessage)
                }
            }
            sidebar::State::SyncView => sync_client.view().map(Message::SyncClientMessage),
        };
        if sidebar.settings_is_open {
            page = settings
                .view(page.map(|_| crate::settings::Message::Other), &self.config)
                .map(Message::SettingsMessage);
        }
        row![
            container(
                sidebar
                    .view(&self.config.theme_kind)
                    .map(Message::SidebarMessage)
            )
            .width(iced::Length::Shrink),
            container(page)
        ]
        .into()
    }
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
        flags: Config::load(),
        window: iced::window::Settings {
            size: Size::new(1080., 720.),
            icon: logo(),
            exit_on_close_request: false,
            ..Default::default()
        },
        default_font: if cfg!(target_os = "windows") {
            Font::with_name("Microsoft YaHei") // Common system font on Windows that supports Chinese
        } else if cfg!(target_os = "macos") {
            Font::with_name("PingFang SC") // Common system font on macOS that supports Chinese
        } else if cfg!(target_os = "linux") {
            Font::with_name("Noto Sans CJK SC") // Common open-source font on Linux that supports Chinese
        } else {
            Font::with_name("Arial Unicode MS") // Fallback to a widely supported font that supports Chinese
        },
        ..Default::default()
    }
}

pub fn none_flags_settings() -> iced::Settings<()> {
    iced::Settings::default()
}

#[inline(always)]
pub fn error_handle(error: impl std::error::Error) {
    eprintln!("{:?}", error);
}
