use crate::config::{LoadBalanceConfig, LoadBalanceType, ModelCapability, ModelEntry};
use crate::error::{ModelError, Result};
use crate::provider::{ModelProvider, ProviderFactory};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

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
    /// 模型状态列表（按索引访问）
    models: RwLock<Vec<WeightedModelState>>,
    /// 按能力类型索引 model 索引
    capability_index: RwLock<HashMap<ModelCapability, Vec<usize>>>,
    /// 已创建的 providers 缓存（内部 Mutex 保证线程安全）
    providers: Mutex<HashMap<String, Arc<Box<dyn ModelProvider>>>>,
    /// 当前 round-robin 索引
    round_robin_counter: AtomicUsize,
    /// 配置
    load_balance: LoadBalanceConfig,
}

impl ModelPool {
    /// 从配置创建模型池
    pub fn new(config: crate::config::ModelConfig) -> Self {
        let models: Vec<WeightedModelState> = config
            .models
            .iter()
            .map(|entry| WeightedModelState::new(entry.clone()))
            .collect();

        let mut capability_index: HashMap<ModelCapability, Vec<usize>> = HashMap::new();
        for (i, entry) in config.models.iter().enumerate() {
            for cap in &entry.capabilities {
                capability_index.entry(*cap).or_default().push(i);
            }
        }

        Self {
            models: RwLock::new(models),
            capability_index: RwLock::new(capability_index),
            round_robin_counter: AtomicUsize::new(0),
            load_balance: config.load_balance.clone(),
            providers: Mutex::new(HashMap::new()),
        }
    }

    /// 确保 provider 已创建，不存在则创建
    async fn ensure_provider(&self, idx: usize) -> Result<Arc<Box<dyn ModelProvider>>> {
        // 先获取 name 和 provider 配置
        let (name, provider_config) = {
            let models = self.models.read().await;
            let name = models[idx].entry.name.clone();
            let provider_config = models[idx].entry.provider.clone();
            (name, provider_config)
        };

        // 检查缓存
        {
            let providers = self.providers.lock().await;
            if let Some(provider) = providers.get(&name) {
                return Ok(provider.clone());
            }
        }

        // 创建 provider（需要 release models lock 后再获取）
        let provider: Box<dyn ModelProvider> = ProviderFactory::create(&provider_config)?;

        let provider = Arc::new(provider);

        // 写入缓存
        {
            let mut providers = self.providers.lock().await;
            // 再次检查防止并发创建
            if let Some(existing) = providers.get(&name) {
                return Ok(existing.clone());
            }
            providers.insert(name, provider.clone());
        }

        Ok(provider)
    }

    /// 按索引报告失败
    async fn report_failure_by_idx(&self, idx: usize) {
        let models = self.models.read().await;
        if let Some(model) = models.get(idx) {
            let failures = model.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;

            if failures >= self.load_balance.max_failures_before_disable {
                model.current_weight.store(0, Ordering::SeqCst);
                return;
            }

            let current = model.current_weight.load(Ordering::SeqCst);
            let new_weight = ((current as f32 * self.load_balance.failure_decrease_ratio).ceil()
                as u32)
                .max(self.load_balance.min_weight);
            model.current_weight.store(new_weight, Ordering::SeqCst);
        }
    }

    /// 按索引报告成功
    async fn report_success_by_idx(&self, idx: usize) {
        let models = self.models.read().await;
        if let Some(model) = models.get(idx) {
            model.consecutive_failures.store(0, Ordering::SeqCst);

            let current = model.current_weight.load(Ordering::SeqCst);
            let new_weight =
                (current + self.load_balance.recovery_increase).min(model.entry.weight);
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

    /// 调用模型（支持自动重试）
    ///
    /// 按负载均衡策略选择初始模型，失败时自动尝试下一个有效模型，
    /// 直到所有模型都失败或其中一个成功。
    pub async fn invoke<F, Fut, R>(&self, capability: ModelCapability, f: F) -> Result<R>
    where
        F: Fn(Arc<Box<dyn ModelProvider>>) -> Fut,
        Fut: Future<Output = Result<R>>,
    {
        // 获取该能力的模型索引
        let indices = {
            let index = self.capability_index.read().await;
            index.get(&capability).cloned().unwrap_or_default()
        };

        if indices.is_empty() {
            return Err(ModelError::NoAvailableModel);
        }

        // 获取所有有效模型（weight > 0）
        let valid_indices: Vec<usize> = {
            let models = self.models.read().await;
            indices
                .into_iter()
                .filter(|&i| models[i].weight() > 0)
                .collect()
        };

        if valid_indices.is_empty() {
            return Err(ModelError::NoAvailableModel);
        }

        // 获取初始起始偏移（根据负载均衡策略）
        let start_offset = {
            let models = self.models.read().await;
            match self.load_balance.strategy {
                LoadBalanceType::WeightedRoundRobin => {
                    self.round_robin_counter.fetch_add(1, Ordering::SeqCst) % valid_indices.len()
                }
                LoadBalanceType::WeightedRandom => {
                    let total_weight: u32 = valid_indices.iter().map(|&i| models[i].weight()).sum();
                    if total_weight == 0 {
                        0usize
                    } else {
                        (rand::random::<u32>() % total_weight) as usize
                    }
                }
            }
        };

        // 遍历所有有效模型进行重试
        let mut errors = Vec::new();
        for offset in 0..valid_indices.len() {
            let idx = valid_indices[(start_offset + offset) % valid_indices.len()];

            // 检查权重是否仍大于0（可能被其他请求在并发时改变）
            {
                let models = self.models.read().await;
                if models[idx].weight() == 0 {
                    continue;
                }
            }

            // 获取模型名称（用于错误信息）
            let model_name = {
                let models = self.models.read().await;
                models[idx].entry.name.clone()
            };

            // 获取或创建 provider
            let provider = match self.ensure_provider(idx).await {
                Ok(p) => p,
                Err(e) => {
                    errors.push((model_name, e));
                    continue;
                }
            };

            // 调用
            match f(provider).await {
                Ok(r) => {
                    self.report_success_by_idx(idx).await;
                    return Ok(r);
                }
                Err(e) => {
                    self.report_failure_by_idx(idx).await;
                    errors.push((model_name, e));
                    // 继续尝试下一个模型
                }
            }
        }

        // 所有模型都失败
        Err(ModelError::AllModelsFailed(errors))
    }
}
