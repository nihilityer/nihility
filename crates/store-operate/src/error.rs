use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Invalid password hash: {0}")]
    PasswordHash(argon2::password_hash::Error),
}
