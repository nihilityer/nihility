use crate::StoreError;
use chrono::Utc;
use nihility_store_entity::message;
use nihility_store_entity::prelude::Message;
pub use nihility_store_entity::sea_orm_active_enums::MsgType;
use sea_orm::prelude::Expr;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

pub async fn insert_message(
    db: &DbConn,
    scene_id: Uuid,
    msg_type: MsgType,
    content: serde_json::Value,
    metadata: serde_json::Value,
    group_id: Uuid,
    is_processed: bool,
) -> Result<message::Model, StoreError> {
    let now = Utc::now();
    let active_model = message::ActiveModel {
        id: Set(Uuid::new_v4()),
        scene_id: Set(scene_id),
        msg_type: Set(msg_type),
        content: Set(content),
        metadata: Set(metadata),
        is_processed: Set(is_processed),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        group_id: Set(group_id),
    };
    Ok(active_model.insert(db).await?)
}

pub async fn find_message_by_id(
    db: &DbConn,
    message_id: Uuid,
) -> Result<message::Model, StoreError> {
    Message::find_by_id(message_id)
        .one(db)
        .await?
        .ok_or_else(|| StoreError::NotFound(format!("message not found: {}", message_id)))
}

pub async fn find_message_by_group_id(
    db: &DbConn,
    group_id: Uuid,
) -> Result<Vec<message::Model>, StoreError> {
    Ok(Message::find()
        .filter(message::Column::GroupId.eq(group_id))
        .all(db)
        .await?)
}

pub async fn find_messages_by_scene_id(
    db: &DbConn,
    scene_id: Uuid,
) -> Result<Vec<message::Model>, StoreError> {
    let messages = Message::find()
        .filter(message::Column::SceneId.eq(scene_id))
        .all(db)
        .await?;
    Ok(messages)
}

pub async fn find_unprocessed_messages_by_scene_ids(
    db: &DbConn,
    scene_ids: &[Uuid],
) -> Result<Vec<message::Model>, StoreError> {
    let messages = Message::find()
        .filter(message::Column::SceneId.is_in(scene_ids.to_vec()))
        .filter(message::Column::IsProcessed.eq(false))
        .all(db)
        .await?;
    Ok(messages)
}

pub async fn update_message_processed(
    db: &DbConn,
    group_id: Uuid,
    is_processed: bool,
) -> Result<(), StoreError> {
    Message::update_many()
        .filter(message::Column::GroupId.eq(group_id))
        .col_expr(message::Column::IsProcessed, Expr::value(is_processed))
        .exec(db)
        .await?;
    Ok(())
}
