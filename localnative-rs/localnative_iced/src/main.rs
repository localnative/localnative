#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod data_view;
mod days;
mod helper;
mod init;
mod note;
mod page_bar;
mod search_bar;
mod setting_view;
mod style;
mod tags;

#[allow(dead_code)]
mod logger;
#[allow(dead_code)]
mod wrap;

use iced::window;
use iced::{scrollable, Application, Column, Command, Container, Element, Row, Settings, Text};

use data_view::{DataView, MiddleData};
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use localnative_core::{cmd, exe::get_sqlite_connection, rpc::server::Stop, rusqlite::Connection};
use once_cell::sync::OnceCell;
use page_bar::PageBar;
use search_bar::SearchBar;
use setting_view::{Backend, Config, SettingView};
use std::{
    path::PathBuf,
    sync::{mpsc::Sender, Arc},
};
use wrap::Wrap;

pub const BACKEND: &str = "WGPU_BACKEND";
static INSTANCE: OnceCell<Option<Arc<Vec<u8>>>> = OnceCell::new();

fn main() -> anyhow::Result<()> {
    let logo = style::icon::Icon::logo().ok()
    .and_then(|logo|window::Icon::from_rgba(logo, 64, 64).ok());
    
    LocalNative::run(Settings {
        flags: is_first(),
        antialiasing: true,
        default_font: font(),
        window: window::Settings {
            icon: logo,
            size: (1080, 720),
            ..Default::default()
        },
        ..Default::default()
    })
    .map_err(|iced_err| anyhow::anyhow!("iced err:{:?}", iced_err))
}

fn is_first() -> bool {
    let path = setting_view::app_dir().join(".env");
    let is_first = dotenv::from_path(path).is_err();
    if std::env::var(BACKEND).is_err() {
        std::env::set_var(BACKEND, &Backend::default().to_string());
    }
    is_first
}
async fn setup_logger(sender: Sender<String>) -> anyhow::Result<()> {
    let dispatch = fern::Dispatch::new().format(|out, message, record| {
        out.finish(format_args!(
            "{}[{}][{}] {}",
            chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
            record.target(),
            record.level(),
            message
        ))
    });
    let log_dir = setting_view::app_dir().join("log");
    if !log_dir.exists() {
        tokio::fs::create_dir_all(&log_dir).await?;
    }
    dispatch
        .level(log::LevelFilter::Info)
        .level_for("tracing", log::LevelFilter::Warn)
        .level_for("wgpu_core", log::LevelFilter::Error)
        .level_for("gpu_alloc", log::LevelFilter::Warn)
        .level_for("wgpu", log::LevelFilter::Warn)
        .level_for("iced_wgpu", log::LevelFilter::Warn)
        .level_for("gfx_backend_dx12", log::LevelFilter::Warn)
        .level_for("trapc", log::LevelFilter::Warn)
        .chain(sender)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_dir.join("localnative.log"))?)
        .apply()
        .map_err(|e| anyhow::anyhow!("set logger error :{:?}", e))?;
    Ok(())
}
fn font() -> Option<&'static [u8]> {
    INSTANCE.get_or_init(|| {
        SystemSource::new().select_best_match(
            &[
                FamilyName::Title("PingFang SC".to_owned()),
                FamilyName::Title("Hiragino Sans GB".to_owned()),
                FamilyName::Title("Heiti SC".to_owned()),
                FamilyName::Title("Microsoft YaHei".to_owned()),
                FamilyName::Title("WenQuanYi Micro Hei".to_owned()),
                FamilyName::Title("Microsoft YaHei".to_owned()),
                // TODO:目前不能英文字体优先使用，需要iced支持
                FamilyName::Title("Helvetica".to_owned()),
                FamilyName::Title("Tahoma".to_owned()),
                FamilyName::Title("Arial".to_owned()),
                FamilyName::SansSerif,
            ],
            &Properties::new(),
        )
        .ok()
        .and_then(|handle| {
            handle.load()
            .ok()
            .and_then(|font| font.copy_font_data())
        })
    })
    .as_ref()
    .map(|f|f.as_slice())
}

#[allow(clippy::large_enum_variant)]
enum LocalNative {
    Loading {
        logger: Option<std::sync::mpsc::Receiver<String>>,
    },
    Loaded(Data),
}

#[derive(Debug)]
pub enum Message {
    Loaded(Result<Config, setting_view::ConfigError>),
    ResultHandle(anyhow::Result<()>),
    Server(anyhow::Result<Stop>),
    NeedCreate(Config),
    PageBar(page_bar::Message),
    UnknowError,
    SearchBar(search_bar::Message),
    NoteMessage(usize, note::Message),
    SettingMessage(setting_view::Message),
    StyleMessage(style::Message),
    SyncViaFile(anyhow::Result<PathBuf>),
    NeedUpdate,
    Ignore,
    Search(String),
    BackendRes(anyhow::Result<Backend>),
    Empty(()),
}
impl Application for LocalNative {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = bool;

    fn new(is_first: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let (sender, recevier) = std::sync::mpsc::channel();
        if is_first {
            (
                Self::Loading {
                    logger: Some(recevier),
                },
                Command::batch(vec![
                    Command::perform(setting_view::Config::load(), Message::Loaded),
                    Command::perform(init::init_app_host(), Message::ResultHandle),
                    Command::perform(setup_logger(sender), Message::ResultHandle),
                ]),
            )
        } else {
            (
                Self::Loading {
                    logger: Some(recevier),
                },
                Command::batch(vec![
                    Command::perform(setting_view::Config::load(), Message::Loaded),
                    Command::perform(setup_logger(sender), Message::ResultHandle),
                ]),
            )
        }
    }

    fn title(&self) -> String {
        "Local Native".to_string()
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> iced::Command<Self::Message> {
        match self {
            LocalNative::Loading { logger } => match message {
                Message::Loaded(config) => {
                    if let Ok(mut config) = config {
                        let resource = Resource::default();
                        if !config.is_created_db {
                            cmd::create(&resource.conn);
                            config.is_created_db = true;
                        }
                        let setting_view = SettingView::init(config, helper::get_ip());
                        MiddleData::upgrade(&resource.conn);
                        let search_bar = SearchBar::default();
                        let mut page_bar = PageBar::default();

                        let middle_data = MiddleData::from_select(
                            &resource.conn,
                            search_bar.search_text.as_str(),
                            &setting_view.config.limit,
                            &page_bar.offset,
                        );
                        let mut data_view = DataView::default();
                        middle_data.encode(&mut data_view, &mut page_bar);
                        let data = if logger.is_some() {
                            let logger = logger.take().unwrap();
                            Data {
                                data_view,
                                resource,
                                setting_view,
                                search_bar,
                                page_bar,
                                logger: Some(logger::Logger::new(logger, "")),
                                ..Default::default()
                            }
                        } else {
                            Data {
                                data_view,
                                resource,
                                setting_view,
                                search_bar,
                                page_bar,
                                ..Default::default()
                            }
                        };
                        *self = LocalNative::Loaded(data);
                        Command::none()
                    } else {
                        Command::perform(Config::new(), Message::NeedCreate)
                    }
                }
                Message::NeedCreate(config) => {
                    Command::perform(setting_view::Config::save(config), Message::Loaded)
                }
                Message::ResultHandle(res) => {
                    if let Err(e) = res {
                        log::error!("fail: {:?}", e);
                    } else {
                        log::debug!("success!");
                    }
                    Command::none()
                }
                _ => unreachable!(),
            },
            LocalNative::Loaded(data) => {
                let Data {
                    data_view,
                    resource,
                    setting_view,
                    page_bar,
                    search_bar,
                    state,
                    server_state,
                    ..
                } = data;
                match message {
                    Message::Loaded(config) => {
                        if let Ok(config) = config {
                            setting_view.config = config;

                            let middle_data = MiddleData::from_select(
                                &resource.conn,
                                search_bar.search_text.as_str(),
                                &setting_view.config.limit,
                                &page_bar.offset,
                            );
                            middle_data.encode(data_view, page_bar);
                            Command::none()
                        } else {
                            Command::perform(Config::new(), Message::NeedCreate)
                        }
                    }
                    Message::SearchBar(sm) => match sm {
                        search_bar::Message::Search(text) => {
                            self.update(Message::Search(text), _clipboard);
                            Command::none()
                        }
                        search_bar::Message::Clear => {
                            search_bar.update(sm);
                            let middle_data = MiddleData::from_select(
                                &resource.conn,
                                search_bar.search_text.as_str(),
                                &setting_view.config.limit,
                                &page_bar.offset,
                            );
                            middle_data.encode(data_view, page_bar);
                            data_view.reset();
                            Command::none()
                        }
                    },

                    Message::SettingMessage(sm) => match sm {
                        setting_view::Message::Apply | setting_view::Message::ThemeChanged => {
                            setting_view.update(sm);
                            if setting_view.board_state.backend_org
                                != setting_view.board_state.backend_temp
                            {
                                log::debug!(
                                    "chage env will run 🤙,org:{},temp:{}",
                                    setting_view.board_state.backend_org.to_string(),
                                    setting_view.board_state.backend_temp.to_string()
                                );
                                Command::batch(vec![
                                    Command::perform(
                                        init::change_env(setting_view.board_state.backend_temp),
                                        Message::BackendRes,
                                    ),
                                    Command::perform(
                                        setting_view::Config::save(setting_view.config),
                                        Message::Loaded,
                                    ),
                                ])
                            } else {
                                Command::perform(
                                    setting_view::Config::save(setting_view.config),
                                    Message::Loaded,
                                )
                            }
                        }
                        setting_view::Message::Server => match state {
                            State::Settings | State::Contents => {
                                *state = State::Sync;
                                if server_state.is_none() {
                                    Command::perform(helper::start_server(), Message::Server)
                                } else {
                                    Command::none()
                                }
                            }
                            State::Sync => {
                                *state = State::Contents;
                                match server_state.take() {
                                    Some(rec) => Command::perform(
                                        helper::stop_server(rec),
                                        Message::ResultHandle,
                                    ),
                                    None => Command::none(),
                                }
                            }
                        },
                        setting_view::Message::SelectSettingBoard => {
                            match state {
                                State::Contents | State::Sync => {
                                    *state = State::Settings;
                                }
                                State::Settings => {
                                    *state = State::Contents;
                                }
                            }
                            Command::none()
                        }
                        setting_view::Message::Sync => {
                            setting_view.update(setting_view::Message::Sync);
                            if let Some(addr) = setting_view.state.socket {
                                Command::batch(vec![
                                    Command::perform(
                                        helper::client_sync_from_server(addr),
                                        Message::ResultHandle,
                                    ),
                                    Command::perform(
                                        helper::client_sync_to_server(addr),
                                        Message::ResultHandle,
                                    ),
                                ])
                            } else {
                                log::warn!("addr input error");
                                Command::none()
                            }
                        }
                        setting_view::Message::FixHost => {
                            Command::perform(init::fix_app_host(), Message::Empty)
                        }
                        setting_view::Message::BackendChanged(backend) => {
                            setting_view.update(setting_view::Message::BackendChanged(backend));
                            Command::perform(Backend::from_file(), Message::BackendRes)
                        }
                        setting_view::Message::OpenFile => {
                            Command::perform(helper::get_sync_file_path(), Message::SyncViaFile)
                        }
                        setting_view::Message::BackContent => {
                            match state {
                                State::Contents => (),
                                State::Sync | State::Settings => {
                                    *state = State::Contents;
                                }
                            }
                            Command::none()
                        }
                        sm => {
                            setting_view.update(sm);
                            Command::none()
                        }
                    },
                    Message::Search(text) => {
                        search_bar.search_text = text;
                        page_bar.offset = 0;
                        let middle_data = MiddleData::from_select(
                            &resource.conn,
                            search_bar.search_text.as_str(),
                            &setting_view.config.limit,
                            &page_bar.offset,
                        );
                        middle_data.encode(data_view, page_bar);
                        data_view.reset();
                        Command::none()
                    }

                    Message::NoteMessage(idx, nm) => {
                        if let Some(note) = data_view.notes.get_mut(idx) {
                            match nm {
                                note::Message::Delete => {
                                    cmd::delete(&resource.conn, note.rowid);
                                    let middle_data = MiddleData::from_select(
                                        &resource.conn,
                                        search_bar.search_text.as_str(),
                                        &setting_view.config.limit,
                                        &page_bar.offset,
                                    );
                                    middle_data.encode(data_view, page_bar);
                                    Command::none()
                                }
                                note::Message::Enter => {
                                    let old_note = note.old_note();
                                    log::debug!("old note:{:?}", &old_note);
                                    note.update(note::Message::Enter);
                                    let rowid = note.rowid;
                                    let mut new_note = note::Note::from(&*note);
                                    log::debug!("new note:{:?}", &new_note);
                                    if new_note != old_note {
                                        cmd::delete(&resource.conn, rowid);
                                        new_note.uuid4 = uuid::Uuid::new_v4().to_string();
                                        cmd::insert(new_note);
                                    }
                                    let middle_data = MiddleData::from_select(
                                        &resource.conn,
                                        search_bar.search_text.as_str(),
                                        &setting_view.config.limit,
                                        &page_bar.offset,
                                    );
                                    middle_data.encode(data_view, page_bar);
                                    Command::none()
                                }
                                nm => {
                                    note.update(nm);
                                    Command::none()
                                }
                            }
                        } else {
                            Command::none()
                        }
                    }

                    Message::PageBar(pm) => {
                        let cm = page_bar.update(pm, (&*setting_view).limit());
                        match cm {
                            Message::NeedUpdate => {
                                let middle_data = MiddleData::from_select(
                                    &resource.conn,
                                    search_bar.search_text.as_str(),
                                    &setting_view.config.limit,
                                    &page_bar.offset,
                                );
                                middle_data.encode(data_view, page_bar);
                                data_view.reset();
                                Command::none()
                            }
                            _ => Command::none(),
                        }
                    }
                    Message::StyleMessage(_) => Command::none(),
                    Message::ResultHandle(res) => {
                        if let Err(e) = res {
                            log::error!("fail: {:?}", e);
                        } else {
                            let middle_data = MiddleData::from_select(
                                &resource.conn,
                                search_bar.search_text.as_str(),
                                &setting_view.config.limit,
                                &page_bar.offset,
                            );
                            middle_data.encode(data_view, page_bar);
                            log::debug!("success!");
                        }
                        Command::none()
                    }
                    Message::NeedCreate(_) => Command::none(),
                    Message::UnknowError => Command::none(),
                    Message::NeedUpdate => Command::none(),
                    Message::Ignore => Command::none(),
                    Message::BackendRes(res) => match res {
                        std::result::Result::Ok(backend) => {
                            setting_view.board_state.backend_org = backend;
                            Command::none()
                        }
                        std::result::Result::Err(e) => {
                            log::error!("backend res error: {:?}", e);
                            setting_view.board_state.backend_org = Backend::default();
                            Command::perform(init::create_env(), Message::ResultHandle)
                        }
                    },
                    Message::Empty(_) => Command::none(),
                    Message::SyncViaFile(path) => {
                        Command::perform(helper::sync_via_file(path), Message::ResultHandle)
                    }
                    Message::Server(res) => {
                        match res {
                            Ok(rec) => {
                                log::info!("trigger init ok...");
                                server_state.replace(rec);
                            }
                            Err(e) => {
                                log::error!("server open error:{:?}", e);
                            }
                        }
                        Command::none()
                    }
                }
            }
        }
    }
    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        match self {
            LocalNative::Loading { .. } => Container::new(
                Text::new("Loading...")
                    .horizontal_alignment(iced::HorizontalAlignment::Center)
                    .size(50),
            )
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .center_y()
            .into(),
            LocalNative::Loaded(data) => match data.state {
                State::Contents => data.contents_view(),
                State::Settings => data.setting_view(),
                State::Sync => data.sync_view(),
            },
        }
    }
}

#[derive(Debug, Default)]
pub struct Data {
    data_view: DataView,
    resource: Resource,
    setting_view: SettingView,
    search_bar: SearchBar,
    page_bar: PageBar,
    server_state: Option<Stop>,
    state: State,
    logger: Option<logger::Logger>,
}

#[derive(Debug)]
pub enum State {
    Contents,
    Settings,
    Sync,
}

impl Default for State {
    fn default() -> Self {
        Self::Contents
    }
}

impl Data {
    fn sync_view(&mut self) -> Element<Message> {
        let Data {
            setting_view: config_view,
            logger,
            ..
        } = self;
        config_view
            .sync_board_open_view(self.server_state.is_some(), logger)
            .map(Message::SettingMessage)
    }
    fn setting_view(&mut self) -> Element<Message> {
        let Data {
            setting_view,
            logger,
            ..
        } = self;
        setting_view
            .setting_board_open_view(self.server_state.is_some(), logger)
            .map(Message::SettingMessage)
    }
    fn contents_view(&mut self) -> Element<Message> {
        let Data {
            data_view,
            setting_view,
            search_bar,
            page_bar,
            logger,
            ..
        } = self;
        let DataView { notes, tags, state } = data_view;
        let limit = setting_view.limit();
        let data_view::State {
            tags_scrollable,
            notes_scrollable,
        } = state;
        let search_text_is_empty = search_bar.search_text.is_empty();
        Row::new()
            .align_items(iced::Align::Start)
            .push(
                setting_view
                    .viwe(self.server_state.is_some(), logger)
                    .map(Message::SettingMessage),
            )
            .push(
                iced::Container::new(
                    Column::new()
                        .push(search_bar.view().map(|sm| match sm {
                            search_bar::Message::Search(text) => Message::Search(text),
                            sm => Message::SearchBar(sm),
                        }))
                        .push({
                            let notes = notes;
                            let scrollable = notes_scrollable;
                            Container::new({
                                let mut scrollable = scrollable::Scrollable::new(scrollable);
                                if notes.is_empty() {
                                    scrollable = scrollable.push({
                                        let text = if search_text_is_empty {
                                            "您还没有任何一个note，您可以通过浏览器扩展添加note。"
                                        } else {
                                            "抱歉，没找到您想要的结果..."
                                        };
                                        Container::new(Text::new(text).size(50))
                                    });
                                    scrollable.align_items(iced::Align::Center).spacing(30)
                                } else {
                                    let notes_cloumn = notes
                                        .iter_mut()
                                        .enumerate()
                                        .fold(
                                            Column::new().align_items(iced::Align::Start),
                                            |column, (idx, note)| {
                                                column.push(note.view().map(move |nm| match nm {
                                                    note::Message::TagMessage(
                                                        _,
                                                        note::tag::Message::Search(text),
                                                    ) => Message::Search(text),
                                                    nm => Message::NoteMessage(idx, nm),
                                                }))
                                            },
                                        )
                                        .padding(30);
                                    scrollable
                                        .align_items(iced::Align::Center)
                                        .spacing(30)
                                        .push(notes_cloumn)
                                        .push(page_bar.view(limit).map(Message::PageBar))
                                }
                            })
                            .center_x()
                            .center_y()
                            .height(iced::Length::Shrink)
                        }),
                )
                .width(iced::Length::FillPortion(8))
                .center_x()
                .center_y(),
            )
            .push({
                let tags = tags;
                let scrollable = tags_scrollable;
                Container::new(
                    scrollable::Scrollable::new(scrollable)
                        .scrollbar_width(1)
                        .push(
                            tags.iter_mut().fold(
                                Wrap {
                                    spacing: 10,
                                    line_spacing: 10,
                                    padding: 10,
                                    line_height: 30,
                                    ..Default::default()
                                }
                                .push(Text::new("tags:")),
                                |wrap, tag| {
                                    wrap.push(tag.view().map(|tm| match tm {
                                        tags::Message::Search(text) => Message::Search(text),
                                    }))
                                },
                            ),
                        )
                        .align_items(iced::Align::Center),
                )
                .width(iced::Length::FillPortion(2))
                .height(iced::Length::Fill)
            })
            .into()
    }
}

#[derive(Debug)]
pub struct Resource {
    conn: Connection,
}
impl Default for Resource {
    fn default() -> Self {
        Self {
            conn: get_sqlite_connection(),
        }
    }
}
pub enum DbError {
    CreateError,
}
