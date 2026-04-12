use crate::StoreError;
use nihility_store_entity::prelude::Scene;
use nihility_store_entity::scene;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};
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
