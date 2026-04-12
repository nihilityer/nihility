use crate::error::*;
use std::path::PathBuf;
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;
use tracing::info;

/// 使用 hound 库写入 WAV 文件
fn write_wav_file(file_path: &PathBuf, pcm_data: &[f32]) -> Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(file_path, spec)?;
    for &sample in pcm_data {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    Ok(())
}

pub async fn start_audio_handle(
    device_id: String,
    mut speech_receiver: Receiver<Vec<f32>>,
) -> Result<JoinHandle<Result<()>>> {
    let output_dir = PathBuf::from("output/audio");
    std::fs::create_dir_all(&output_dir).ok();

    Ok(tokio::spawn(async move {
        while let Ok(speech_data) = speech_receiver.recv().await {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let file_name = format!("{}_{}.wav", device_id, timestamp);
            let file_path = output_dir.join(&file_name);
            write_wav_file(&file_path, &speech_data)?;
            info!(
                "VAD speech segment saved: {} ({} samples)",
                file_path.display(),
                speech_data.len()
            );
        }
        Result::Ok(())
    }))
}
