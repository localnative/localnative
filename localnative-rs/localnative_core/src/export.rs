// export.rs — Markdown export for Local Native notes
//
// Exports notes as individual .md files with YAML frontmatter.

use crate::db::{self, Note};
use rusqlite::Connection;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;

/// Sanitize a title into a filename-safe slug.
///
/// Non-alphanumeric characters become hyphens, consecutive hyphens are
/// collapsed, leading/trailing hyphens are stripped, and the result is
/// truncated so the full path always fits the OS limits (255 bytes is the
/// common denominator; 64 leaves ample room for suffixes).
fn sanitize_filename(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    // Collapse consecutive hyphens and trim.
    let mut result = String::new();
    let mut prev_hyphen = true; // treat start as hyphen to strip leading
    for c in slug.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push('-');
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }
    // Strip trailing hyphen
    if result.ends_with('-') {
        result.pop();
    }
    // Bound the length *in bytes* so multi-byte characters can't push the
    // filename over the filesystem limit, without splitting a character.
    if result.len() > 64 {
        let mut end = 64;
        while end > 0 && !result.is_char_boundary(end) {
            end -= 1;
        }
        result.truncate(end);
    }
    result
}

/// Quote a string for YAML using JSON quoting — valid YAML double-quoted
/// scalars use the same escapes, and unlike ad-hoc rules this never
/// mis-handles YAML-special unquoted values (`yes`, `null`, `3.14`, `: `).
fn yaml_escape(s: &str) -> String {
    serde_json::to_string(s).expect("strings always serialize")
}

/// Render a single note as a Markdown string with YAML frontmatter.
pub fn note_to_markdown(note: &Note) -> String {
    let tags: Vec<&str> = note
        .tags
        .split(',')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect();

    let tags_yaml = if tags.is_empty() {
        "[]".to_string()
    } else {
        let items: Vec<String> = tags.iter().map(|t| yaml_escape(t)).collect();
        format!("[{}]", items.join(", "))
    };

    let mut md = String::new();

    // YAML frontmatter
    md.push_str("---\n");
    md.push_str(&format!("uuid: {}\n", yaml_escape(&note.uuid4)));
    md.push_str(&format!("title: {}\n", yaml_escape(&note.title)));
    md.push_str(&format!("url: {}\n", yaml_escape(&note.url)));
    md.push_str(&format!("tags: {}\n", tags_yaml));
    md.push_str(&format!("created_at: {}\n", yaml_escape(&note.created_at)));
    md.push_str(&format!("is_public: {}\n", note.is_public));
    md.push_str("---\n\n");

    // Title
    let title = if note.title.is_empty() {
        "Untitled"
    } else {
        &note.title
    };
    md.push_str(&format!("# {}\n\n", title));

    // URL (only if non-empty)
    if !note.url.is_empty() {
        md.push_str(&format!("{}\n\n", note.url));
    }

    // Description
    if !note.description.is_empty() {
        md.push_str("## Description\n\n");
        md.push_str(&note.description);
        md.push_str("\n\n");
    }

    // Comments
    if !note.comments.is_empty() {
        md.push_str("## Comments\n\n");
        md.push_str(&note.comments);
        md.push('\n');
    }

    md
}

/// Outcome of a Markdown export.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ExportSummary {
    /// Notes written to disk.
    pub written: usize,
    /// Notes that could not be written (logged); the export continued.
    pub skipped: usize,
}

/// Export notes to a directory as individual Markdown files.
///
/// If `query` is `Some`, only notes matching the query are exported. A note
/// that cannot be written is skipped and counted — one bad title no longer
/// aborts the whole export — and only a failure to create the output
/// directory or read the database is an error.
pub fn export_notes(
    conn: &Connection,
    output_dir: &Path,
    query: Option<&str>,
) -> Result<ExportSummary, ExportError> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Fetch notes
    let notes = match query {
        Some(q) if !q.is_empty() => db::queries::search_all(conn, q)?,
        _ => db::queries::select_all(conn)?,
    };

    let mut used_names: HashSet<String> = HashSet::new();
    let mut summary = ExportSummary::default();

    for note in &notes {
        let base_name = sanitize_filename(&note.title);
        let base_name = if base_name.is_empty() {
            "untitled".to_string()
        } else {
            base_name
        };

        // Ensure unique filenames by appending uuid suffix on collision
        let file_name = if used_names.contains(&base_name) {
            let short_uuid = &note.uuid4[..8.min(note.uuid4.len())];
            format!("{}-{}", base_name, short_uuid)
        } else {
            base_name.clone()
        };
        used_names.insert(base_name);
        used_names.insert(file_name.clone());

        let file_path = output_dir.join(format!("{}.md", file_name));
        let markdown = note_to_markdown(note);
        match fs::write(&file_path, markdown) {
            Ok(()) => summary.written += 1,
            Err(e) => {
                tracing::warn!(uuid4 = %note.uuid4, %e, "skipped note during export");
                summary.skipped += 1;
            }
        }
    }

    Ok(summary)
}

#[derive(Debug)]
pub enum ExportError {
    Io(io::Error),
    Db(db::DbError),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Io(e) => write!(f, "IO error: {}", e),
            ExportError::Db(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for ExportError {}

impl From<io::Error> for ExportError {
    fn from(e: io::Error) -> Self {
        ExportError::Io(e)
    }
}

impl From<db::DbError> for ExportError {
    fn from(e: db::DbError) -> Self {
        ExportError::Db(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Hello World!"), "hello-world");
        assert_eq!(sanitize_filename("foo--bar"), "foo-bar");
        assert_eq!(sanitize_filename("  spaces  "), "spaces");
        assert_eq!(sanitize_filename(""), "");
        assert_eq!(sanitize_filename("My Note: A Title"), "my-note-a-title");
        assert_eq!(sanitize_filename("---leading---"), "leading");
    }

    #[test]
    fn test_sanitize_filename_is_bounded() {
        // 100 CJK characters is 300 bytes — well past any filename limit.
        let slug = sanitize_filename(&"长".repeat(100));
        assert!(
            slug.len() <= 64,
            "slug was {slug_len} bytes",
            slug_len = slug.len()
        );
        // Truncation lands on a character boundary.
        assert!(slug.chars().all(|c| c == '长'));
    }

    #[test]
    fn test_yaml_escape() {
        // Always quoted: the empty string, YAML-special unquoted words, and
        // anything with structure — JSON quoting is valid YAML.
        assert_eq!(yaml_escape("simple"), "\"simple\"");
        assert_eq!(yaml_escape("yes"), "\"yes\"");
        assert_eq!(yaml_escape("3.14"), "\"3.14\"");
        assert_eq!(yaml_escape("has: colon"), "\"has: colon\"");
        assert_eq!(yaml_escape("has \"quotes\""), "\"has \\\"quotes\\\"\"");
        assert_eq!(yaml_escape(""), "\"\"");
    }

    #[test]
    fn test_note_to_markdown_basic() {
        let note = Note {
            rowid: 1,
            uuid4: "abcd-1234".to_string(),
            title: "Test Note".to_string(),
            url: "https://example.com".to_string(),
            tags: "rust,local-first,sync".to_string(),
            description: "A test description.".to_string(),
            comments: "A comment.".to_string(),
            annotations: String::new(),
            created_at: "2024-01-15 10:30:00".to_string(),
            is_public: true,
            metadata: String::new(),
            updated_at: String::new(),
            deleted: false,
        };

        let md = note_to_markdown(&note);
        assert!(md.starts_with("---\n"));
        assert!(md.contains("uuid: \"abcd-1234\""));
        assert!(md.contains("title: \"Test Note\""));
        assert!(md.contains("url: \"https://example.com\""));
        assert!(md.contains("tags: [\"rust\", \"local-first\", \"sync\"]"));
        assert!(md.contains("is_public: true"));
        assert!(md.contains("# Test Note"));
        assert!(md.contains("## Description"));
        assert!(md.contains("A test description."));
        assert!(md.contains("## Comments"));
        assert!(md.contains("A comment."));
    }

    #[test]
    fn test_export_continues_past_unwritable_notes() {
        let path =
            std::env::temp_dir().join(format!("ln_export_test_{}.sqlite3", std::process::id()));
        let conn = crate::db::init_db_at(&path).unwrap();
        queries_for_export(&conn, "Ordinary note");
        queries_for_export(&conn, &"长".repeat(100));

        let out = std::env::temp_dir().join(format!("ln_export_test_out_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&out);
        let summary = export_notes(&conn, &out, None).unwrap();
        assert_eq!(summary.written, 2, "nothing skipped: {summary:?}");
        assert_eq!(summary.skipped, 0);
        let _ = std::fs::remove_dir_all(&out);
        let _ = std::fs::remove_file(&path);
    }

    fn queries_for_export(conn: &Connection, title: &str) {
        db::queries::insert_note(conn, title, "", "", "", "", b"", true).unwrap();
    }

    #[test]
    fn test_note_to_markdown_empty_fields() {
        let note = Note {
            rowid: 2,
            uuid4: "efgh-5678".to_string(),
            title: "".to_string(),
            url: "".to_string(),
            tags: "".to_string(),
            description: "".to_string(),
            comments: "".to_string(),
            annotations: String::new(),
            created_at: "2024-01-15 10:30:00".to_string(),
            is_public: false,
            metadata: String::new(),
            updated_at: String::new(),
            deleted: false,
        };

        let md = note_to_markdown(&note);
        assert!(md.contains("title: \"\""));
        assert!(md.contains("tags: []"));
        assert!(md.contains("# Untitled"));
        assert!(!md.contains("## Description"));
        assert!(!md.contains("## Comments"));
    }

    #[test]
    fn test_export_notes_creates_files() {
        let path =
            std::env::temp_dir().join(format!("ln_export_files_{}.sqlite3", std::process::id()));
        let conn = crate::db::init_db_at(&path).unwrap();
        db::queries::insert_note(
            &conn,
            "First Note",
            "https://example.com",
            "tag1,tag2",
            "Desc 1",
            "Comment 1",
            b"",
            true,
        )
        .unwrap();
        db::queries::insert_note(
            &conn,
            "Second Note",
            "https://example.org",
            "tag3",
            "Desc 2",
            "",
            b"",
            false,
        )
        .unwrap();

        let tmp_dir =
            std::env::temp_dir().join(format!("ln_export_files_out_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp_dir);

        let summary = export_notes(&conn, &tmp_dir, None).unwrap();
        assert_eq!(summary.written, 2);
        assert_eq!(summary.skipped, 0);
        assert!(tmp_dir.join("first-note.md").exists());
        assert!(tmp_dir.join("second-note.md").exists());

        // Verify content of the second note
        let content = fs::read_to_string(tmp_dir.join("second-note.md")).unwrap();
        assert!(content.contains("uuid: \""));
        assert!(content.contains("title: \"Second Note\""));
        assert!(content.contains("is_public: false"));

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_export_duplicate_titles() {
        let path =
            std::env::temp_dir().join(format!("ln_export_dup_{}.sqlite3", std::process::id()));
        let conn = crate::db::init_db_at(&path).unwrap();
        db::queries::insert_note(&conn, "Same Title", "", "", "", "", b"", false).unwrap();
        db::queries::insert_note(&conn, "Same Title", "", "", "", "", b"", false).unwrap();

        let tmp_dir =
            std::env::temp_dir().join(format!("ln_export_dup_out_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp_dir);

        let summary = export_notes(&conn, &tmp_dir, None).unwrap();
        assert_eq!(summary.written, 2);
        // First one gets the clean name, second gets uuid suffix
        assert!(tmp_dir.join("same-title.md").exists());
        let entries: Vec<_> = fs::read_dir(&tmp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 2);

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
