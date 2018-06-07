pub const JSON: &'static str = "application/json";

macro_rules! mk_sql {
    ($t:expr, $($x:expr),*) => {
        format!($t, $($x),*)
    }
}

pub fn select(limit: u32, offset: u32) -> String {
    format!(
        r#"SELECT title, url, tags, description, comments, annotations, created_at
           FROM note order by created_at desc limit {} offset {}"#,
        limit, offset
    )
}

pub fn search(limit: u32, offset: u32, query: &str) -> String {
    format!(
        r#"SELECT title, url, tags, description, comments, annotations, created_at FROM note
           where title like '%{}%'
            order by created_at desc limit {} offset {}"#,
        query, limit, offset
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let x = mk_sql!["{}", "good"];
        println!("{}", x);
    }
}
