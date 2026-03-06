use sea_orm_migration::prelude::*;
pub use sea_orm_migration::MigratorTrait;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260306_121657_html_pages::Migration)]
    }
}
mod m20260306_121657_html_pages;
