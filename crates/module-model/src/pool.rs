use crate::config::{LoadBalanceConfig, LoadBalanceType, ModelCapability, ModelEntry};
use crate::error::{ModelError, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use tokio::sync::RwLock;

/// 模型运行时状态
pub struct WeightedModelState {
    pub entry: ModelEntry,
    pub current_weight: AtomicU32,
    pub consecutive_failures: AtomicU32,
}

impl WeightedModelState {
    fn new(entry: ModelEntry) -> Self {
        Self {
            current_weight: AtomicU32::new(entry.weight),
            consecutive_failures: AtomicU32::new(0),
            entry,
        }
    }

    fn weight(&self) -> u32 {
        self.current_weight.load(Ordering::SeqCst)
    }
}

/// 全局模型池
pub struct ModelPool {
    /// 模型状态列表
    models: RwLock<Vec<WeightedModelState>>,
    /// 按能力类型索引 model name
    capability_index: RwLock<HashMap<ModelCapability, Vec<String>>>,
    /// 当前 round-robin 索引
    round_robin_counter: AtomicUsize,
    /// 配置
    config: LoadBalanceConfig,
}

impl ModelPool {
    /// 从配置创建模型池
    pub fn new(config: &crate::config::ModelConfig) -> Self {
        let models: Vec<WeightedModelState> = config
            .models
            .iter()
            .map(|entry| WeightedModelState::new(entry.clone()))
            .collect();

        let mut capability_index: HashMap<ModelCapability, Vec<String>> = HashMap::new();
        for entry in &config.models {
            for cap in &entry.capabilities {
                capability_index
                    .entry(*cap)
                    .or_default()
                    .push(entry.name.clone());
            }
        }

        Self {
            models: RwLock::new(models),
            capability_index: RwLock::new(capability_index),
            round_robin_counter: AtomicUsize::new(0),
            config: config.load_balance.clone(),
        }
    }

    /// 根据能力类型获取可用模型（权重 > 0）
    pub async fn get_available_models(&self, capability: ModelCapability) -> Vec<ModelEntry> {
        let index = self.capability_index.read().await;
        let names = index.get(&capability).cloned().unwrap_or_default();

        let models = self.models.read().await;
        names
            .into_iter()
            .filter_map(|name| {
                models
                    .iter()
                    .find(|m| m.entry.name == name && m.weight() > 0)
                    .map(|m| m.entry.clone())
            })
            .collect()
    }

    /// 报告失败 - 降低权重
    pub async fn report_failure(&self, model_name: &str) {
        let models = self.models.read().await;
        if let Some(model) = models.iter().find(|m| m.entry.name == model_name) {
            let failures = model.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;

            if failures >= self.config.max_failures_before_disable {
                model.current_weight.store(0, Ordering::SeqCst);
                return;
            }

            let current = model.current_weight.load(Ordering::SeqCst);
            let new_weight = ((current as f32 * self.config.failure_decrease_ratio).ceil() as u32)
                .max(self.config.min_weight);
            model.current_weight.store(new_weight, Ordering::SeqCst);
        }
    }

    /// 报告成功 - 重置失败计数，恢复权重
    pub async fn report_success(&self, model_name: &str) {
        let models = self.models.read().await;
        if let Some(model) = models.iter().find(|m| m.entry.name == model_name) {
            model.consecutive_failures.store(0, Ordering::SeqCst);

            let current = model.current_weight.load(Ordering::SeqCst);
            let new_weight = (current + self.config.recovery_increase).min(model.entry.weight);
            model.current_weight.store(new_weight, Ordering::SeqCst);
        }
    }

    /// 手动调整权重
    pub async fn adjust_weight(&self, model_name: &str, new_weight: u32) {
        let mut models = self.models.write().await;
        if let Some(model) = models.iter_mut().find(|m| m.entry.name == model_name) {
            model.current_weight.store(new_weight, Ordering::SeqCst);
        }
    }

    /// 获取所有模型名称
    pub async fn model_names(&self) -> Vec<String> {
        let models = self.models.read().await;
        models.iter().map(|m| m.entry.name.clone()).collect()
    }

    /// 获取模型当前权重
    pub async fn get_weight(&self, model_name: &str) -> Option<u32> {
        let models = self.models.read().await;
        models
            .iter()
            .find(|m| m.entry.name == model_name)
            .map(|m| m.weight())
    }

    /// 调用模型，自动选择 + 失败处理
    pub async fn invoke<F, Fut, R>(&self, capability: ModelCapability, mut f: F) -> Result<R>
    where
        F: FnMut(ModelEntry) -> Fut,
        Fut: Future<Output = Result<R>>,
    {
        let mut available_models = self.get_available_models(capability).await;
        if available_models.is_empty() {
            return Err(ModelError::NoAvailableModel);
        }

        let mut last_error = None;

        while !available_models.is_empty() {
            // 选择一个模型
            let selected = {
                let models = self.models.read().await;
                let indices: Vec<usize> = available_models
                    .iter()
                    .filter_map(|m| {
                        models
                            .iter()
                            .position(|sm| sm.entry.name == m.name && sm.weight() > 0)
                    })
                    .collect();

                if indices.is_empty() {
                    break;
                }

                let idx = match self.config.strategy {
                    LoadBalanceType::WeightedRoundRobin => {
                        let counter = self.round_robin_counter.fetch_add(1, Ordering::SeqCst);
                        counter % indices.len()
                    }
                    LoadBalanceType::WeightedRandom => {
                        let mut rng = rand::rngs::ThreadRng::default();
                        (<dyn rand::Rng>::next_u32(&mut rng) as usize) % indices.len()
                    }
                };

                indices.get(idx).copied()
            };

            let Some(idx) = selected else {
                break;
            };

            let model = {
                let models = self.models.read().await;
                models.get(idx).map(|m| m.entry.clone())
            };

            let Some(model) = model else {
                break;
            };

            // 调用
            match f(model.clone()).await {
                Ok(result) => {
                    self.report_success(&model.name).await;
                    return Ok(result);
                }
                Err(e) => {
                    self.report_failure(&model.name).await;
                    last_error = Some(e);
                    // 从待尝试列表中移除
                    available_models.retain(|m| m.name != model.name);
                }
            }
        }

        Err(last_error.unwrap_or(ModelError::AllModelsFailed))
    }
}
