use std::str::FromStr;

use chrono::{Local, NaiveDateTime, TimeZone};
use connect::{get_current_page_info, CmdResult, PageInfo};

use locales::tr;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use xilem_web::{
    concurrent::{async_repeat, memoized_await},
    core::{
        fork,
        one_of::{OneOf, OneOf2},
    },
    document_body,
    elements::html::{a, button, div, h1, h2, i, input, label, option, select, span, textarea},
    event_handler::defer,
    interfaces::{
        Element, HtmlButtonElement, HtmlDivElement, HtmlElement, HtmlInputElement,
        HtmlOptionElement, HtmlSpanElement, HtmlTextAreaElement,
    },
    App, IntoAttributeValue,
};
mod connect;
mod locales;

pub struct AppState {
    public_checked: bool,
    title: String,
    url: String,
    desc: String,
    tags: String,
    count: usize,
    annotations: String,
    notes: Vec<Note>,
    seach_info: SearchInfo,
    is_dark_theme: bool,
    language: String,
}

impl AppState {
    pub fn update_info(&mut self, page_info: Option<PageInfo>) {
        if let Some(info) = page_info {
            self.title = info.title;
            self.url = info.url;
        }
    }
    pub fn update_resp(&mut self, resp: connect::Response) {
        self.count = resp.count;
        self.notes = resp.notes();
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SearchInfo {
    search_text: String,
    offset: usize,
    limit: usize,
}

#[derive(Debug)]
pub struct Note {
    created_at: String,
    uuid4: String,
    rowid: usize,
    title: String,
    url: String,
    description: String,
    tags: Vec<String>,
    deleting: bool,
    is_expand: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerdeNote {
    created_at: String,
    uuid4: String,
    rowid: usize,
    title: String,
    url: String,
    description: String,
    tags: String,
}

impl SerdeNote {
    pub fn into_note(self) -> Note {
        let Self {
            created_at,
            uuid4,
            rowid,
            title,
            url,
            description,
            tags,
        } = self;
        Note {
            created_at,
            uuid4,
            rowid,
            title,
            url,
            description,
            tags: tags
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect(),
            deleting: false,
            is_expand: false,
        }
    }
}

const DESCRIPTION_TRUNCATION_LENGTH: usize = 150;
const DESCRIPTION_MAX_LINES: usize = 3;

fn note_ui(state: &mut AppState) -> impl HtmlDivElement<AppState> {
    div(state
        .notes
        .iter()
        .enumerate()
        .map(|(idx, note)| note_item(&state.language, idx, note))
        .collect::<Vec<_>>())
    .class("notes-container")
}

fn note_item(language: &str, idx: usize, note: &Note) -> impl HtmlDivElement<AppState> {
    let is_truncated = note.description.chars().count() > DESCRIPTION_TRUNCATION_LENGTH;
    let is_truncated_lines = note.description.lines().count() > DESCRIPTION_MAX_LINES;

    div((
        note_header(idx, note),
        h2(note.title.clone())
            .class("note-title")
            .attr("title", note.title.clone()),
        a(note.url.clone())
            .attr("href", note.url.clone())
            .attr("target", "_blank")
            .class("note-url"),
        note_description(language, idx, note, is_truncated, is_truncated_lines),
        note_tags(idx, note),
    ))
    .class("note")
}

fn note_header(idx: usize, note: &Note) -> impl HtmlDivElement<AppState> {
    div((
        span(convert_utc_to_local(&note.created_at)).class("note-date"),
        delete_button(idx, note),
    ))
    .class("note-header")
}

fn delete_button(idx: usize, note: &Note) -> impl HtmlButtonElement<AppState> {
    button(if note.deleting {
        i("").class("fas fa-spinner fa-spin")
    } else {
        i("").class("fas fa-trash")
    })
    .class("delete-btn")
    .disabled(note.deleting)
    .on_click(defer(
        move |state: &mut AppState, _| {
            state.notes[idx].deleting = true;
            let note_rowid = state.notes[idx].rowid;
            let SearchInfo {
                search_text,
                offset,
                limit,
            } = state.seach_info.clone();
            connect::Message::delete(search_text, limit, offset, note_rowid as i64)
        },
        handle_response,
    ))
}

fn note_description(
    language: &str,
    idx: usize,
    note: &Note,
    is_truncated: bool,
    is_truncated_lines: bool,
) -> impl HtmlDivElement<AppState> {
    div((
        span(get_truncated_description(
            note,
            is_truncated_lines,
            is_truncated,
        )),
        expand_text(language, idx, note, is_truncated_lines || is_truncated),
    ))
    .class("note-desc")
}

fn get_truncated_description(note: &Note, is_truncated_lines: bool, is_truncated: bool) -> String {
    match (note.is_expand, is_truncated_lines, is_truncated) {
        (false, true, _) => {
            note.description
                .lines()
                .take(DESCRIPTION_MAX_LINES)
                .collect::<Vec<_>>()
                .join("\n")
                + "..."
        }
        (false, false, true) => {
            note.description
                .chars()
                .take(DESCRIPTION_TRUNCATION_LENGTH)
                .collect::<String>()
                + "..."
        }
        _ => note.description.clone(),
    }
}

fn expand_text(
    language: &str,
    idx: usize,
    note: &Note,
    is_truncated: bool,
) -> Option<impl HtmlSpanElement<AppState>> {
    if is_truncated {
        Some(
            span(if note.is_expand {
                tr::collapse(language).to_string()
            } else {
                tr::expand(language).to_string()
            })
            .class("expand-text")
            .on_click(move |state: &mut AppState, _| {
                state.notes[idx].is_expand = !state.notes[idx].is_expand;
            }),
        )
    } else {
        None
    }
}

fn note_tags(idx: usize, note: &Note) -> impl HtmlDivElement<AppState> {
    div(note
        .tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            button(tag.clone())
                .class("tag-btn")
                .on_click(move |state: &mut AppState, _| {
                    state.seach_info.search_text = state.notes[idx].tags[i].clone();
                })
        })
        .collect::<Vec<_>>())
    .class("note-tags")
}
fn active_view(state: &mut AppState) -> impl HtmlElement<AppState> {
    let div = div((
        header_section(state),
        input_section(state),
        search_section(state),
        note_ui(state),
        response_section(),
    ))
    .class("app-container");
    if state.is_dark_theme {
        OneOf2::A(div.class("dark-theme"))
    } else {
        OneOf::B(div)
    }
}

fn header_section(state: &mut AppState) -> impl HtmlDivElement<AppState> {
    div((
        h1(tr::app_title(&state.language).to_string()).class("app-title"),
        div((
            checkbox("cb-public", "public", state.public_checked, |state| {
                state.public_checked = !state.public_checked;
                set_local_storage_parsed("public_checked", state.public_checked);
            })
            .class("toggle-input"),
            label(tr::toggle_label(&state.language).to_string())
                .attr("for", "cb-public")
                .attr("id", "label-public")
                .class("toggle-label"),
        ))
        .class("toggle-container"),
        div((
            label(tr::select_label(&state.language).to_string())
                .attr("id", "label-language")
                .class("select-label"),
            select((
                option(tr::option_en_us(&state.language).to_string())
                    .selected(state.language == "en")
                    .attr("value", "en"),
                option(tr::option_zh_cn(&state.language).to_string())
                    .selected(state.language == "zh")
                    .attr("value", "zh"),
            ))
            .attr("id", "select-language")
            .class("language-select")
            .on_change(|state: &mut AppState, event: web_sys::Event| {
                if let Some(element) = event
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                {
                    state.language = element.value();
                    set_local_storage_parsed("language", state.language.clone());
                }
            }),
        ))
        .class("language-container"),
        div(()).class("theme-transition"),
        button(i(()).class("fas").class(if state.is_dark_theme {
            "fa-sun"
        } else {
            "fa-moon"
        }))
        .class("theme-toggle")
        .attr("id", "theme-toggle")
        .on_click(|state: &mut AppState, _| {
            state.is_dark_theme = !state.is_dark_theme;
            set_local_storage_parsed("is_dark_theme", state.is_dark_theme);
        }),
    ))
    .class("header-section")
}

fn input_section(state: &mut AppState) -> impl HtmlDivElement<AppState> {
    div((
        div((
            text_input(
                "title",
                tr::title(&state.language).to_string(),
                state.title.clone(),
                |state, value| state.title = value,
            ),
            text_input(
                "url",
                tr::url(&state.language).to_string(),
                state.url.clone(),
                |state, value| state.url = value,
            ),
        ))
        .class("input-row"),
        textarea_input(
            "desc-text",
            tr::description(&state.language).to_string(),
            state.desc.clone(),
            |state, value| state.desc = value,
        ),
        text_input(
            "tags-text",
            tr::tags(&state.language).to_string(),
            state.tags.clone(),
            |state, value| state.tags = value,
        ),
        button(tr::insert_btn(&state.language).to_string())
            .class("insert-btn")
            .on_click(defer(
                move |state: &mut AppState, _| {
                    let tags = state
                        .tags
                        .split(|c: char| c.is_whitespace() || c == ',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(",");
                    connect::Message::insert(
                        state.title.clone(),
                        state.url.clone(),
                        tags,
                        state.desc.clone(),
                        String::new(), // comments
                        String::new(), // annotations
                        state.public_checked,
                        state.seach_info.limit,
                        state.seach_info.offset,
                    )
                },
                handle_response,
            )),
    ))
    .class("input-section")
}

fn response_section() -> impl HtmlDivElement<AppState> {
    div((
        div(()).attr("id", "request-text"),
        div(()).attr("id", "response-text"),
    ))
    .class("response-section")
}

fn search_section(state: &mut AppState) -> impl HtmlDivElement<AppState> {
    div((
        div((
            text_input(
                "search-text",
                tr::search(&state.language).to_string(),
                state.seach_info.search_text.clone(),
                |state, value| state.seach_info.search_text = value,
            ),
            button(i("").class("fas fa-times"))
                .class("clear-btn")
                .attr("title", tr::clear_search(&state.language).to_string())
                .on_click(|state: &mut AppState, _| {
                    state.seach_info.search_text = String::new();
                }),
        ))
        .class("search-input-container"),
        div((
            button(i("").class("fas fa-chevron-left"))
                .class("nav-btn")
                .attr("title", tr::previous_page(&state.language).to_string())
                .on_click(|state: &mut AppState, _| {
                    if state.seach_info.offset >= state.seach_info.limit {
                        state.seach_info.offset -= state.seach_info.limit;
                    }
                }),
            button(i("").class("fas fa-chevron-right"))
                .class("nav-btn")
                .attr("title", tr::next_page(&state.language).to_string())
                .on_click(|state: &mut AppState, _| {
                    if state.seach_info.offset + state.seach_info.limit < state.count {
                        state.seach_info.offset += state.seach_info.limit;
                    }
                }),
            span({
                let start = state.seach_info.offset + 1;
                let end = (state.seach_info.offset + state.seach_info.limit).min(state.count);
                if start == end {
                    format!("{}/{}", start, state.count)
                } else {
                    format!("{}-{}/{}", start, end, state.count)
                }
            })
            .class("page-indicator"),
        ))
        .class("search-nav-container"),
    ))
    .class("search-section")
}

fn checkbox(
    id: impl IntoAttributeValue,
    name: impl IntoAttributeValue + Clone,
    checked: bool,
    on_change: impl Fn(&mut AppState) + 'static,
) -> impl HtmlInputElement<AppState> {
    input(())
        .attr("type", "checkbox")
        .attr("id", id)
        .attr("name", name)
        .attr("checked", checked)
        .on_click(move |state: &mut AppState, event| {
            log::debug!("checkbox event: {:?}", event);
            on_change(state);
        })
}

fn text_input(
    id: impl IntoAttributeValue,
    placeholder: impl IntoAttributeValue,
    value: impl IntoAttributeValue,
    on_input: impl Fn(&mut AppState, String) + 'static,
) -> impl HtmlInputElement<AppState> {
    input(())
        .attr("type", "text")
        .attr("id", id)
        .attr("placeholder", placeholder)
        .attr("value", value)
        .class("text-input")
        .on_input(move |state: &mut AppState, event: web_sys::Event| {
            if let Some(element) = event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                on_input(state, element.value());
            }
        })
}

fn textarea_input(
    id: impl IntoAttributeValue,
    placeholder: impl IntoAttributeValue,
    value: impl IntoAttributeValue,
    on_input: impl Fn(&mut AppState, String) + 'static,
) -> impl HtmlTextAreaElement<AppState> {
    textarea(())
        .attr("id", id)
        .attr("placeholder", placeholder)
        .attr("value", value)
        .class("textarea-input")
        .on_input(move |state: &mut AppState, event: web_sys::Event| {
            if let Some(element) = event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
            {
                on_input(state, element.value());
            }
        })
}

fn handle_response(state: &mut AppState, recv: Result<CmdResult, wasm_bindgen::JsValue>) {
    match recv {
        Ok(CmdResult::QueryResult(resp)) => {
            state.count = resp.count;
            state.notes = resp.notes();
            state.seach_info.offset = state.seach_info.offset.min(state.count.saturating_sub(1));
        }
        Ok(CmdResult::Error {
            message,
            source_text,
        }) => {
            log::error!(
                "Error: {}, Source: {}, Search Text: {}",
                message,
                source_text,
                state.seach_info.search_text
            );
        }
        Err(e) => {
            log::error!("Failed to receive command result: {:?}", e);
        }
    }
}

fn app_logic(state: &mut AppState) -> impl HtmlElement<AppState> {
    let active_view = active_view(state);

    let view = fork(
        active_view,
        memoized_await(
            state.seach_info.clone(),
            |info| {
                let SearchInfo {
                    search_text,
                    offset,
                    limit,
                } = info.clone();

                connect::Message::search(search_text, limit, offset)
            },
            handle_response,
        ),
    );
    fork(
        view,
        async_repeat(
            |proxy, _shutdown| async move {
                let info = get_current_page_info().await;
                proxy.send_message(Init(info));
            },
            |state: &mut AppState, info: Init| {
                state.update_info(info.0);
            },
        ),
    )
}

#[derive(Debug)]
struct Init(Option<PageInfo>);

fn get_local_storage(key: &str) -> Option<String> {
    let window = web_sys::window().expect("no global `window` exists");
    let storage = window
        .local_storage()
        .expect("failed to get local storage")?;
    storage.get(key).ok().flatten()
}

fn set_local_storage(key: &str, value: String) {
    let window = web_sys::window().expect("no global `window` exists");
    let storage = window
        .local_storage()
        .expect("failed to get local storage")
        .unwrap();
    storage.set(key, &value).unwrap();
}

fn get_local_storage_parsed<T: std::str::FromStr>(key: &str, default: T) -> T {
    get_local_storage(key)
        .and_then(|value| value.parse::<T>().ok())
        .unwrap_or(default)
}

fn set_local_storage_parsed<T: std::string::ToString>(key: &str, value: T) {
    set_local_storage(key, value.to_string());
}

pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    let is_dark_theme = get_local_storage_parsed("is_dark_theme", false);
    let public_checked = get_local_storage_parsed("public_checked", true);
    let language = get_local_storage_parsed("language", "en".to_string());

    let state = AppState {
        public_checked,
        title: String::new(),
        url: String::new(),
        desc: String::new(),
        tags: String::new(),
        annotations: String::new(),
        seach_info: SearchInfo {
            search_text: String::new(),
            offset: 0,
            limit: 10,
        },
        notes: Vec::new(),
        count: 0,
        is_dark_theme,
        language,
    };

    App::new(document_body(), state, app_logic).run();
}

fn convert_utc_to_local(utc_time_str: &str) -> String {
    let utc = NaiveDateTime::from_str(utc_time_str).expect("msg");
    Local.from_utc_datetime(&utc).to_rfc3339()
}
