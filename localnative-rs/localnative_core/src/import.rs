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

use crate::db::{self, DbResult};
use regex::Regex;
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::OnceLock;

/// A note parsed from an external source, ready for import.
#[derive(Debug, Clone)]
pub struct ImportedNote {
    pub title: String,
    pub url: String,
    pub tags: String,
    pub description: String,
    pub created_at: Option<String>,
}

/// Summary of an import operation.
#[derive(Debug)]
pub struct ImportResult {
    pub total_found: usize,
    pub imported: usize,
    pub skipped_duplicate: usize,
    pub errors: usize,
}

impl std::fmt::Display for ImportResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Import complete: {} found, {} imported, {} skipped (duplicate URL), {} errors",
            self.total_found, self.imported, self.skipped_duplicate, self.errors
        )
    }
}

// ---------------------------------------------------------------------------
// Pocket HTML export parser
// ---------------------------------------------------------------------------

fn pocket_link_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"<a\s+[^>]*href="([^"]*)"[^>]*>"#).expect("invalid pocket link regex")
    })
}

fn pocket_attr_re(attr: &str) -> Regex {
    Regex::new(&format!(r#"{}="([^"]*)""#, regex::escape(attr))).expect("invalid attr regex")
}

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

/// Parse a Pocket HTML export file into a list of imported notes.
///
/// Pocket exports are structured HTML with `<a>` tags inside `<li>` items.
/// Each `<a>` has `href`, `time_added`, and `tags` attributes.
pub fn parse_pocket_html(content: &str) -> Vec<ImportedNote> {
    let link_re = pocket_link_re();
    let time_re = pocket_attr_re("time_added");
    let tags_re = pocket_attr_re("tags");

    let mut notes = Vec::new();

    // Extract the inner text between <a ...>TITLE</a>
    let title_re: &Regex = {
        static RE: OnceLock<Regex> = OnceLock::new();
        RE.get_or_init(|| Regex::new(r#"<a\s+[^>]*>([^<]*)</a>"#).expect("invalid title regex"))
    };

    // We iterate over every <a> tag that has an href
    for link_cap in link_re.captures_iter(content) {
        let full_match = link_cap.get(0).unwrap().as_str();
        let url = decode_html_entities(&link_cap[1]);

        if url.is_empty() {
            continue;
        }

        // Find the title: look for the corresponding </a> closing tag
        let match_start = link_cap.get(0).unwrap().start();
        let remainder = &content[match_start..];
        let title = title_re
            .captures(remainder)
            .map(|c| decode_html_entities(c.get(1).unwrap().as_str()))
            .unwrap_or_default();

        // Extract time_added attribute (Unix timestamp)
        let created_at = time_re.captures(full_match).and_then(|c| {
            let ts_str = c.get(1).unwrap().as_str();
            ts_str.parse::<i64>().ok().map(|ts| {
                chrono::DateTime::from_timestamp(ts, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default()
            })
        });

        // Extract tags attribute (comma-separated)
        let tags = tags_re
            .captures(full_match)
            .map(|c| {
                let raw = c.get(1).unwrap().as_str();
                decode_html_entities(raw)
            })
            .unwrap_or_default();

        notes.push(ImportedNote {
            title,
            url,
            tags,
            description: String::new(),
            created_at,
        });
    }

    notes
}

// ---------------------------------------------------------------------------
// Omnivore JSON export parser
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug)]
struct OmnivoreLabel {
    name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct OmnivoreEntry {
    title: Option<String>,
    url: Option<String>,
    description: Option<String>,
    saved_at: Option<String>,
    labels: Option<Vec<OmnivoreLabel>>,
}

/// Parse an Omnivore JSON export file into a list of imported notes.
///
/// Omnivore exports are JSON arrays of objects with fields like `title`, `url`,
/// `description`, `savedAt`, and `labels`.
pub fn parse_omnivore_json(content: &str) -> Result<Vec<ImportedNote>, serde_json::Error> {
    let entries: Vec<OmnivoreEntry> = serde_json::from_str(content)?;

    let notes = entries
        .into_iter()
        .filter_map(|entry| {
            let url = entry.url.unwrap_or_default();
            if url.is_empty() {
                return None;
            }

            let tags = entry
                .labels
                .unwrap_or_default()
                .into_iter()
                .map(|l| l.name)
                .collect::<Vec<_>>()
                .join(",");

            // Parse savedAt (ISO 8601) into our created_at format
            let created_at = entry.saved_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            });

            Some(ImportedNote {
                title: entry.title.unwrap_or_default(),
                url,
                tags,
                description: entry.description.unwrap_or_default(),
                created_at,
            })
        })
        .collect();

    Ok(notes)
}

// ---------------------------------------------------------------------------
// Raindrop.io CSV export parser
// ---------------------------------------------------------------------------

/// Parse a single CSV line respecting quoted fields.
///
/// Fields may be enclosed in double quotes. A doubled quote `""` inside a
/// quoted field represents a literal `"`. Commas inside quoted fields are not
/// treated as separators.
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    // Escaped quote
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(ch);
            }
        } else if ch == '"' {
            in_quotes = true;
        } else if ch == ',' {
            fields.push(current.trim().to_string());
            current = String::new();
        } else {
            current.push(ch);
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Parse a Raindrop.io CSV export file into a list of imported notes.
///
/// Raindrop CSV columns (in order):
/// `id, title, note, excerpt, url, folder, tags, created, cover, highlights, favorite`
///
/// Field mapping:
/// - `title` -> title
/// - `url` -> url
/// - `tags` -> tags (already comma-separated, wrapped in extra quotes)
/// - `excerpt` -> description
/// - `created` -> created_at (ISO 8601)
pub fn parse_raindrop_csv(content: &str) -> Vec<ImportedNote> {
    let mut notes = Vec::new();

    let mut lines = content.lines();

    // Skip the header line
    if lines.next().is_none() {
        return notes;
    }

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }

        let fields = parse_csv_line(line);

        // We need at least 8 columns (up to `created` at index 7)
        if fields.len() < 8 {
            continue;
        }

        let title = fields[1].clone();
        let excerpt = fields[3].clone();
        let url = fields[4].clone();
        let tags = fields[6].clone();
        let created_raw = &fields[7];

        if url.is_empty() {
            continue;
        }

        // Parse ISO 8601 created timestamp
        let created_at = chrono::DateTime::parse_from_rfc3339(created_raw)
            .ok()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string());

        notes.push(ImportedNote {
            title,
            url,
            tags,
            description: excerpt,
            created_at,
        });
    }

    notes
}

// ---------------------------------------------------------------------------
// Import into database
// ---------------------------------------------------------------------------

/// Check whether a note with the given URL already exists in the database.
fn url_exists(conn: &Connection, url: &str) -> DbResult<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM note WHERE url = ?1",
        rusqlite::params![url],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Import a list of parsed notes into the database, skipping duplicates by URL.
pub fn import_notes(conn: &Connection, notes: Vec<ImportedNote>) -> DbResult<ImportResult> {
    let total_found = notes.len();
    let mut imported = 0;
    let mut skipped_duplicate = 0;
    let mut errors = 0;

    for note in notes {
        if url_exists(conn, &note.url)? {
            skipped_duplicate += 1;
            continue;
        }

        let result = if let Some(ref created_at) = note.created_at {
            db::queries::insert_note_with_timestamp(
                conn,
                &note.title,
                &note.url,
                &note.tags,
                &note.description,
                "",
                &[],
                true,
                created_at,
            )
        } else {
            db::queries::insert_note(
                conn,
                &note.title,
                &note.url,
                &note.tags,
                &note.description,
                "",
                &[],
                true,
            )
        };

        match result {
            Ok(_) => imported += 1,
            Err(e) => {
                tracing::warn!(url = %note.url, error = %e, "failed to import note");
                errors += 1;
            }
        }
    }

    Ok(ImportResult {
        total_found,
        imported,
        skipped_duplicate,
        errors,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pocket_html() {
        let html = r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"><title>Pocket Export</title></head>
<body>
<h1>Unread</h1>
<ul>
  <li><a href="https://example.com/article1" time_added="1700000000" tags="rust,programming">Rust Article</a></li>
  <li><a href="https://example.com/article2" time_added="1700100000" tags="">Another Article</a></li>
  <li><a href="https://example.com/article3" time_added="1700200000" tags="news">News &amp; Updates</a></li>
</ul>
</body>
</html>"#;

        let notes = parse_pocket_html(html);
        assert_eq!(notes.len(), 3);

        assert_eq!(notes[0].url, "https://example.com/article1");
        assert_eq!(notes[0].title, "Rust Article");
        assert_eq!(notes[0].tags, "rust,programming");
        assert!(notes[0].created_at.is_some());

        assert_eq!(notes[1].url, "https://example.com/article2");
        assert_eq!(notes[1].title, "Another Article");
        assert_eq!(notes[1].tags, "");

        assert_eq!(notes[2].title, "News & Updates");
        assert_eq!(notes[2].tags, "news");
    }

    #[test]
    fn test_parse_pocket_html_empty() {
        let html = r#"<!DOCTYPE html><html><body><ul></ul></body></html>"#;
        let notes = parse_pocket_html(html);
        assert!(notes.is_empty());
    }

    #[test]
    fn test_parse_omnivore_json() {
        let json = r#"[
            {
                "title": "First Article",
                "url": "https://example.com/first",
                "description": "A great article",
                "savedAt": "2024-01-15T10:30:00Z",
                "labels": [{"name": "tech"}, {"name": "rust"}]
            },
            {
                "title": "Second Article",
                "url": "https://example.com/second",
                "description": null,
                "savedAt": "2024-02-20T14:00:00Z",
                "labels": []
            },
            {
                "title": "No URL Entry",
                "url": "",
                "description": "Should be skipped",
                "savedAt": null,
                "labels": null
            }
        ]"#;

        let notes = parse_omnivore_json(json).unwrap();
        assert_eq!(notes.len(), 2);

        assert_eq!(notes[0].title, "First Article");
        assert_eq!(notes[0].url, "https://example.com/first");
        assert_eq!(notes[0].description, "A great article");
        assert_eq!(notes[0].tags, "tech,rust");
        assert!(notes[0].created_at.is_some());

        assert_eq!(notes[1].title, "Second Article");
        assert_eq!(notes[1].description, "");
        assert_eq!(notes[1].tags, "");
    }

    #[test]
    fn test_parse_omnivore_json_empty() {
        let notes = parse_omnivore_json("[]").unwrap();
        assert!(notes.is_empty());
    }

    #[test]
    fn test_parse_omnivore_json_invalid() {
        assert!(parse_omnivore_json("not json").is_err());
    }

    #[test]
    fn test_parse_raindrop_csv() {
        let csv = r#"id,title,note,excerpt,url,folder,tags,created,cover,highlights,favorite
1001,"Rust by Example","","Learn Rust with examples","https://doc.rust-lang.org/rust-by-example/","tutorials","rust, programming","2024-03-15T09:30:00Z","","",false
1002,"Hacker News","my note","Tech news aggregator","https://news.ycombinator.com/","news","","2024-04-01T12:00:00Z","","",true
1003,"Quoted ""Title""","","An excerpt with, commas","https://example.com/quoted","misc","tag1, ""tag2""","2024-05-10T08:00:00Z","","",false"#;

        let notes = parse_raindrop_csv(csv);
        assert_eq!(notes.len(), 3);

        assert_eq!(notes[0].title, "Rust by Example");
        assert_eq!(notes[0].url, "https://doc.rust-lang.org/rust-by-example/");
        assert_eq!(notes[0].tags, "rust, programming");
        assert_eq!(notes[0].description, "Learn Rust with examples");
        assert_eq!(
            notes[0].created_at.as_deref(),
            Some("2024-03-15 09:30:00")
        );

        assert_eq!(notes[1].title, "Hacker News");
        assert_eq!(notes[1].url, "https://news.ycombinator.com/");
        assert_eq!(notes[1].tags, "");
        assert_eq!(notes[1].description, "Tech news aggregator");

        assert_eq!(notes[2].title, "Quoted \"Title\"");
        assert_eq!(notes[2].description, "An excerpt with, commas");
    }

    #[test]
    fn test_parse_raindrop_csv_empty() {
        let csv = "id,title,note,excerpt,url,folder,tags,created,cover,highlights,favorite\n";
        let notes = parse_raindrop_csv(csv);
        assert!(notes.is_empty());
    }

    #[test]
    fn test_parse_raindrop_csv_header_only() {
        let csv = "id,title,note,excerpt,url,folder,tags,created,cover,highlights,favorite";
        let notes = parse_raindrop_csv(csv);
        assert!(notes.is_empty());
    }

    #[test]
    fn test_parse_raindrop_csv_skip_empty_url() {
        let csv = "id,title,note,excerpt,url,folder,tags,created,cover,highlights,favorite\n1001,\"Title\",\"\",\"desc\",\"\",\"folder\",\"tag\",\"2024-01-01T00:00:00Z\",\"\",\"\",false";
        let notes = parse_raindrop_csv(csv);
        assert!(notes.is_empty());
    }

    #[test]
    fn test_parse_csv_line() {
        let fields = parse_csv_line(r#"1,"hello, world","",test"#);
        assert_eq!(fields, vec!["1", "hello, world", "", "test"]);

        let fields = parse_csv_line(r#""a ""b"" c",simple"#);
        assert_eq!(fields, vec!["a \"b\" c", "simple"]);
    }

    #[test]
    fn test_import_to_db() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note (
                uuid4 TEXT NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                url TEXT NOT NULL DEFAULT '',
                tags TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                comments TEXT NOT NULL DEFAULT '',
                annotations BLOB NOT NULL DEFAULT x'',
                created_at TEXT NOT NULL DEFAULT '',
                is_public BOOLEAN NOT NULL DEFAULT 1,
                metadata TEXT NOT NULL DEFAULT ''
            );
            CREATE TABLE meta (meta_key TEXT PRIMARY KEY, meta_value TEXT);
            INSERT INTO meta VALUES ('version', '0.7.0');",
        )
        .unwrap();

        let notes = vec![
            ImportedNote {
                title: "Test 1".into(),
                url: "https://example.com/1".into(),
                tags: "tag1".into(),
                description: "Desc 1".into(),
                created_at: Some("2024-01-01 00:00:00".into()),
            },
            ImportedNote {
                title: "Test 2".into(),
                url: "https://example.com/2".into(),
                tags: "tag2".into(),
                description: "Desc 2".into(),
                created_at: None,
            },
        ];

        let result = import_notes(&conn, notes).unwrap();
        assert_eq!(result.total_found, 2);
        assert_eq!(result.imported, 2);
        assert_eq!(result.skipped_duplicate, 0);

        // Import again — should skip duplicates
        let notes2 = vec![
            ImportedNote {
                title: "Test 1 Again".into(),
                url: "https://example.com/1".into(),
                tags: "".into(),
                description: "".into(),
                created_at: None,
            },
            ImportedNote {
                title: "Test 3".into(),
                url: "https://example.com/3".into(),
                tags: "".into(),
                description: "".into(),
                created_at: None,
            },
        ];

        let result2 = import_notes(&conn, notes2).unwrap();
        assert_eq!(result2.total_found, 2);
        assert_eq!(result2.imported, 1);
        assert_eq!(result2.skipped_duplicate, 1);
    }
}
