use crate::error::ModelError;
use hound::{WavSpec, WavWriter};
use std::io::Cursor;

/// 将 PCM 数据转换为 WAV 格式
pub fn pcm_to_wav(
    pcm_data: &[u8],
    sample_rate: u32,
    channels: u8,
    bits_per_sample: u8,
) -> crate::error::Result<Vec<u8>> {
    let spec = WavSpec {
        channels: channels as u16,
        sample_rate,
        bits_per_sample: bits_per_sample as u16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buffer = Vec::new();
    {
        let mut writer = WavWriter::new(Cursor::new(&mut buffer), spec)?;

        // 根据位深处理 PCM 数据
        match bits_per_sample {
            8 => {
                // 8-bit PCM: unsigned, 0-255 -> 转换为 signed i8: -128 to 127
                for &byte in pcm_data {
                    let sample = (byte as i8).wrapping_sub(-128i8);
                    writer.write_sample(sample)?;
                }
            }
            16 => {
                // 16-bit PCM: signed, little-endian
                for chunk in pcm_data.chunks(2) {
                    if chunk.len() == 2 {
                        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                        writer.write_sample(sample)?;
                    }
                }
            }
            24 => {
                // 24-bit PCM: signed, little-endian (stored as i32 in the file)
                for chunk in pcm_data.chunks(3) {
                    if chunk.len() == 3 {
                        let sample = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], 0]);
                        writer.write_sample(sample)?;
                    }
                }
            }
            32 => {
                // 32-bit PCM: signed
                for chunk in pcm_data.chunks(4) {
                    if chunk.len() == 4 {
                        let sample = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                        writer.write_sample(sample)?;
                    }
                }
            }
            _ => {
                return Err(ModelError::AudioEncode(format!(
                    "Unsupported bits_per_sample: {}",
                    bits_per_sample
                )));
            }
        }

        writer.finalize()?;
    }

    Ok(buffer)
}
