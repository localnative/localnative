use rusqlite::Connection;

pub fn check_table_exist(conn: &Connection, table_name: &str) -> bool {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name= :table_name")
        .unwrap();
    match stmt.query_row_named(&[(":table_name", &table_name)], |row| {
        Ok(OneString { s: row.get(0)? })
    }) {
        Ok(rs) => {
            if rs.s == table_name {
                true
            } else {
                panic!(
                    "check_table_exist returned table name not match: {}",
                    table_name
                )
            }
        }
        Err(_) => false,
    }
}

pub struct OneString {
    pub s: String,
}
