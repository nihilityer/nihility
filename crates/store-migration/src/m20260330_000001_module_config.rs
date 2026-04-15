use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ModuleConfig::Table)
                    .if_not_exists()
                    .col(pk_uuid(ModuleConfig::Id))
                    .col(string_uniq(ModuleConfig::ModuleName))
                    .col(json_binary(ModuleConfig::ConfigValue))
                    .col(json_binary(ModuleConfig::JsonSchema))
                    .col(
                        timestamp_with_time_zone(ModuleConfig::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(ModuleConfig::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ModuleConfig {
    Table,
    Id,
    ModuleName,
    ConfigValue,
    JsonSchema,
    CreatedAt,
    UpdatedAt,
}
