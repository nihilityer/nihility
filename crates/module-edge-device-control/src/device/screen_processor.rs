//! 屏幕处理器
//!
//! 集成了屏幕转换和差异检测功能

use crate::error::*;
use nihility_edge_protocol::{FullScreenData, IncrementalScreenData, UpdateRegion};

#[derive(Clone, Debug)]
pub struct ScreenProcessor {
    width: u16,
    height: u16,
    last_frame: Option<Vec<u8>>,
    threshold_percent: u8,
}

impl ScreenProcessor {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            last_frame: None,
            threshold_percent: 5,
        }
    }

    /// PNG 字节 → 1-bit 位图
    pub fn convert_png_to_1bit(&self, png_data: &[u8]) -> Result<FullScreenData> {
        // 1. 解码 PNG
        let img = image::load_from_memory(png_data)?;

        // 2. 缩放到目标尺寸
        let resized = img.resize_exact(
            self.width as u32,
            self.height as u32,
            image::imageops::FilterType::Lanczos3,
        );

        // 3. 转换为灰度
        let gray = resized.to_luma8();

        // 4. 二值化（阈值 128）
        let bitmap = self.binarize(&gray);

        Ok(FullScreenData {
            width: self.width,
            height: self.height,
            data: bitmap,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        })
    }

    /// 灰度图 → 1-bit 位图
    fn binarize(&self, gray: &image::GrayImage) -> Vec<u8> {
        let mut bitmap = vec![0u8; Self::expected_size(self.width as usize, self.height as usize)];

        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = gray.get_pixel(x as u32, y as u32)[0];

                // 阈值二值化（亮度 >= 128 为白色/1，否则黑色/0）
                if pixel >= 128 {
                    let bit_index = (y as usize * self.width as usize) + x as usize;
                    let byte_index = bit_index / 8;
                    let bit_offset = 7 - (bit_index % 8); // 高位在前
                    bitmap[byte_index] |= 1 << bit_offset;
                }
            }
        }

        bitmap
    }

    fn expected_size(width: usize, height: usize) -> usize {
        (width * height).div_ceil(8)
    }

    /// 比较两帧，生成增量更新
    pub fn diff(&mut self, new_frame: &FullScreenData) -> Option<IncrementalScreenData> {
        let Some(ref last_frame) = self.last_frame else {
            // 首次推送，保存帧并返回 None（使用全量更新）
            self.last_frame = Some(new_frame.data.clone());
            return None;
        };

        // 计算变化像素数
        let changed_pixels = self.count_changed_pixels(last_frame, &new_frame.data);
        let total_pixels = self.width as usize * self.height as usize;
        let change_percent = (changed_pixels * 100) / total_pixels;

        // 如果变化小于阈值，跳过更新
        if change_percent < self.threshold_percent as usize {
            return None;
        }

        // 如果变化超过 50%，建议全量更新
        if change_percent > 50 {
            self.last_frame = Some(new_frame.data.clone());
            return None; // 调用者应发送 FullScreenUpdate
        }

        // 生成增量更新区域（按 64x64 块检测）
        let regions = self.generate_regions(last_frame, &new_frame.data);

        self.last_frame = Some(new_frame.data.clone());

        Some(IncrementalScreenData {
            regions,
            timestamp: new_frame.timestamp,
        })
    }

    fn count_changed_pixels(&self, old: &[u8], new: &[u8]) -> usize {
        old.iter()
            .zip(new.iter())
            .map(|(a, b)| (a ^ b).count_ones() as usize)
            .sum()
    }

    fn generate_regions(&self, old: &[u8], new: &[u8]) -> Vec<UpdateRegion> {
        let block_size = 64u16;
        let mut regions = Vec::new();

        for by in (0..self.height).step_by(block_size as usize) {
            for bx in (0..self.width).step_by(block_size as usize) {
                let block_width = (self.width - bx).min(block_size);
                let block_height = (self.height - by).min(block_size);

                if self.is_block_changed(old, new, bx, by, block_width, block_height) {
                    let data = self.extract_block(new, bx, by, block_width, block_height);
                    regions.push(UpdateRegion {
                        x: bx,
                        y: by,
                        width: block_width,
                        height: block_height,
                        data,
                    });
                }
            }
        }

        regions
    }

    fn is_block_changed(&self, old: &[u8], new: &[u8], x: u16, y: u16, w: u16, h: u16) -> bool {
        for row in y..(y + h) {
            for col in x..(x + w) {
                let bit_index = (row as usize * self.width as usize) + col as usize;
                let byte_index = bit_index / 8;
                let bit_offset = 7 - (bit_index % 8);

                let old_bit = (old[byte_index] >> bit_offset) & 1;
                let new_bit = (new[byte_index] >> bit_offset) & 1;

                if old_bit != new_bit {
                    return true;
                }
            }
        }
        false
    }

    fn extract_block(&self, frame: &[u8], x: u16, y: u16, w: u16, h: u16) -> Vec<u8> {
        let mut block = vec![0u8; Self::expected_size(w as usize, h as usize)];
        let mut out_bit_index = 0;

        for row in y..(y + h) {
            for col in x..(x + w) {
                let in_bit_index = (row as usize * self.width as usize) + col as usize;
                let in_byte_index = in_bit_index / 8;
                let in_bit_offset = 7 - (in_bit_index % 8);

                let bit = (frame[in_byte_index] >> in_bit_offset) & 1;

                let out_byte_index = out_bit_index / 8;
                let out_bit_offset = 7 - (out_bit_index % 8);
                block[out_byte_index] |= bit << out_bit_offset;

                out_bit_index += 1;
            }
        }

        block
    }
}
