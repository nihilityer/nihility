use sea_orm_migration::prelude::prelude::serde_json::json;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ActiveModelTrait, Set};
use tracing::info;
use uuid::Uuid;

const BASE_SCENE_DESC: &str = "助手需要提供服务的根场景，所有场景需要注意：除非有特殊需求，所有输入的回复应该发送到与消息来源相同的场景";
const VIRTUAL_SCENE_DESC: &str =
    "助手在虚拟世界中提供服务的场景，收集的输入来源更广泛，与创建者的诉求相关性较低";
const REAL_SCENE_DESC: &str = "助手通过现实世界中具体设备终端提供服务的场景，收集的输入与创建者诉求相关性较高，但无用消息或不需要处理的消息较多";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let base_id = nihility_store_entity::scene::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set("base".to_string()),
            metadata: Set(json!({
                "description": BASE_SCENE_DESC,
            })),
            ..Default::default()
        }
        .insert(db)
        .await?
        .id;
        info!("created base scene: {}", base_id);
        let virtual_id = nihility_store_entity::scene::ActiveModel {
            id: Set(Uuid::new_v4()),
            parent_id: Set(Some(base_id)),
            name: Set("virtual".to_string()),
            metadata: Set(json!({
                "description": VIRTUAL_SCENE_DESC,
            })),
            ..Default::default()
        }
        .insert(db)
        .await?
        .id;
        info!("created virtual scene: {}", virtual_id);
        let real_id = nihility_store_entity::scene::ActiveModel {
            id: Set(Uuid::new_v4()),
            parent_id: Set(Some(base_id)),
            name: Set("real".to_string()),
            metadata: Set(json!({
                "description": REAL_SCENE_DESC,
            })),
            ..Default::default()
        }
        .insert(db)
        .await?
        .id;
        info!("created real scene: {}", real_id);
        Ok(())
    }
}
