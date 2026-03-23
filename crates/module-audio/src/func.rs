use crate::func::merge_channels::MergeChannelsParam;
use crate::func::pcm_to_wav::PcmToWavParam;
use crate::AudioModule;
use nihility_module::{BoxStream, Callable, FunctionMetadata, Module};
use schemars::schema_for;
use serde_json::Value;

pub mod merge_channels;
pub mod pcm_to_wav;

#[async_trait::async_trait]
impl Callable for AudioModule {
    async fn call(&self, func_name: &str, param: Value) -> anyhow::Result<Value> {
        match func_name {
            "pcm_to_wav" => {
                let p = serde_json::from_value::<PcmToWavParam>(param)?;
                let result = self.pcm_to_wav(p)?;
                Ok(serde_json::to_value(result)?)
            }
            "merge_channels" => {
                let p = serde_json::from_value::<MergeChannelsParam>(param)?;
                let result = self.merge_channels(p)?;
                Ok(serde_json::to_value(result)?)
            }
            _ => Err(anyhow::anyhow!("Unsupported func_name: {}", func_name)),
        }
    }

    async fn call_mut(&mut self, _func_name: &str, _param: Value) -> anyhow::Result<Value> {
        Err(anyhow::anyhow!("AudioModule does not support mutable calls"))
    }

    async fn call_stream(
        &self,
        _func_name: &str,
        _param: Value,
    ) -> anyhow::Result<BoxStream<Value>> {
        Err(anyhow::anyhow!(
            "vad_stream is not exposed via Callable, use create_vad_stream_handler directly"
        ))
    }
}

impl Module for AudioModule {
    fn description(&self) -> &str {
        "音频处理模块，提供WAV格式转换、声道合并、VAD等功能"
    }

    fn no_perm_func(&self) -> Vec<FunctionMetadata> {
        vec![
            FunctionMetadata {
                name: "pcm_to_wav".to_string(),
                desc: "将f32音频数据转换为WAV格式".to_string(),
                tags: vec!["audio".to_string()],
                params: serde_json::to_value(schema_for!(PcmToWavParam))
                    .expect("audio func pcm_to_wav build param"),
            },
            FunctionMetadata {
                name: "merge_channels".to_string(),
                desc: "将多声道音频数据合并为单声道".to_string(),
                tags: vec!["audio".to_string()],
                params: serde_json::to_value(schema_for!(MergeChannelsParam))
                    .expect("audio func merge_channels build param"),
            },
        ]
    }

    fn perm_func(&mut self) -> Vec<FunctionMetadata> {
        vec![]
    }
}
