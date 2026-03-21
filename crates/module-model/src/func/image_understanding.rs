use crate::config::ModelCapability;
use crate::error::Result;
use crate::provider::{BoxStream, ProviderFactory};
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

/// 图片理解流式请求参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImageUnderstandingStreamParam {
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
                    let provider = ProviderFactory::create(&model.provider)?;
                    provider
                        .image_understanding(&param.image_url, &param.prompt)
                        .await
                },
            )
            .await
    }

    /// 图片理解流式响应
    pub async fn image_understanding_stream(
        &self,
        param: &ImageUnderstandingStreamParam,
    ) -> Result<BoxStream<String>> {
        self.pool
            .invoke(
                ModelCapability::ImageUnderstanding,
                move |model| async move {
                    let provider = ProviderFactory::create(&model.provider)?;
                    provider
                        .image_understanding_stream(&param.image_url, &param.prompt)
                        .await
                },
            )
            .await
    }
}
