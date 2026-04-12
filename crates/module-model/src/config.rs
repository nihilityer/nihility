use crate::provider::openai_api::OpenAiApiConfig;
use crate::provider::sense_voice::{SenseVoiceConfig, SenseVoiceLanguage, SenseVoiceTextNorm};
use serde::{Deserialize, Serialize};

/// 模型能力类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
pub enum ModelCapability {
    /// 文本补全能力
    TextCompletion,
    /// 图片理解能力
    ImageUnderstanding,
    /// 语音识别能力
    SpeechRecognition,
}

/// 负载均衡策略类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub enum LoadBalanceType {
    /// 加权轮询策略
    #[default]
    WeightedRoundRobin,
    /// 加权随机策略
    WeightedRandom,
}

/// 负载均衡器配置
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LoadBalanceConfig {
    /// 负载均衡策略类型
    pub strategy: LoadBalanceType,
    /// 失败时权重下降比例，取值范围 0.0-1.0
    /// 每次失败后，权重 = 权重 * failure_decrease_ratio
    #[serde(default = "default_failure_decrease_ratio")]
    pub failure_decrease_ratio: f32,
    /// 权重最小值，防止权重过低
    #[serde(default = "default_min_weight")]
    pub min_weight: u32,
    /// 恢复时权重增加量
    #[serde(default = "default_recovery_increase")]
    pub recovery_increase: u32,
    /// 禁用模型前的最大连续失败次数
    #[serde(default = "default_max_failures")]
    pub max_failures_before_disable: u32,
}

fn default_failure_decrease_ratio() -> f32 {
    0.5
}

fn default_min_weight() -> u32 {
    10
}

fn default_recovery_increase() -> u32 {
    10
}

fn default_max_failures() -> u32 {
    5
}

impl Default for LoadBalanceConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalanceType::WeightedRoundRobin,
            failure_decrease_ratio: 0.5,
            min_weight: 10,
            recovery_increase: 10,
            max_failures_before_disable: 5,
        }
    }
}

/// 单个模型条目配置
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ModelEntry {
    /// 模型名称标识
    pub name: String,
    /// 模型提供者配置
    pub provider: ProviderType,
    /// 模型权重，用于负载均衡
    #[serde(default = "default_weight")]
    pub weight: u32,
    /// 模型支持的能力列表
    pub capabilities: Vec<ModelCapability>,
}

/// 模型提供者类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum ProviderType {
    /// OpenAI API 提供者
    OpenAI(OpenAiApiConfig),
    /// 嵌入式模型提供者
    Embed(EmbedProvider),
}

/// 嵌入式模型提供者配置
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum EmbedProvider {
    /// SenseVoice 语音识别模型
    SenseVoice(SenseVoiceConfig),
}

fn default_weight() -> u32 {
    100
}

/// 模型模块主配置结构
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ModelConfig {
    /// 模型列表
    pub models: Vec<ModelEntry>,
    /// 负载均衡配置
    #[serde(default)]
    pub load_balance: LoadBalanceConfig,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            models: vec![
                ModelEntry {
                    name: "embed-sense-voice".to_string(),
                    provider: ProviderType::Embed(EmbedProvider::SenseVoice(SenseVoiceConfig {
                        language: SenseVoiceLanguage::Zh,
                        text_norm: SenseVoiceTextNorm::WithItn,
                        remove_status_token: false,
                        ..Default::default()
                    })),
                    weight: 1,
                    capabilities: vec![ModelCapability::SpeechRecognition],
                },
                ModelEntry {
                    name: "llama.cpp".to_string(),
                    provider: ProviderType::OpenAI(OpenAiApiConfig {
                        base_url: "http://127.0.0.1:8000/v1".to_string(),
                        api_key: "test".to_string(),
                        model: "Qwen3.5-9B".to_string(),
                    }),
                    weight: 1,
                    capabilities: vec![ModelCapability::TextCompletion],
                },
            ],
            load_balance: Default::default(),
        }
    }
}
