use crate::error::*;
use ndarray::{Array2, ArrayD};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::info;

const MODEL_REPO: &str = "nihilityer/nihility";
const MODEL_NAME: &str = "silero_vad.onnx";

/// Silero语音活动检测模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SileroConfig {
    pub sample_rate: usize,
    pub chunk_size: usize,
    pub model_path: String,
}

/// Silero语音活动检测模型
pub struct Silero {
    pub(crate) config: SileroConfig,
    state: ArrayD<f32>,
    session: Session,
}

impl Silero {
    /// 使用配置初始化Silero语音活动检测模型
    pub async fn init(mut config: SileroConfig) -> Result<Self> {
        let model_dir =
            Path::new(&config.model_path)
                .parent()
                .ok_or(VoiceActivityDetectionError::Init(format!(
                    "Invalid model path: {:?}",
                    config.model_path
                )))?;
        if !fs::exists(&config.model_path)? {
            info!("Download Silero model to directory: {:?}", model_dir);
            fs::create_dir_all(model_dir)?;
            let hf_api = hf_hub::api::tokio::ApiBuilder::from_env()
                .with_cache_dir(model_dir.to_path_buf())
                .build()?;
            hf_api
                .model(MODEL_REPO.to_string())
                .download(MODEL_NAME)
                .await?;
        }
        let chunk_size = match &config.sample_rate {
            8000 => 256,
            16000 => 512,
            _ => {
                return Err(VoiceActivityDetectionError::Init(format!(
                    "Sample rate {} not supported",
                    config.sample_rate
                )));
            }
        };
        if config.chunk_size != chunk_size {
            config.chunk_size = chunk_size;
        }
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| VoiceActivityDetectionError::OrtSessionBuilder(e.to_string()))?
            .with_inter_threads(1)
            .map_err(|e| VoiceActivityDetectionError::OrtSessionBuilder(e.to_string()))?
            .with_intra_threads(1)
            .map_err(|e| VoiceActivityDetectionError::OrtSessionBuilder(e.to_string()))?
            .commit_from_file(&config.model_path)?;
        Ok(Self {
            config,
            state: ndarray::Array3::<f32>::zeros((2, 1, 128)).into_dyn(),
            session,
        })
    }

    /// 预测当前语音数据块语音活动概率
    pub fn predict(&mut self, samples: &[f32]) -> Result<f32> {
        if samples.len() != self.config.chunk_size {
            return Err(VoiceActivityDetectionError::Unknown(format!(
                "samples len must be {}",
                self.config.chunk_size
            )));
        }
        let input = Array2::from_shape_vec((1, self.config.chunk_size), samples.to_vec())?;
        let sample_rate = ndarray::arr0::<i64>(self.config.sample_rate as i64);

        let state_taken = std::mem::take(&mut self.state);

        // ort::inputs! macro no longer supports ArrayView, use Tensor::from_array instead
        let inputs = ort::inputs![
            Tensor::from_array(input.to_owned())?,
            Tensor::from_array(state_taken.to_owned())?,
            Tensor::from_array(sample_rate.to_owned())?,
        ];

        let outputs = self.session.run(inputs)?;

        // 递归地更新状态。
        self.state = outputs
            .get("stateN")
            .expect("model error, output do not have stateN field")
            .try_extract_array::<f32>()?
            .to_owned();

        // 获取语音发生的概率。
        let output = outputs
            .get("output")
            .expect("model error, output do not have output field")
            .try_extract_array::<f32>()?;
        Ok(output[[0, 0]])
    }
}

impl Default for SileroConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            chunk_size: 512,
            model_path: "model/silero_vad.onnx".to_string(),
        }
    }
}
