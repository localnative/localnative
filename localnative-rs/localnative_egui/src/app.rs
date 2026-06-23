/*
    Local Native
    Copyright (C) 2018-2019  Yi Wang

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Application state and UI for the egui front-end.
//!
//! All database work goes through `localnative_core::db::queries` on the UI
//! thread (local SQLite is fast). Peer sync is the one slow operation, so it is
//! run on a background thread via `localnative_core::run_sync` and polled each
//! frame through an `mpsc` channel.

use std::sync::mpsc::{self, Receiver};
use std::thread;

use eframe::egui;
use localnative_core::db::models::{Note, QueryResult};
use localnative_core::db::{self, Pool, queries};

/// Number of notes shown per page.
const PAGE_SIZE: u32 = 20;

/// Deferred pagination action, applied after the read-only render borrow ends.
enum Page {
    None,
    Prev,
    Next,
}

pub struct LocalNativeApp {
    /// `None` when the database failed to open; `status` then holds the reason.
    pool: Option<Pool>,
    /// Current full-text query. An empty string lists all notes, newest first.
    query: String,
    result: QueryResult,
    offset: u32,
    status: String,

    // Add-note form.
    show_add: bool,
    new_title: String,
    new_url: String,
    new_tags: String,
    new_description: String,
    new_comments: String,
    new_is_public: bool,

    // Peer sync.
    sync_addr: String,
    /// `Some` while a background sync is in flight.
    sync_rx: Option<Receiver<String>>,
}

impl LocalNativeApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (pool, status) = match db::init_pool() {
            Ok(pool) => (Some(pool), String::new()),
            Err(e) => (None, format!("failed to open database: {e}")),
        };

        let mut app = Self {
            pool,
            query: String::new(),
            result: QueryResult::default(),
            offset: 0,
            status,
            show_add: false,
            new_title: String::new(),
            new_url: String::new(),
            new_tags: String::new(),
            new_description: String::new(),
            new_comments: String::new(),
            new_is_public: true,
            sync_addr: String::from("127.0.0.1:2345"),
            sync_rx: None,
        };
        app.refresh();
        app
    }

    /// Re-run the current query at the current offset and replace `result`.
    fn refresh(&mut self) {
        let Some(pool) = self.pool.as_ref() else {
            return;
        };
        let conn = match pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                self.status = format!("database connection error: {e}");
                return;
            }
        };
        match queries::do_search(&conn, &self.query, PAGE_SIZE, self.offset) {
            Ok(mut result) => {
                result.tags.sort_by_key(|t| std::cmp::Reverse(t.count));
                self.result = result;
                self.status = if self.query.is_empty() {
                    format!("{} note(s)", self.result.count)
                } else {
                    format!("{} note(s) matching \"{}\"", self.result.count, self.query)
                };
            }
            Err(e) => self.status = format!("search error: {e}"),
        }
    }

    /// Reset to the first page and re-run the query.
    fn run_query(&mut self) {
        self.offset = 0;
        self.refresh();
    }

    fn add_note(&mut self) {
        let Some(pool) = self.pool.as_ref() else {
            return;
        };
        if self.new_title.trim().is_empty() && self.new_url.trim().is_empty() {
            self.status = "a note needs at least a title or a URL".to_string();
            return;
        }
        let conn = match pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                self.status = format!("database connection error: {e}");
                return;
            }
        };
        match queries::insert_note(
            &conn,
            self.new_title.trim(),
            self.new_url.trim(),
            self.new_tags.trim(),
            self.new_description.trim(),
            self.new_comments.trim(),
            &[],
            self.new_is_public,
        ) {
            Ok(_) => {
                self.new_title.clear();
                self.new_url.clear();
                self.new_tags.clear();
                self.new_description.clear();
                self.new_comments.clear();
                self.show_add = false;
                self.run_query();
            }
            Err(e) => self.status = format!("insert error: {e}"),
        }
    }

    fn delete_note(&mut self, rowid: i64) {
        let Some(pool) = self.pool.as_ref() else {
            return;
        };
        let conn = match pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                self.status = format!("database connection error: {e}");
                return;
            }
        };
        match queries::delete_note(&conn, rowid) {
            Ok(()) => self.refresh(),
            Err(e) => self.status = format!("delete error: {e}"),
        }
    }

    /// Spawn a background thread that syncs against `sync_addr`.
    fn start_sync(&mut self) {
        if self.sync_rx.is_some() {
            return; // already running
        }
        let addr = self.sync_addr.trim().to_string();
        if addr.is_empty() {
            self.status = "enter a peer address first (e.g. 192.168.1.5:2345)".to_string();
            return;
        }
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let cmd = serde_json::json!({ "action": "client-sync", "addr": addr });
            let resp = localnative_core::run_sync(&cmd.to_string());
            let _ = tx.send(resp);
        });
        self.status = format!("syncing with {}…", self.sync_addr.trim());
        self.sync_rx = Some(rx);
    }

    /// Poll the background sync thread; refresh notes once it finishes.
    fn poll_sync(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.sync_rx.as_ref() else {
            return;
        };
        match rx.try_recv() {
            Ok(resp) => {
                self.sync_rx = None;
                self.status = summarize_sync(&resp);
                self.refresh();
            }
            Err(mpsc::TryRecvError::Empty) => {
                // Still running — keep repainting so completion is noticed promptly.
                ctx.request_repaint();
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.sync_rx = None;
                self.status = "sync thread terminated unexpectedly".to_string();
            }
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("Local Native");
                ui.separator();
                ui.label("Search:");
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("full-text query — blank shows all")
                        .desired_width(280.0),
                );
                let entered = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if entered || ui.button("Search").clicked() {
                    self.run_query();
                }
                if ui.button("Clear").clicked() {
                    self.query.clear();
                    self.run_query();
                }
                ui.separator();
                let add_label = if self.show_add { "Close" } else { "Add note" };
                if ui.button(add_label).clicked() {
                    self.show_add = !self.show_add;
                }
            });
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label("Peer:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.sync_addr)
                        .hint_text("host:port")
                        .desired_width(160.0),
                );
                let syncing = self.sync_rx.is_some();
                if ui
                    .add_enabled(!syncing, egui::Button::new("Sync"))
                    .clicked()
                {
                    self.start_sync();
                }
                if syncing {
                    ui.spinner();
                }
            });
            ui.add_space(2.0);
            ui.label(self.status.as_str());
            ui.add_space(4.0);
        });
    }

    fn tags_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("tags")
            .resizable(true)
            .default_size(180.0)
            .show_inside(ui, |ui| {
                ui.add_space(4.0);
                ui.heading("Tags");
                ui.separator();
                let mut clicked: Option<String> = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.result.tags.is_empty() {
                        ui.label(egui::RichText::new("no tags").weak());
                    }
                    for tag in &self.result.tags {
                        if tag.tag.trim().is_empty() {
                            continue;
                        }
                        let label = format!("{}  ({})", tag.tag, tag.count);
                        if ui.selectable_label(false, label).clicked() {
                            clicked = Some(tag.tag.clone());
                        }
                    }
                });
                if let Some(tag) = clicked {
                    self.query = tag;
                    self.run_query();
                }
            });
    }

    fn add_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("add_note")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(4.0);
                ui.heading("Add note");
                egui::Grid::new("add_form")
                    .num_columns(2)
                    .spacing([8.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Title");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_title)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();
                        ui.label("URL");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_url)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();
                        ui.label("Tags");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_tags)
                                .hint_text("comma,separated")
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();
                        ui.label("Description");
                        ui.add(
                            egui::TextEdit::multiline(&mut self.new_description)
                                .desired_rows(2)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();
                        ui.label("Comments");
                        ui.add(
                            egui::TextEdit::multiline(&mut self.new_comments)
                                .desired_rows(2)
                                .desired_width(f32::INFINITY),
                        );
                        ui.end_row();
                    });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.new_is_public, "Public");
                    if ui.button("Save").clicked() {
                        self.add_note();
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_add = false;
                    }
                });
                ui.add_space(4.0);
            });
    }

    fn notes_panel(&mut self, ui: &mut egui::Ui) {
        let mut to_delete: Option<i64> = None;
        let mut to_open: Option<String> = None;
        let mut page = Page::None;

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let total = self.result.count;
                let start = if total == 0 { 0 } else { self.offset + 1 };
                let end = (self.offset + PAGE_SIZE).min(total);
                ui.label(format!("Showing {start}–{end} of {total}"));
                if ui
                    .add_enabled(self.offset > 0, egui::Button::new("Prev"))
                    .clicked()
                {
                    page = Page::Prev;
                }
                if ui
                    .add_enabled(self.offset + PAGE_SIZE < total, egui::Button::new("Next"))
                    .clicked()
                {
                    page = Page::Next;
                }
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.result.notes.is_empty() {
                    ui.add_space(20.0);
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("No notes to show.").weak());
                    });
                }
                for note in &self.result.notes {
                    render_note(ui, note, &mut to_open, &mut to_delete);
                    ui.separator();
                }
            });
        });

        // Apply deferred actions now that the read-only borrow of `self.result`
        // has ended.
        match page {
            Page::Prev => {
                self.offset = self.offset.saturating_sub(PAGE_SIZE);
                self.refresh();
            }
            Page::Next => {
                self.offset += PAGE_SIZE;
                self.refresh();
            }
            Page::None => {}
        }
        if let Some(url) = to_open
            && let Err(e) = open::that(&url)
        {
            self.status = format!("could not open {url}: {e}");
        }
        if let Some(rowid) = to_delete {
            self.delete_note(rowid);
        }
    }
}

impl eframe::App for LocalNativeApp {
    // eframe 0.34 drives the app through `ui` (given the full-window root `Ui`);
    // `update` is deprecated. Panels carve space from `ui` via `show_inside`.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_sync(ui.ctx());
        self.toolbar(ui);
        if self.show_add {
            self.add_panel(ui);
        }
        self.tags_panel(ui);
        self.notes_panel(ui);
    }
}

/// Render a single note as a card-like block. Free function so it borrows only
/// `note` (not the whole app), letting the caller defer mutations.
fn render_note(
    ui: &mut egui::Ui,
    note: &Note,
    to_open: &mut Option<String>,
    to_delete: &mut Option<i64>,
) {
    ui.horizontal(|ui| {
        let title = if note.title.trim().is_empty() {
            "(untitled)"
        } else {
            note.title.trim()
        };
        ui.label(egui::RichText::new(title).strong().size(15.0));
        if !note.is_public {
            ui.label(egui::RichText::new("private").weak().small());
        }
    });

    if !note.url.is_empty() && ui.link(note.url.as_str()).clicked() {
        *to_open = Some(note.url.clone());
    }

    if !note.description.trim().is_empty() {
        ui.label(note.description.as_str());
    }

    if !note.tags.trim().is_empty() {
        ui.horizontal_wrapped(|ui| {
            for tag in note
                .tags
                .split(',')
                .map(|t| t.trim())
                .filter(|t| !t.is_empty())
            {
                ui.label(egui::RichText::new(format!("#{tag}")).small().weak());
            }
        });
    }

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(note.created_at.as_str()).small().weak());
        if !note.url.is_empty() && ui.small_button("Open").clicked() {
            *to_open = Some(note.url.clone());
        }
        if ui.small_button("Delete").clicked() {
            *to_delete = Some(note.rowid);
        }
    });
}

/// Turn the JSON returned by `run_sync` into a short status line.
fn summarize_sync(resp: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(resp) {
        Ok(v) => {
            if let Some(msg) = v.get("client-sync").and_then(|m| m.as_str()) {
                format!("sync complete: {msg}")
            } else if let Some(err) = v.get("error") {
                format!("sync failed: {err}")
            } else {
                format!("sync: {resp}")
            }
        }
        Err(_) => format!("sync: {resp}"),
    }
}

#[cfg(test)]
mod tests {
    use super::summarize_sync;

    #[test]
    fn summarize_sync_reports_success() {
        // `run_sync` wraps a successful client sync as {"client-sync": "<msg>"}.
        let out = summarize_sync(r#"{"client-sync":"synced 3 notes"}"#);
        assert_eq!(out, "sync complete: synced 3 notes");
    }

    #[test]
    fn summarize_sync_reports_error() {
        let out = summarize_sync(r#"{"error":"connection refused","source_text":"x"}"#);
        assert!(out.starts_with("sync failed:"), "got: {out}");
    }

    #[test]
    fn summarize_sync_passes_through_unknown_json() {
        let out = summarize_sync(r#"{"server":"started"}"#);
        assert!(out.starts_with("sync:"), "got: {out}");
    }

    #[test]
    fn summarize_sync_handles_non_json() {
        assert_eq!(summarize_sync("boom"), "sync: boom");
    }
}
