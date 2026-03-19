use crate::error::*;
use chrono::Utc;
use nihility_server_entity::html_pages;
use nihility_server_entity::prelude::HtmlPages;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, PaginatorTrait, QueryFilter, Set};
use uuid::Uuid;

pub struct HtmlPageService;

impl HtmlPageService {
    pub async fn find_html_by_path(db: &DbConn, path: &str) -> Result<String> {
        match HtmlPages::find()
            .filter(html_pages::Column::Path.eq(path))
            .one(db)
            .await?
        {
            None => Err(NihilityServerError::NotFound(format!(
                "html page: {}",
                path
            ))),
            Some(record) => Ok(record.html),
        }
    }

    /// 获取所有 HTML 页面列表
    pub async fn list_all(db: &DbConn) -> Result<Vec<html_pages::Model>> {
        let pages = HtmlPages::find().all(db).await?;
        Ok(pages)
    }

    /// 获取 HTML 页面总数
    pub async fn count_all(db: &DbConn) -> Result<u64> {
        let count = HtmlPages::find().count(db).await?;
        Ok(count)
    }

    /// 根据 ID 查找 HTML 页面
    pub async fn find_by_id(db: &DbConn, id: &Uuid) -> Result<html_pages::Model> {
        match HtmlPages::find_by_id(*id).one(db).await? {
            None => Err(NihilityServerError::NotFound(format!(
                "html page with id: {}",
                id
            ))),
            Some(record) => Ok(record),
        }
    }

    /// 创建新的 HTML 页面
    pub async fn create(db: &DbConn, path: String, html: String) -> Result<html_pages::Model> {
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

    /// 更新现有 HTML 页面
    pub async fn update(
        db: &DbConn,
        id: &Uuid,
        path: String,
        html: String,
    ) -> Result<html_pages::Model> {
        // 先查找页面是否存在
        let existing_page = Self::find_by_id(db, id).await?;

        let now = Utc::now().fixed_offset();
        let mut active_page: html_pages::ActiveModel = existing_page.into();
        active_page.path = Set(path);
        active_page.html = Set(html);
        active_page.update_at = Set(now);

        let updated_page = active_page.update(db).await?;
        Ok(updated_page)
    }

    /// 删除 HTML 页面
    pub async fn delete(db: &DbConn, id: &Uuid) -> Result<()> {
        // 先查找页面是否存在
        let existing_page = Self::find_by_id(db, id).await?;

        let active_page: html_pages::ActiveModel = existing_page.into();
        active_page.delete(db).await?;

        Ok(())
    }
}
