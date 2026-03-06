use crate::error::*;
use nihility_server_entity::html_pages;
use nihility_server_entity::prelude::HtmlPages;
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};

pub struct HtmlPage;

impl HtmlPage {
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
}
