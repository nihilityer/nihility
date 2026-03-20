use crate::config::ModelCapability;
use crate::error::Result;
use crate::provider;
use crate::ModelModule;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 图片理解请求参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImageUnderstandingParam {
    /// 图片 URL 或 base64 编码的数据
    pub image_url: String,
    /// 提示词
    pub prompt: String,
}

impl ModelModule {
    /// 图片理解
    pub async fn image_understanding(
        &self,
        param: &ImageUnderstandingParam,
    ) -> Result<String> {
        self.pool
            .invoke(
                ModelCapability::ImageUnderstanding,
                move |model| async move {
                    let provider = provider::ProviderFactory::create(&model.provider)?;
                    provider
                        .image_understanding(&param.image_url, &param.prompt)
                        .await
                },
            )
            .await
    }
}
