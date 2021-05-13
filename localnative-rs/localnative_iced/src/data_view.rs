use std::sync::Arc;

use crate::{days::Day, page_bar::PageBar, tags::TagView};
use crate::{
    note::{Note, NoteView},
    tags::Tag,
};

use iced::{button, futures::lock::Mutex, scrollable};
use localnative_core::cmd::{create, delete, insert};
use localnative_core::exe::do_search;
use localnative_core::rusqlite::Connection;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct MiddleData {
    count: u32,
    notes: Vec<Note>,
    days: Vec<Day>,
    tags: Vec<Tag>,
}

pub fn encode(
    data_view: &mut DataView,
    page_info: &mut PageBar,
    mdata: (Vec<NoteView>, Vec<TagView>, u32),
) {
    let (notes, tags, count) = mdata;
    data_view.notes = notes;
    data_view.tags = tags;
    // TODO:
    // data_view.days = self.days;
    page_info.count = count;
}
impl MiddleData {
    pub async fn delete(
        conn: Arc<Mutex<Connection>>,
        query: String,
        limit: u32,
        offset: u32,
        rowid: i64,
    ) -> anyhow::Result<(Vec<NoteView>, Vec<TagView>, u32)> {
        let conn = &*conn.lock().await;
        delete(conn, rowid);
        Self::from_select_inner(conn, query, limit, offset)
    }
    pub async fn upgrade(
        conn: Arc<Mutex<Connection>>,
        query: String,
        limit: u32,
        offset: u32,
        is_created_db: bool,
    ) -> anyhow::Result<(Vec<NoteView>, Vec<TagView>, u32)> {
        let conn = &*conn.lock().await;
        if !is_created_db {
            create(conn);
        }
        if let Ok(version) = localnative_core::upgrade::upgrade(conn) {
            log::debug!("upgrade done:{}", version);
        } else {
            log::error!("upgrade error");
        }
        Self::from_select_inner(conn, query, limit, offset)
    }
    pub async fn insert(
        conn: Arc<Mutex<Connection>>,
        query: String,
        limit: u32,
        offset: u32,
        rowid: i64,
        note: Note,
    ) -> anyhow::Result<(Vec<NoteView>, Vec<TagView>, u32)> {
        let conn = &*conn.lock().await;
        delete(conn, rowid);
        insert(note);
        Self::from_select_inner(conn, query, limit, offset)
    }
    pub async fn from_select(
        conn: Arc<Mutex<Connection>>,
        query: String,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<(Vec<NoteView>, Vec<TagView>, u32)> {
        let conn = &*conn.lock().await;
        Self::from_select_inner(conn, query, limit, offset)
    }
    pub fn from_select_inner(
        conn: &Connection,
        query: String,
        limit: u32,
        offset: u32,
    ) -> anyhow::Result<(Vec<NoteView>, Vec<TagView>, u32)> {
        let text = do_search(conn, &query, &limit, &offset);
        let mut mdata = serde_json::from_str::<MiddleData>(&text)?;
        mdata.tags.sort_by(|a, b| b.count.cmp(&a.count));
        let notes: Vec<NoteView> = mdata
            .notes
            .into_iter()
            .map(move |note| note.into())
            .collect();
        let tags: Vec<TagView> = mdata
            .tags
            .into_iter()
            .filter(|tag| !tag.name.is_empty())
            .map(move |tag| TagView {
                tag,
                search_button: button::State::new(),
            })
            .collect();
        Ok((notes, tags, mdata.count))
    }
}

#[derive(Debug, Default)]
pub struct DataView {
    pub notes: Vec<NoteView>,
    //pub days: Vec<String>,
    pub tags: Vec<TagView>,
    pub state: State,
}
impl DataView {
    pub fn reset(&mut self) {
        let Self { state, .. } = self;
        let State {
            tags_scrollable,
            notes_scrollable,
        } = state;
        tags_scrollable.scroll_to(0.0, iced::Rectangle::default(), iced::Rectangle::default());
        notes_scrollable.scroll_to(0.0, iced::Rectangle::default(), iced::Rectangle::default());
    }
}
#[derive(Debug, Default)]
pub struct State {
    pub tags_scrollable: scrollable::State,
    pub notes_scrollable: scrollable::State,
}
