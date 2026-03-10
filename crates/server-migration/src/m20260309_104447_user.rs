use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use nihility_secret::generate_secret;
use nihility_server_entity::user;
use sea_orm_migration::sea_orm::{ActiveModelTrait, Set};
use sea_orm_migration::{prelude::*, schema::*};
use tracing::info;
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_uuid(User::Id))
                    .col(string_uniq(User::Name))
                    .col(string(User::Password))
                    .to_owned(),
            )
            .await?;

        let conn = manager.get_connection();

        let password = generate_secret(16);
        info!("Generated Password is {}", password);

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(&password.into_bytes(), &salt)
            .expect("build password hash fail")
            .to_string();

        user::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set("admin".to_string()),
            password: Set(password_hash),
        }
        .insert(conn)
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Name,
    Password,
}
