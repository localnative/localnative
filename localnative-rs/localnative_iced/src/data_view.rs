use crate::{days::Day, page_bar::PageBar, tags::TagView};
use crate::{
    note::{Note, NoteView},
    tags::Tag,
};

use iced::{button, scrollable};
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

impl MiddleData {
    pub fn encode(self, data_view: &mut DataView, page_info: &mut PageBar) {
        let notes: Vec<NoteView> = self
            .notes
            .into_iter()
            .map(move |note| note.into())
            .collect();
        let tags: Vec<TagView> = self
            .tags
            .into_iter()
            .map(move |tag| TagView {
                tag,
                search_button: button::State::new(),
            })
            .collect();
        data_view.notes = notes;
        data_view.tags = tags;
        // TODO:
        // data_view.days = self.days;
        page_info.count = self.count;
    }
    pub fn upgrade(conn: &Connection) {
        if let Ok(version) = localnative_core::upgrade::upgrade(&conn) {
            log::debug!("upgrade done:{}", version);
        } else {
            log::error!("upgrade error");
        }
    }
    pub fn from_select(conn: &Connection, query: &str, limit: &u32, offset: &u32) -> MiddleData {
        Self::upgrade(conn);
        let text = do_search(conn, query, limit, offset);
        log::debug!("{}", text);
        if let Ok(res) = serde_json::from_str(&text) {
            log::info!("Select data success.");
            res
        } else {
            log::warn!("Select data fail.");
            MiddleData::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct DataView {
    pub notes: Vec<NoteView>,
    //pub days: Vec<String>,
    pub tags: Vec<TagView>,
    pub state: State,
}

#[derive(Debug, Default)]
pub struct State {
    pub tags_scrollable: scrollable::State,
    pub notes_scrollable: scrollable::State,
}
