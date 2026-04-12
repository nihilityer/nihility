use crate::StoreError;
use chrono::Utc;
use nihility_store_entity::html_pages;
use nihility_store_entity::prelude::HtmlPages;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

pub async fn find_html_by_path(db: &DbConn, path: &str) -> Result<String, StoreError> {
    match HtmlPages::find()
        .filter(html_pages::Column::Path.eq(path))
        .one(db)
        .await?
    {
        None => Err(StoreError::NotFound(format!("html page: {}", path))),
        Some(record) => Ok(record.html),
    }
}

pub async fn list_all(db: &DbConn) -> Result<Vec<html_pages::Model>, StoreError> {
    let pages = HtmlPages::find().all(db).await?;
    Ok(pages)
}

pub async fn find_by_id(db: &DbConn, id: &Uuid) -> Result<html_pages::Model, StoreError> {
    match HtmlPages::find_by_id(*id).one(db).await? {
        None => Err(StoreError::NotFound(format!("html page with id: {}", id))),
        Some(record) => Ok(record),
    }
}

pub async fn create(
    db: &DbConn,
    path: String,
    html: String,
) -> Result<html_pages::Model, StoreError> {
    let now = Utc::now().fixed_offset();
    let new_page = html_pages::ActiveModel {
        id: Set(Uuid::new_v4()),
        path: Set(path),
        html: Set(html),
        update_at: Set(now),
    };

    let page = new_page.insert(db).await?;
    Ok(page)
}

pub async fn update(
    db: &DbConn,
    id: &Uuid,
    path: String,
    html: String,
) -> Result<html_pages::Model, StoreError> {
    let existing_page = find_by_id(db, id).await?;

    let now = Utc::now().fixed_offset();
    let mut active_page: html_pages::ActiveModel = existing_page.into();
    active_page.path = Set(path);
    active_page.html = Set(html);
    active_page.update_at = Set(now);

    let updated_page = active_page.update(db).await?;
    Ok(updated_page)
}

pub async fn delete(db: &DbConn, id: &Uuid) -> Result<(), StoreError> {
    let existing_page = find_by_id(db, id).await?;

    let active_page: html_pages::ActiveModel = existing_page.into();
    active_page.delete(db).await?;

    Ok(())
}
