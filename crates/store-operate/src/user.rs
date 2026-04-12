use crate::StoreError;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use nihility_store_entity::prelude::User;
use nihility_store_entity::user;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};

pub async fn check_user_password(
    db: &DbConn,
    username: String,
    password: String,
) -> Result<bool, StoreError> {
    match User::find()
        .filter(user::Column::Name.eq(username.clone()))
        .one(db)
        .await?
    {
        None => Err(StoreError::NotFound(format!("user: {}", username))),
        Some(user) => {
            let parsed_hash =
                PasswordHash::new(&user.password).map_err(StoreError::PasswordHash)?;
            Ok(Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok())
        }
    }
}
