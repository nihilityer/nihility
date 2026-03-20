pub mod adjust_weight;
pub mod image_understanding;
pub mod speech_recognition;
pub mod text_completion;

use nihility_module::{Callable, FunctionMetadata, Module};
use schemars::schema_for;
use serde_json::Value;

use crate::func::adjust_weight::AdjustWeightParam;
use crate::func::image_understanding::ImageUnderstandingParam;
use crate::func::speech_recognition::SpeechRecognitionParam;
use crate::func::text_completion::TextCompletionParam;
use crate::ModelModule;

#[async_trait::async_trait]
impl Callable for ModelModule {
    async fn call(&self, func_name: &str, param: Value) -> anyhow::Result<Value> {
        match func_name {
            "text_completion" => {
                let param = serde_json::from_value::<TextCompletionParam>(param)?;
                let result = self.text_completion(&param).await?;
                Ok(serde_json::to_value(result)?)
            }
            "image_understanding" => {
                let param = serde_json::from_value::<ImageUnderstandingParam>(param)?;
                let result = self.image_understanding(&param).await?;
                Ok(serde_json::to_value(result)?)
            }
            "speech_recognition" => {
                let param = serde_json::from_value::<SpeechRecognitionParam>(param)?;
                let result = self.speech_recognition(&param).await?;
                Ok(serde_json::to_value(result)?)
            }
            _ => Err(anyhow::anyhow!("Unsupported func_name: {}", func_name)),
        }
    }

    async fn call_mut(&mut self, func_name: &str, param: Value) -> anyhow::Result<Value> {
        match func_name {
            "adjust_weight" => {
                let param = serde_json::from_value::<AdjustWeightParam>(param)?;
                self.adjust_weight(&param).await;
                Ok(serde_json::json!({ "success": true }))
            }
            _ => Err(anyhow::anyhow!("Unsupported func_name: {}", func_name)),
        }
    }
}

impl Module for ModelModule {
    fn description(&self) -> &str {
        "AI 模型调用模块，支持文本补全、图片理解、语音识别等接口"
    }

    fn no_perm_func(&self) -> Vec<FunctionMetadata> {
        vec![
            FunctionMetadata {
                name: "text_completion".to_string(),
                desc: "文本补全".to_string(),
                tags: vec![],
                params: serde_json::to_value(schemars::schema_for!(TextCompletionParam))
                    .expect("model module func text_completion build param"),
            },
            FunctionMetadata {
                name: "image_understanding".to_string(),
                desc: "图片理解".to_string(),
                tags: vec![],
                params: serde_json::to_value(schemars::schema_for!(ImageUnderstandingParam))
                    .expect("model module func image_understanding build param"),
            },
            FunctionMetadata {
                name: "speech_recognition".to_string(),
                desc: "语音识别".to_string(),
                tags: vec![],
                params: serde_json::to_value(schemars::schema_for!(SpeechRecognitionParam))
                    .expect("model module func speech_recognition build param"),
            },
        ]
    }

    fn perm_func(&mut self) -> Vec<FunctionMetadata> {
        vec![FunctionMetadata {
            name: "adjust_weight".to_string(),
            desc: "手动调整模型权重".to_string(),
            tags: vec![],
            params: serde_json::to_value(schema_for!(AdjustWeightParam))
                .expect("model module func adjust_weight build param"),
        }]
    }
}
