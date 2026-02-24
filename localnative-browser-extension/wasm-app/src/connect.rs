use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_extensions_sys::chrome;

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    notes: Vec<crate::SerdeNote>,
    pub count: usize,
}

impl Response {
    pub fn notes(self) -> Vec<crate::Note> {
        self.notes
            .into_iter()
            .map(crate::SerdeNote::into_note)
            .collect()
    }
}

#[derive(Serialize, Debug, Default)]
pub struct Note {
    title: String,
    url: String,
    tags: String,
    description: String,
    comments: String,
    annotations: String,
    is_public: bool,
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Insert,
    #[default]
    Search,
    Delete,
}

#[derive(Serialize, Debug, Default)]
pub struct Message {
    action: Action,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    note: Option<Note>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rowid: Option<i64>,
}

impl Message {
    pub async fn delete(
        query: impl Into<String>,
        limit: usize,
        offset: usize,
        rowid: i64,
    ) -> Result<CmdResult, JsValue> {
        let msg = Self {
            action: Action::Delete,
            rowid: Some(rowid),
            query: Some(query.into()),
            limit: Some(limit),
            offset: Some(offset),
            ..Default::default()
        };

        cmd(&msg).await
    }

    pub async fn search(
        query: impl Into<String>,
        limit: usize,
        offset: usize,
    ) -> Result<CmdResult, JsValue> {
        let msg = Self {
            action: Action::Search,
            query: Some(query.into()),
            limit: Some(limit),
            offset: Some(offset),
            ..Default::default()
        };
        cmd(&msg).await
    }

    pub async fn insert(
        title: impl Into<String>,
        url: impl Into<String>,
        tags: impl Into<String>,
        description: impl Into<String>,
        comments: impl Into<String>,
        annotations: impl Into<String>,
        is_public: bool,
        limit: usize,
        offset: usize,
    ) -> Result<CmdResult, JsValue> {
        let msg = Self {
            action: Action::Insert,
            note: Some(Note {
                title: title.into(),
                url: url.into(),
                tags: tags.into(),
                description: description.into(),
                comments: comments.into(),
                annotations: annotations.into(),
                is_public,
            }),
            limit: Some(limit),
            offset: Some(offset),
            ..Default::default()
        };
        cmd(&msg).await
    }
}

#[derive(Debug)]
pub struct PageInfo {
    pub title: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum CmdResult {
    QueryResult(Response),
    Error {
        message: String,
        source_text: String,
    },
}

#[inline]
async fn cmd(message: &Message) -> Result<CmdResult, JsValue> {
    let js_message = serde_wasm_bindgen::to_value(message)?;
    let js_object = js_sys::Object::from_entries(&js_message).unwrap();

    let res = chrome()
        .runtime()
        .send_native_message("app.localnative", &js_object)
        .await?;

    serde_wasm_bindgen::from_value::<CmdResult>(res)
        .map_err(|e| JsValue::from_str(&format!("Deserialization error: {:?}", e)))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryInfo {
    active: bool,
    last_focused_window: bool,
}

pub async fn get_current_page_info() -> Option<PageInfo> {
    let query_info = QueryInfo {
        active: true,
        last_focused_window: true,
    };

    // 使用 serde_wasm_bindgen 将 QueryInfo 转换为 JsValue
    let query_info_js: JsValue = serde_wasm_bindgen::to_value(&query_info)
        .map_err(|e| {
            log::error!("Failed to serialize query info: {:?}", e);
            e
        })
        .ok()?;

    let tabs_result = chrome()
        .tabs()
        .query(query_info_js.unchecked_ref())
        .await
        .ok()?;

    let tabs = serde_wasm_bindgen::from_value::<Vec<Tab>>(tabs_result)
        .map_err(|e| {
            log::error!("Failed to deserialize tabs: {:?}", e);
            e
        })
        .ok()?;

    tabs.into_iter().next().map(|tab| PageInfo {
        title: tab.title.unwrap_or_else(|| "title".into()),
        url: tab.url.unwrap_or_else(|| "url".into()),
    })
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub title: Option<String>,
    pub url: Option<String>,
}
