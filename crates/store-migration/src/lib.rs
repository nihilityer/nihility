use sea_orm_migration::prelude::*;
pub use sea_orm_migration::MigratorTrait;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260306_121657_html_pages::Migration),
            Box::new(m20260309_104447_user::Migration),
            Box::new(m20260330_000001_module_config::Migration),
            Box::new(m20260408_000001_message_pool::Migration),
            Box::new(m20260415_112507_add_message_group_id::Migration),
            Box::new(m20260416_123542_base_scene::Migration),
        ]
    }
}
mod m20260306_121657_html_pages;
mod m20260309_104447_user;
mod m20260330_000001_module_config;
mod m20260408_000001_message_pool;
mod m20260415_112507_add_message_group_id;
mod m20260416_123542_base_scene;
