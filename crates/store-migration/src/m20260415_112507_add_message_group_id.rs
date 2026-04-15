use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::uuid;
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Message::Table)
                    .add_column_if_not_exists(uuid(Message::GroupId).default(Uuid::new_v4()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_message_group_id")
                    .table(Message::Table)
                    .col(Message::GroupId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Message {
    Table,
    GroupId,
}
