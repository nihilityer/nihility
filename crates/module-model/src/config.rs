use crate::provider::openai_api::OpenAiApiConfig;
use crate::provider::sense_voice::SenseVoiceConfig;
use serde::{Deserialize, Serialize};

/// 模型能力类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
pub enum ModelCapability {
    TextCompletion,
    ImageUnderstanding,
    SpeechRecognition,
}

/// 负载均衡策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub enum LoadBalanceType {
    #[default]
    WeightedRoundRobin,
    WeightedRandom,
}

/// 负载均衡配置
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LoadBalanceConfig {
    pub strategy: LoadBalanceType,
    #[serde(default = "default_failure_decrease_ratio")]
    pub failure_decrease_ratio: f32,
    #[serde(default = "default_min_weight")]
    pub min_weight: u32,
    #[serde(default = "default_recovery_increase")]
    pub recovery_increase: u32,
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

/// 单个模型配置
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ModelEntry {
    pub name: String,
    pub provider: ProviderType,
    #[serde(default = "default_weight")]
    pub weight: u32,
    pub capabilities: Vec<ModelCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum ProviderType {
    OpenAI(OpenAiApiConfig),
    Embed(EmbedProvider),
}

/// 嵌入提供者配置
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum EmbedProvider {
    SenseVoice(SenseVoiceConfig),
}

fn default_weight() -> u32 {
    100
}

/// 主配置结构 - 全局模型池
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct ModelConfig {
    pub models: Vec<ModelEntry>,
    #[serde(default)]
    pub load_balance: LoadBalanceConfig,
}
