use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Notes::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Notes::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Notes::Title).text().not_null())
                    .col(ColumnDef::new(Notes::Url).text().not_null())
                    .col(ColumnDef::new(Notes::Description).text().not_null())
                    .col(ColumnDef::new(Notes::Tags).text().not_null())
                    .col(ColumnDef::new(Notes::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Notes::IsPublic).boolean().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Comments::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Comments::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Comments::Value).text().not_null())
                    .col(ColumnDef::new(Comments::CreatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(NotesComments::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(NotesComments::NoteId).uuid().not_null())
                    .col(ColumnDef::new(NotesComments::CommentId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk-notes_comments")
                            .col(NotesComments::NoteId)
                            .col(NotesComments::CommentId)
                            .primary(),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk-notes_comments-note_id")
                            .from_tbl(NotesComments::Table)
                            .from_col(NotesComments::NoteId)
                            .to_tbl(Notes::Table)
                            .to_col(Notes::Id),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("fk-notes_comments-comments")
                            .from_tbl(NotesComments::Table)
                            .from_col(NotesComments::CommentId)
                            .to_tbl(Comments::Table)
                            .to_col(Comments::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Notes {
    Table,
    Id,
    Title,
    Url,
    Description,
    CreatedAt,
    IsPublic,
    Tags,
}

#[derive(DeriveIden)]
enum Comments {
    Table,
    Id,
    Value,
    CreatedAt,
}

#[derive(DeriveIden)]
enum NotesComments {
    Table,
    NoteId,
    CommentId,
}
