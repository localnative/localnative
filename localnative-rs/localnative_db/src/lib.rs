use futures_util::future::try_join_all;
use old_entity::note;
use sea_orm::{
    prelude::Uuid, sea_query::TableDropStatement, ActiveModelTrait, ActiveValue::Set, ColumnTrait,
    CursorTrait, EntityName, EntityTrait, ModelTrait, QueryFilter, QueryOrder, QuerySelect,
    QueryTrait, Related,
};
use sea_orm_migration::MigratorTrait;
mod entity;
mod migration;
mod old_entity;

pub struct Migrator {
    conn: sea_orm::DatabaseConnection,
}

impl Migrator {
    pub fn new(conn: sea_orm::DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        let manager = sea_orm_migration::SchemaManager::new(&self.conn);

        if manager.has_column("note", "rowid").await? {
            sea_orm_migration::MigrationTrait::up(&migration::InitMigration, &manager).await?;

            // 老的note，需要查询，并且迁移到新的note
            let notes = old_entity::note::Entity::find()
                .select_only()
                .columns([
                    old_entity::note::Column::Title,
                    old_entity::note::Column::Url,
                    old_entity::note::Column::Tags,
                    old_entity::note::Column::Description,
                    old_entity::note::Column::Comments,
                    old_entity::note::Column::CreatedAt,
                    old_entity::note::Column::IsPublic,
                ])
                .into_partial_model::<note::PartialNote>()
                .all(&self.conn)
                .await?;

            for old_note in notes {
                println!("will migrate note: {:?}", old_note);
                let note = entity::notes::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    title: Set(old_note.title),
                    url: Set(old_note.url),
                    description: Set(old_note.description),
                    created_at: Set(chrono::NaiveDateTime::parse_from_str(
                        old_note.created_at.as_str(),
                        "%F %T:%f UTC",
                    )?
                    .and_utc()),
                    is_public: Set(old_note.is_public),
                    tags: Set(old_note.tags.into()),
                }
                .insert(&self.conn)
                .await?;

                for comment in
                    old_note
                        .comments
                        .split(",")
                        .map(|comment| entity::comments::ActiveModel {
                            id: Set(Uuid::new_v4()),
                            value: Set(comment.to_string()),
                            created_at: Set(note.created_at),
                        })
                {
                    let comment_model = comment.insert(&self.conn).await?;
                    entity::notes_comments::ActiveModel {
                        note_id: Set(note.id),
                        comment_id: Set(comment_model.id),
                    }
                    .insert(&self.conn)
                    .await?;
                }
            }

            // 删除掉老的表
            if manager
                .has_table(old_entity::prelude::Note.table_name())
                .await?
            {
                let mut stmt = TableDropStatement::new();
                stmt.table(old_entity::prelude::Note);

                manager.drop_table(stmt).await?;
            }

            if manager
                .has_table(old_entity::prelude::Meta.table_name())
                .await?
            {
                let mut stmt = TableDropStatement::new();
                stmt.table(old_entity::prelude::Meta);

                manager.drop_table(stmt).await?;
            }
        } else {
            // 说明并没有任何的note存在，这是一台新的机器，直接创建note的表就可以了。
            migration::Migrator::refresh(&self.conn).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_db() -> anyhow::Result<()> {
    let conn = sea_orm::Database::connect(
        "sqlite:/Users/cupnfish/Documents/GitHub/localnative/localnative-rs/localnative.sqlite3",
    )
    .await?;
    let migrator = Migrator { conn };
    migrator.init().await?;

    for i in 0..100 {
        let note = NoteInsert::new(
            format!("test{}", i),
            "https://www.google.com".to_string(),
            "test".to_string(),
            vec!["test".to_string(), "test2".to_string(), "test3".to_string()],
        )
        .insert(&migrator.conn)
        .await?;

        for i in 0..10 {
            CommnetInsert::new(format!("test{}", i))
                .insert(&migrator.conn, note.id)
                .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[tokio::test]
async fn test_init_db() -> anyhow::Result<()> {
    let conn = sea_orm::Database::connect(
        "sqlite:/Users/cupnfish/Documents/GitHub/localnative/localnative-rs/init.sqlite3",
    )
    .await?;
    let migrator = Migrator { conn };
    migrator.init().await?;

    Ok(())
}

#[cfg(test)]
#[test]
fn test_parse_timestamp() {
    let time =
        chrono::NaiveDateTime::parse_from_str("2022-06-06 10:25:30:818661000 UTC", "%F %T:%f UTC")
            .unwrap();

    println!("{:?}", time);
}

pub struct NoteInsert {
    pub title: String,
    pub url: String,
    pub description: String,
    pub created_at: sea_orm::prelude::ChronoDateTimeUtc,
    pub is_public: bool,
    pub tags: Vec<String>,
}

impl NoteInsert {
    pub fn new(title: String, url: String, description: String, tags: Vec<String>) -> Self {
        Self {
            title,
            url,
            description,
            created_at: chrono::Utc::now(),
            is_public: true,
            tags,
        }
    }

    pub async fn insert(
        self,
        conn: &sea_orm::DatabaseConnection,
    ) -> Result<entity::notes::Model, sea_orm::DbErr> {
        entity::notes::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(self.title),
            url: Set(self.url),
            description: Set(self.description),
            created_at: Set(self.created_at),
            is_public: Set(self.is_public),
            tags: Set(self.tags.into()),
        }
        .insert(conn)
        .await
    }
}

pub struct Comment {
    pub id: Uuid,
    pub value: String,
    pub created_at: sea_orm::prelude::ChronoDateTimeUtc,
}

pub struct CommnetInsert {
    pub value: String,
    pub created_at: sea_orm::prelude::ChronoDateTimeUtc,
}

impl CommnetInsert {
    pub fn new(comment: String) -> Self {
        Self {
            value: comment,
            created_at: chrono::Utc::now(),
        }
    }
    pub async fn insert(
        self,
        conn: &sea_orm::DatabaseConnection,
        note_id: Uuid,
    ) -> Result<entity::comments::Model, sea_orm::DbErr> {
        let comment = entity::comments::ActiveModel {
            id: Set(Uuid::new_v4()),
            value: Set(self.value),
            created_at: Set(self.created_at),
        }
        .insert(conn)
        .await?;

        entity::notes_comments::ActiveModel {
            note_id: Set(note_id),
            comment_id: Set(comment.id),
        }
        .insert(conn)
        .await?;

        Ok(comment)
    }
}

#[cfg(test)]
#[test]
fn test_join() {
    let a = entity::prelude::Notes::find().find_with_related(entity::prelude::Comments);
    println!("{:#?}", a);
}

pub struct QueryManager<C, V> {
    cursor_by: C,
    after: Option<V>,
    before: Option<V>,
    index: Index,
    filters: Vec<sea_query::Condition>,
}

enum Index {
    Last(u64),
    First(u64),
    Default,
}

impl<C, V> QueryManager<C, V>
where
    C: sea_orm::IntoIdentity,
    V: sea_orm::sea_query::IntoValueTuple,
{
    pub fn new(cursor_by: C) -> Self {
        Self {
            cursor_by,
            after: None,
            before: None,
            index: Index::Default,
            filters: vec![],
        }
    }
    pub fn after(mut self, after: V) -> Self {
        self.after.replace(after);
        self
    }

    pub fn before(mut self, before: V) -> Self {
        self.before.replace(before);
        self
    }

    pub fn last(mut self, last: u64) -> Self {
        self.index = Index::Last(last);
        self
    }

    pub fn first(mut self, first: u64) -> Self {
        self.index = Index::First(first);
        self
    }

    pub fn filter(mut self, condition: impl sea_query::IntoCondition) -> Self {
        self.filters.push(condition.into_condition());
        Self
    }
}

impl<V> QueryManager<entity::notes::Column, V>
where
    V: sea_orm::sea_query::IntoValueTuple,
{
    pub async fn query(
        self,
        conn: &sea_orm::DatabaseConnection,
    ) -> Result<Vec<entity::notes::Model>, sea_orm::DbErr> {
        self.cursor_statement().all(conn).await
    }

    fn cursor_statement(self) -> sea_orm::Cursor<sea_orm::SelectModel<entity::notes::Model>> {
        let select = self
            .filters
            .into_iter()
            .fold(entity::prelude::Notes::find(), |select, condition| {
                select.filter(condition)
            });

        let mut statement = select.cursor_by(self.cursor_by);

        match self.index {
            Index::Last(last) => statement.last(last),
            Index::First(first) => statement.first(first),
            Index::Default => &mut statement,
        };

        if let Some(after) = self.after {
            statement.after(after);
        }

        if let Some(before) = self.before {
            statement.before(before);
        }

        statement
    }
}

impl<V> QueryManager<entity::comments::Column, V>
where
    V: sea_orm::sea_query::IntoValueTuple,
{
    pub async fn query(
        self,
        conn: &sea_orm::DatabaseConnection,
        note: &entity::notes::Model,
    ) -> Result<Vec<entity::comments::Model>, sea_orm::DbErr> {
        self.statement(note).all(conn).await
    }

    fn statement(
        self,
        note: &entity::notes::Model,
    ) -> sea_orm::Cursor<sea_orm::SelectModel<entity::comments::Model>> {
        let select = self.filters.into_iter().fold(
            note.find_related(entity::prelude::Comments),
            |select, condition| select.filter(condition),
        );

        let mut statement = select.cursor_by(self.cursor_by);

        match self.index {
            Index::Last(last) => statement.last(last),
            Index::First(first) => statement.first(first),
            Index::Default => &mut statement,
        };

        if let Some(after) = self.after {
            statement.after(after);
        }

        if let Some(before) = self.before {
            statement.before(before);
        }

        statement
    }
}
