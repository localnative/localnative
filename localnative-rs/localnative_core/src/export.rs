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
/// Non-alphanumeric characters become hyphens, consecutive hyphens
/// are collapsed, and leading/trailing hyphens are stripped.
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
    result
}

/// Escape a string value for YAML (wrap in quotes if it contains special chars).
fn yaml_escape(s: &str) -> String {
    if s.is_empty() {
        return "\"\"".to_string();
    }
    // If the string contains characters that need quoting in YAML
    if s.contains(':')
        || s.contains('#')
        || s.contains('\'')
        || s.contains('"')
        || s.contains('\n')
        || s.contains('\\')
        || s.starts_with(' ')
        || s.ends_with(' ')
        || s.starts_with('[')
        || s.starts_with('{')
    {
        // Use double-quoted form, escaping inner double-quotes and backslashes
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
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

/// Export notes to a directory as individual Markdown files.
///
/// If `query` is `Some`, only notes matching the query are exported.
/// Returns the number of notes exported.
pub fn export_notes(conn: &Connection, output_dir: &Path, query: Option<&str>) -> Result<usize, ExportError> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Fetch notes
    let notes = match query {
        Some(q) if !q.is_empty() => db::queries::search_all(conn, q)?,
        _ => db::queries::select_all(conn)?,
    };

    let mut used_names: HashSet<String> = HashSet::new();
    let mut count = 0;

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
        fs::write(&file_path, markdown)?;
        count += 1;
    }

    Ok(count)
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
        assert_eq!(
            sanitize_filename("My Note: A Title"),
            "my-note-a-title"
        );
        assert_eq!(sanitize_filename("---leading---"), "leading");
    }

    #[test]
    fn test_yaml_escape() {
        assert_eq!(yaml_escape("simple"), "simple");
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
        };

        let md = note_to_markdown(&note);
        assert!(md.starts_with("---\n"));
        assert!(md.contains("uuid: abcd-1234"));
        assert!(md.contains("title: Test Note"));
        assert!(md.contains("url: \"https://example.com\""));
        assert!(md.contains("tags: [rust, local-first, sync]"));
        assert!(md.contains("is_public: true"));
        assert!(md.contains("# Test Note"));
        assert!(md.contains("## Description"));
        assert!(md.contains("A test description."));
        assert!(md.contains("## Comments"));
        assert!(md.contains("A comment."));
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
        use rusqlite::Connection;

        let conn = Connection::open_in_memory().unwrap();
        // Create schema
        conn.execute_batch(
            "CREATE TABLE note (
                uuid4 TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                tags TEXT NOT NULL,
                description TEXT NOT NULL,
                comments TEXT NOT NULL,
                annotations BLOB NOT NULL,
                created_at TEXT NOT NULL,
                is_public INTEGER NOT NULL DEFAULT 0,
                metadata TEXT NOT NULL DEFAULT ''
            );
            CREATE VIRTUAL TABLE note_fts USING fts5(
                title, url, tags, description,
                content='note', content_rowid='rowid'
            );
            INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
            VALUES ('uuid-1111', 'First Note', 'https://example.com', 'tag1,tag2', 'Desc 1', 'Comment 1', '', '2024-01-15 10:00:00', 1);
            INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
            VALUES ('uuid-2222', 'Second Note', 'https://example.org', 'tag3', 'Desc 2', '', '', '2024-01-16 11:00:00', 0);
            INSERT INTO note_fts(rowid, title, url, tags, description)
            SELECT rowid, title, url, tags, description FROM note;",
        ).unwrap();

        let tmp_dir = std::env::temp_dir().join("localnative_export_test");
        let _ = fs::remove_dir_all(&tmp_dir);

        let count = export_notes(&conn, &tmp_dir, None).unwrap();
        assert_eq!(count, 2);
        assert!(tmp_dir.join("first-note.md").exists());
        assert!(tmp_dir.join("second-note.md").exists());

        // Verify content of first note
        let content = fs::read_to_string(tmp_dir.join("second-note.md")).unwrap();
        assert!(content.contains("uuid: uuid-2222"));
        assert!(content.contains("is_public: false"));

        // Cleanup
        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_export_duplicate_titles() {
        use rusqlite::Connection;

        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE note (
                uuid4 TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                tags TEXT NOT NULL,
                description TEXT NOT NULL,
                comments TEXT NOT NULL,
                annotations BLOB NOT NULL,
                created_at TEXT NOT NULL,
                is_public INTEGER NOT NULL DEFAULT 0,
                metadata TEXT NOT NULL DEFAULT ''
            );
            CREATE VIRTUAL TABLE note_fts USING fts5(
                title, url, tags, description,
                content='note', content_rowid='rowid'
            );
            INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
            VALUES ('uuid-aaaa', 'Same Title', '', '', '', '', '', '2024-01-15 10:00:00', 0);
            INSERT INTO note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
            VALUES ('uuid-bbbb', 'Same Title', '', '', '', '', '', '2024-01-16 11:00:00', 0);
            INSERT INTO note_fts(rowid, title, url, tags, description)
            SELECT rowid, title, url, tags, description FROM note;",
        ).unwrap();

        let tmp_dir = std::env::temp_dir().join("localnative_export_dup_test");
        let _ = fs::remove_dir_all(&tmp_dir);

        let count = export_notes(&conn, &tmp_dir, None).unwrap();
        assert_eq!(count, 2);
        // First one gets the clean name, second gets uuid suffix
        assert!(tmp_dir.join("same-title.md").exists());
        // Second file should have uuid prefix appended
        let entries: Vec<_> = fs::read_dir(&tmp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 2);

        // Cleanup
        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
