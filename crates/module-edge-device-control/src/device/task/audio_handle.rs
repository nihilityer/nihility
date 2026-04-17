use crate::error::*;
use nihility_module_message_pool::func::add_message::AddMessagesParam;
use nihility_module_message_pool::{ContentData, Message, MessagePool};
use nihility_module_model::func::speech_recognition::SpeechRecognitionParam;
use nihility_module_model::Model;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{oneshot, RwLock};
use tokio::task::JoinHandle;
use tracing::info;
use uuid::Uuid;

pub async fn start_audio_handle(
    scene_id_receiver: oneshot::Receiver<Uuid>,
    device_id: String,
    model: Arc<RwLock<Model>>,
    message_pool: Arc<RwLock<MessagePool>>,
    mut speech_receiver: UnboundedReceiver<Vec<f32>>,
) -> Result<JoinHandle<Result<()>>> {
    let output_dir = PathBuf::from("output/audio");
    std::fs::create_dir_all(&output_dir).ok();

    let join_handle = tokio::spawn(async move {
        let scene_id = scene_id_receiver.await.map_err(|e| {
            EdgeDeviceControlError::DeviceStatus(format!(
                "Device {} scene id receiver error: {}",
                device_id, e
            ))
        })?;
        while let Some(audio_data) = speech_receiver.recv().await {
            let asr_result = model
                .read()
                .await
                .speech_recognition(SpeechRecognitionParam { audio_data })
                .await?;
            info!("Audio auto speech recognition result: {:?}", asr_result);
            message_pool
                .read()
                .await
                .add_messages(AddMessagesParam {
                    scene_id,
                    messages: vec![Message {
                        content: ContentData::Text { body: asr_result },
                        metadata: Default::default(),
                    }],
                })
                .await?;
        }
        info!("Device {} audio handle task exit", device_id);
        Result::Ok(())
    });

    Ok(join_handle)
}
