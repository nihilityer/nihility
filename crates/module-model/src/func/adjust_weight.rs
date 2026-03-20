use crate::ModelModule;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 调整权重参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AdjustWeightParam {
    /// 需要调整的模型名称
    pub model_name: String,
    /// 调整后模型权重
    pub new_weight: u32,
}

impl ModelModule {
    /// 手动调整权重
    pub async fn adjust_weight(&self, param: &AdjustWeightParam) {
        self.pool
            .adjust_weight(&param.model_name, param.new_weight)
            .await
    }
}
