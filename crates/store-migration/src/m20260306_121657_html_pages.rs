use nihility_store_entity::html_pages;
use sea_orm_migration::sea_orm::{ActiveModelTrait, Set};
use sea_orm_migration::{prelude::*, schema::*};
use uuid::Uuid;

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
                    .col(pk_uuid(HtmlPages::Id).default(Uuid::new_v4()))
                    .col(string_uniq(HtmlPages::Path))
                    .col(text(HtmlPages::Html))
                    .col(
                        timestamp_with_time_zone(HtmlPages::UpdateAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let conn = manager.get_connection();

        html_pages::ActiveModel {
            id: Set(Uuid::new_v4()),
            path: Set("test".to_string()),
            html: Set(include_str!("../html/test.html").to_string()),
            update_at: Default::default(),
        }
        .insert(conn)
        .await?;

        Ok(())
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
