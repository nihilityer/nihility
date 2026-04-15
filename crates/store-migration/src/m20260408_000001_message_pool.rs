use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::sea_orm::DbBackend;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Scene::Table)
                    .if_not_exists()
                    .col(pk_uuid(Scene::Id))
                    .col(string_uniq(Scene::Name))
                    .col(uuid_null(Scene::ParentId))
                    .col(json_binary(Scene::Metadata))
                    .col(timestamp_with_time_zone(Scene::CreatedAt))
                    .col(timestamp_with_time_zone(Scene::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_scene_parent_id")
                    .table(Scene::Table)
                    .col(Scene::ParentId)
                    .to_owned(),
            )
            .await?;

        if let DbBackend::Postgres = manager.get_database_backend() {
            manager
                .create_type(
                    Type::create()
                        .as_enum(MsgType::Table)
                        .values([
                            MsgType::Text,
                            MsgType::Audio,
                            MsgType::Image,
                            MsgType::Video,
                        ])
                        .to_owned(),
                )
                .await?;
        }

        manager
            .create_table(
                Table::create()
                    .table(Message::Table)
                    .if_not_exists()
                    .col(pk_uuid(Message::Id))
                    .col(uuid(Message::SceneId))
                    .col(enumeration(
                        Message::MsgType,
                        MsgType::Table,
                        [
                            MsgType::Text,
                            MsgType::Audio,
                            MsgType::Image,
                            MsgType::Video,
                        ],
                    ))
                    .col(json_binary(Message::Content))
                    .col(json_binary(Message::Metadata))
                    .col(boolean(Message::IsProcessed).default(Expr::value(false)))
                    .col(timestamp_with_time_zone(Message::CreatedAt))
                    .col(timestamp_with_time_zone(Scene::UpdatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message_scene_id")
                            .from(Message::Table, Message::SceneId)
                            .to(Scene::Table, Scene::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_message_scene_id")
                    .table(Message::Table)
                    .col(Message::SceneId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Scene {
    Table,
    Id,
    Name,
    ParentId,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum MsgType {
    Table,
    Text,
    Audio,
    Image,
    Video,
}

#[derive(DeriveIden)]
enum Message {
    Table,
    Id,
    SceneId,
    MsgType,
    Content,
    Metadata,
    IsProcessed,
    CreatedAt,
}
