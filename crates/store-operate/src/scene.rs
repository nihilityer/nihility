use crate::StoreError;
use chrono::Utc;
use nihility_store_entity::prelude::Scene;
use nihility_store_entity::scene;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, ModelTrait, QueryFilter, Set};
use uuid::Uuid;

pub async fn find_scene_by_id(db: &DbConn, scene_id: Uuid) -> Result<scene::Model, StoreError> {
    Scene::find_by_id(scene_id)
        .one(db)
        .await?
        .ok_or_else(|| StoreError::NotFound(format!("scene not found: {}", scene_id)))
}

pub async fn find_scenes_by_parent_id(
    db: &DbConn,
    parent_id: Uuid,
) -> Result<Vec<scene::Model>, StoreError> {
    let scenes = Scene::find()
        .filter(scene::Column::ParentId.eq(parent_id))
        .all(db)
        .await?;
    Ok(scenes)
}

pub async fn insert_scene(
    db: &DbConn,
    name: String,
    parent_id: Option<Uuid>,
    metadata: serde_json::Value,
) -> Result<scene::Model, StoreError> {
    let now = Utc::now();
    let active_model = scene::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name),
        parent_id: Set(parent_id),
        metadata: Set(metadata),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    };
    Ok(active_model.insert(db).await?)
}

pub async fn update_scene(
    db: &DbConn,
    id: Uuid,
    name: Option<String>,
    parent_id: Option<Uuid>,
    metadata: Option<serde_json::Value>,
) -> Result<scene::Model, StoreError> {
    let existing = Scene::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| StoreError::NotFound(format!("scene not found: {}", id)))?;

    let now = Utc::now();
    let mut active_model: scene::ActiveModel = existing.into();
    if let Some(n) = name {
        active_model.name = Set(n);
    }
    if let Some(pid) = parent_id {
        active_model.parent_id = Set(Some(pid));
    }
    if let Some(m) = metadata {
        active_model.metadata = Set(m);
    }
    active_model.updated_at = Set(now.into());

    Ok(active_model.update(db).await?)
}

pub async fn delete_scene(db: &DbConn, id: Uuid) -> Result<(), StoreError> {
    let existing = Scene::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| StoreError::NotFound(format!("scene not found: {}", id)))?;
    scene::Model::delete(existing, db).await?;
    Ok(())
}

pub async fn find_all_scenes(db: &DbConn) -> Result<Vec<scene::Model>, StoreError> {
    let scenes = Scene::find().all(db).await?;
    Ok(scenes)
}
