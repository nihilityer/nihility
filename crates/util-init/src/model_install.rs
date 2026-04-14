use crate::error::*;
use hf_hub::Cache;
use std::fs;
use std::path::Path;
use tokio::task::JoinSet;
use tracing::info;

const MODEL_DIR: &str = "model";

const MODEL_REPO: &str = "nihilityer/nihility";

const DOWNLOAD_LIST: [&str; 4] = [
    "silero_vad.onnx",
    "sense_voice/tokenizer.json",
    "sense_voice/model_quant.onnx",
    "sense_voice/cmvn.npy",
];

pub async fn model_install() -> Result<()> {
    let model_dir = Path::new(MODEL_DIR);
    if !model_dir.exists() {
        info!("Creating model directory {}", MODEL_DIR);
        fs::create_dir_all(model_dir)?;
    }

    let mut task_set = JoinSet::<Result<String>>::new();
    for model in DOWNLOAD_LIST {
        let model_file = model_dir.join(model);
        if !model_file.exists() {
            info!("Downloading model file {}", model);
            task_set.spawn(async move {
                let model_repo = hf_hub::api::tokio::ApiBuilder::from_env()
                    .with_cache_dir(model_dir.to_path_buf())
                    .build()?
                    .model(MODEL_REPO.to_string());
                model_repo.download(model).await?;
                Ok(model.to_string())
            });
        }
    }
    let cache = Cache::new(model_dir.to_path_buf()).model(MODEL_REPO.to_string());
    while let Some(result) = task_set.join_next().await {
        match result {
            Ok(Ok(model)) => {
                info!("Model file: {} download success", model);
                match cache.get(&model) {
                    None => {
                        return Err(InitError::Model(format!("Cannot find model: {}", model)));
                    }
                    Some(cache_path) => {
                        let model_path = model_dir.join(&model);
                        if let Some(model_pre) = model_path.parent()
                            && !model_pre.exists()
                        {
                            fs::create_dir_all(model_pre)?;
                        }
                        if let Err(e) = fs::copy(cache_path, model_path) {
                            return Err(InitError::Model(format!(
                                "Cannot copy model {}: {}",
                                model, e
                            )));
                        }
                    }
                }
            }
            Ok(Err(err)) => {
                return Err(InitError::Model(format!(
                    "Failed to download model: {}",
                    err
                )));
            }
            Err(e) => {
                return Err(InitError::Model(format!("Failed to download model: {}", e)));
            }
        }
    }

    Ok(())
}
