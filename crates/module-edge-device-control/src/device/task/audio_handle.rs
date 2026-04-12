use crate::error::*;
use nihility_module_model::func::speech_recognition::SpeechRecognitionParam;
use nihility_module_model::Model;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::info;

pub async fn start_audio_handle(
    device_id: String,
    model: Arc<RwLock<Model>>,
    mut speech_receiver: UnboundedReceiver<Vec<f32>>,
) -> Result<JoinHandle<Result<()>>> {
    let output_dir = PathBuf::from("output/audio");
    std::fs::create_dir_all(&output_dir).ok();

    let join_handle = tokio::spawn(async move {
        while let Some(audio_data) = speech_receiver.recv().await {
            let asr_result = model
                .read()
                .await
                .speech_recognition(SpeechRecognitionParam { audio_data })
                .await?;
            info!("Audio auto speech recognition result: {:?}", asr_result);
        }
        info!("Device {} audio handle task exit", device_id);
        Result::Ok(())
    });

    Ok(join_handle)
}
