use crate::error::Result;
use crate::provider::ModelProvider;
use crate::utils::Cmvn;
use crate::utils::Lfr;
use crate::utils::{OnlineFbank, OnlineFbankConfig};
use async_trait::async_trait;
use ndarray::{Array2, Axis};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokenizers::Tokenizer;
use tokio::sync::Mutex;
use tracing::debug;

/// SenseVoice模型初始化配置
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SenseVoiceConfig {
    /// onnx模型路径
    pub model_path: String,
    /// tokenizer文件路径
    pub tokenizer_path: String,
    /// 每个输出帧包含的输入帧数（上下文窗口大小）
    pub lfr_m: usize,
    /// 帧率降低的倍数（跳跃步长）
    pub lfr_n: usize,
    /// 倒谱均值方差归一化数据地址
    pub cmvn_path: String,
    /// 识别目标语言
    pub language: SenseVoiceLanguage,
    /// 规范化文本方式
    pub text_norm: SenseVoiceTextNorm,
    /// 是否移除识别结果中表示状态的前四个token
    pub remove_status_token: bool,
    /// fbank特征提取相关配置
    pub online_fbank_config: OnlineFbankConfig,
}

/// SenseVoice模型
pub struct SenseVoice {
    /// 识别目标语言
    language: SenseVoiceLanguage,
    /// 规范化文本方式
    text_norm: SenseVoiceTextNorm,
    /// 是否移除识别结果中表示状态的前四个token
    remove_status_token: bool,
    /// Low Frame Rate
    lfr: Lfr,
    /// 倒谱均值方差归一化
    cmvn: Cmvn,
    /// fbank特征提取
    online_fbank: Arc<Mutex<OnlineFbank>>,
    /// onnx模型session
    session: Arc<Mutex<Session>>,
    /// tokenizer
    tokenizer: Tokenizer,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum SenseVoiceLanguage {
    Auto = 0,
    Zh = 3,
    En = 4,
    Yue = 7,
    Ja = 11,
    Ko = 12,
    NoSpeech = 13,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum SenseVoiceTextNorm {
    WithItn = 14,
    WoItn = 15,
}

impl SenseVoice {
    pub fn init(config: SenseVoiceConfig) -> Result<Self> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| {
                crate::error::ModelError::Provider(format!("Sense Voice Ort Session build fail: {}", e))
            })?
            .with_inter_threads(1)
            .map_err(|e| {
                crate::error::ModelError::Provider(format!("Sense Voice Ort Session build fail: {}", e))
            })?
            .with_intra_threads(1)
            .map_err(|e| {
                crate::error::ModelError::Provider(format!("Sense Voice Ort Session build fail: {}", e))
            })?
            .commit_from_file(&config.model_path)?;

        let tokenizer = Tokenizer::from_file(&config.tokenizer_path)
            .map_err(|e| crate::error::ModelError::Provider(format!("tokenizer error: {}", e)))?;

        Ok(Self {
            language: config.language,
            text_norm: config.text_norm,
            remove_status_token: config.remove_status_token,
            lfr: Lfr::init(config.lfr_m, config.lfr_n),
            cmvn: Cmvn::init(config.cmvn_path)?,
            online_fbank: Arc::new(Mutex::new(OnlineFbank::init(config.online_fbank_config))),
            session: Arc::new(Mutex::new(session)),
            tokenizer,
        })
    }

    pub async fn infer(&self, waveform: Vec<f32>) -> Result<String> {
        let (feat, feat_len) = self.extract_feat(waveform).await;
        let ort_input = ort::inputs![
            Tensor::from_array(feat.insert_axis(Axis(0)))?,
            Tensor::from_array(([1], vec![feat_len as i32]))?,
            Tensor::from_array(([1], vec![self.language as i32]))?,
            Tensor::from_array(([1], vec![self.text_norm as i32]))?,
        ];
        let mut session = self.session.lock().await;
        let outputs = session.run(ort_input)?;

        let logits_output = outputs.get("ctc_logits").expect("Missing ctc_logits");
        let lens_output = outputs
            .get("encoder_out_lens")
            .expect("Missing encoder_out_lens");

        // 提取数据为 ndarray::ArrayView
        let logits_tensor = logits_output.try_extract_array::<f32>()?;
        let lens_tensor = lens_output.try_extract_array::<i32>()?;

        let logits_view = logits_tensor.view();
        let lens_view = lens_tensor.view();

        // 获取有效长度 (batch_size=1, 所以取 index 0)
        let actual_len = lens_view[0] as usize;

        // logits shape: [1, logits_length, 25055]
        // 我们将其视为扁平切片来加速处理
        let vocab_size = logits_view.shape()[2];
        let raw_logits = logits_view
            .as_slice()
            .expect("Model output logits exception"); // 变为一维 slice

        let mut yseq: Vec<u32> = Vec::with_capacity(actual_len);

        // 循环每一帧 (只处理到 actual_len)
        for t in 0..actual_len {
            // 计算当前帧在扁平数组中的偏移量
            // 因为 batch=1，可以直接忽略 batch 维度
            let start = t * vocab_size;
            let end = start + vocab_size;
            let frame = &raw_logits[start..end];

            // Argmax: 找到概率最大的索引
            if let Some((idx, _)) = frame
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
            {
                yseq.push(idx as u32);
            }
        }

        // Unique Consecutive (去除连续重复的 token)
        yseq.dedup();

        // Remove Blank ID (过滤 blank)， blank_id为`0`
        let mut token_int: Vec<u32> = yseq.into_iter().filter(|&id| id != 0).collect();

        // 移除前四个表示识别结果状态的token
        if self.remove_status_token {
            let status_token_ids = token_int.drain(..4).collect::<Vec<_>>();

            let status_token = self
                .tokenizer
                .decode(&status_token_ids, true)
                .map_err(|e| crate::error::ModelError::Provider(format!("tokenizer error: {}", e)))?;
            debug!("SenseVoice infer status result: {:?}", status_token);
        }

        self.tokenizer
            .decode(&token_int, true)
            .map_err(|e| crate::error::ModelError::Provider(format!("tokenizer error: {}", e)))
    }

    /// 计算输入音频数据的特征数据
    async fn extract_feat(&self, waveform: Vec<f32>) -> (Array2<f32>, usize) {
        let mut fb = self.online_fbank.lock().await;
        fb.accept_waveform(waveform);
        let speech = fb.process_all();
        fb.buffer_clear();

        let feat = self.lfr.apply_lfr(speech);
        let feat = self.cmvn.apply_cmvn(feat);
        let feat_len = feat.len_of(Axis(0));
        (feat, feat_len)
    }
}

impl Default for SenseVoiceConfig {
    fn default() -> Self {
        Self {
            model_path: "model/sense_voice/model_quant.onnx".to_string(),
            tokenizer_path: "model/sense_voice/tokenizer.json".to_string(),
            lfr_m: 7,
            lfr_n: 6,
            cmvn_path: "model/sense_voice/cmvn.npy".to_string(),
            language: SenseVoiceLanguage::Auto,
            text_norm: SenseVoiceTextNorm::WithItn,
            remove_status_token: true,
            online_fbank_config: OnlineFbankConfig::default(),
        }
    }
}

#[async_trait]
impl ModelProvider for SenseVoice {
    async fn speech_recognition(
        &self,
        audio_data: &[f32],
        sample_rate: u32,
        channels: u8,
        audio_module: &Arc<nihility_module_audio::AudioModule>,
    ) -> Result<String> {
        // 验证采样率（固定 16000）
        debug_assert_eq!(
            sample_rate, 16000,
            "Audio sample rate must be 16000 Hz"
        );

        // Step 1: 声道合并 (stereo -> mono)
        let waveform = if channels > 1 {
            audio_module.merge_channels(crate::MergeChannelsParam {
                waveform: audio_data.to_vec(),
                channels,
            })?
        } else {
            audio_data.to_vec()
        };

        // Step 2: 推理
        self.infer(waveform).await
    }
}
