use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HtmlPages::Table)
                    .if_not_exists()
                    .col(pk_uuid(HtmlPages::Id))
                    .col(string_uniq(HtmlPages::Path))
                    .col(text(HtmlPages::Html))
                    .col(
                        timestamp_with_time_zone(HtmlPages::UpdateAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HtmlPages::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum HtmlPages {
    Table,
    Id,
    Path,
    Html,
    UpdateAt,
}
