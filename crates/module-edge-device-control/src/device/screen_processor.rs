//! 屏幕处理器
//!
//! 集成了屏幕转换和差异检测功能

use crate::error::*;
use nihility_edge_protocol::{
    FullScreenData, IncrementalScreenData, ScreenConfig, ScreenRotation, UpdateRegion,
};
use tracing::{debug, info};

/// 屏幕更新类型
#[derive(Debug)]
pub enum ScreenUpdate {
    /// 全量更新（首次或大幅变化）
    Full(FullScreenData),
    /// 增量更新（中等变化）
    Incremental(IncrementalScreenData),
    /// 跳过更新（变化太小）
    Skip,
}

#[derive(Clone, Debug)]
pub struct ScreenProcessor {
    width: u16,
    height: u16,
    last_frame: Option<Vec<u8>>,
    screen_config: ScreenConfig,
}

impl ScreenProcessor {
    /// 创建屏幕处理器
    pub fn new(width: u16, height: u16, screen_config: ScreenConfig) -> Self {
        Self {
            width,
            height,
            last_frame: None,
            screen_config,
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
                // 应用屏幕配置转换坐标
                let (src_x, src_y) = self.transform_coordinates(x, y);
                let pixel = gray.get_pixel(src_x as u32, src_y as u32)[0];

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

    /// 根据屏幕配置转换坐标
    fn transform_coordinates(&self, x: u16, y: u16) -> (u16, u16) {
        let mut tx = x;
        let mut ty = y;

        // 应用镜像
        if self.screen_config.mirror_horizontal {
            tx = self.width - 1 - tx;
        }
        if self.screen_config.mirror_vertical {
            ty = self.height - 1 - ty;
        }

        // 应用旋转
        match self.screen_config.rotation {
            ScreenRotation::Rotate0 => (tx, ty),
            ScreenRotation::Rotate90 => (ty, self.width - 1 - tx),
            ScreenRotation::Rotate180 => (self.width - 1 - tx, self.height - 1 - ty),
            ScreenRotation::Rotate270 => (self.height - 1 - ty, tx),
        }
    }

    fn expected_size(width: usize, height: usize) -> usize {
        (width * height).div_ceil(8)
    }

    /// 比较两帧，确定更新类型
    pub fn diff(&mut self, new_frame: FullScreenData) -> ScreenUpdate {
        let Some(ref last_frame) = self.last_frame else {
            // 首次推送，使用全量更新
            info!("First frame: sending full screen update");
            self.last_frame = Some(new_frame.data.clone());
            return ScreenUpdate::Full(new_frame);
        };

        // 计算变化像素数
        let changed_pixels = self.count_changed_pixels(last_frame, &new_frame.data);
        let total_pixels = self.width as usize * self.height as usize;

        // 没有像素更新时不发送任何消息
        if changed_pixels == 0 {
            return ScreenUpdate::Skip;
        }

        let change_percent = (changed_pixels * 100) / total_pixels;

        debug!(
            "Screen change detected: {}/{} pixels ({}.{:02}%)",
            changed_pixels,
            total_pixels,
            change_percent,
            ((changed_pixels * 10000) / total_pixels) % 100
        );

        // 如果变化超过 50%，使用全量更新
        if change_percent > 50 {
            info!(
                "Large change detected ({}% > 50%), sending full screen update",
                change_percent
            );
            self.last_frame = Some(new_frame.data.clone());
            return ScreenUpdate::Full(new_frame);
        }

        // 生成合并后的单一更新区域
        let regions = self.generate_merged_region(last_frame, &new_frame.data);

        let regions_bytes: usize = regions.iter().map(|r| r.data.len()).sum();
        info!(
            "Incremental update: {} region(s), {} bytes total ({}% change)",
            regions.len(),
            regions_bytes,
            change_percent
        );

        self.last_frame = Some(new_frame.data.clone());

        ScreenUpdate::Incremental(IncrementalScreenData {
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

    /// 生成合并后的单一更新区域（计算所有变化像素的边界矩形）
    fn generate_merged_region(&self, old: &[u8], new: &[u8]) -> Vec<UpdateRegion> {
        let mut min_x = self.width;
        let mut min_y = self.height;
        let mut max_x = 0u16;
        let mut max_y = 0u16;
        let mut has_change = false;

        // 遍历所有像素，找出变化区域的边界
        for y in 0..self.height {
            for x in 0..self.width {
                let bit_index = (y as usize * self.width as usize) + x as usize;
                let byte_index = bit_index / 8;
                let bit_offset = 7 - (bit_index % 8);

                let old_bit = (old[byte_index] >> bit_offset) & 1;
                let new_bit = (new[byte_index] >> bit_offset) & 1;

                if old_bit != new_bit {
                    has_change = true;
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }

        if !has_change {
            return Vec::new();
        }

        // 计算边界矩形的宽高
        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;

        // 确保宽度是8的倍数（对于1-bit位图的字节对齐）
        // 将 min_x 向下对齐到8的倍数
        let aligned_min_x = (min_x / 8) * 8;
        // 将宽度向上取整到8的倍数，同时确保不超出屏幕边界
        let raw_width = (max_x - aligned_min_x + 1) as usize;
        let aligned_width = raw_width.div_ceil(8) * 8;
        let final_width = aligned_width.min((self.width - aligned_min_x) as usize) as u16;

        debug!(
            "Region alignment: original x={}, width={} -> aligned x={}, width={}",
            min_x, width, aligned_min_x, final_width
        );

        // 提取该区域的数据
        let data = self.extract_block(new, aligned_min_x, min_y, final_width, height);

        vec![UpdateRegion {
            x: aligned_min_x,
            y: min_y,
            width: final_width,
            height,
            data,
        }]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_coordinates_no_rotation() {
        let config = ScreenConfig {
            rotation: ScreenRotation::Rotate0,
            mirror_horizontal: false,
            mirror_vertical: false,
        };
        let processor = ScreenProcessor::new(400, 300, config);

        assert_eq!(processor.transform_coordinates(0, 0), (0, 0));
        assert_eq!(processor.transform_coordinates(399, 299), (399, 299));
        assert_eq!(processor.transform_coordinates(100, 150), (100, 150));
    }

    #[test]
    fn test_transform_coordinates_rotate_90() {
        let config = ScreenConfig {
            rotation: ScreenRotation::Rotate90,
            mirror_horizontal: false,
            mirror_vertical: false,
        };
        let processor = ScreenProcessor::new(400, 300, config);

        // (0, 0) -> (0, 399)
        assert_eq!(processor.transform_coordinates(0, 0), (0, 399));
        // (399, 0) -> (0, 0)
        assert_eq!(processor.transform_coordinates(399, 0), (0, 0));
        // (399, 299) -> (299, 0)
        assert_eq!(processor.transform_coordinates(399, 299), (299, 0));
    }

    #[test]
    fn test_transform_coordinates_rotate_180() {
        let config = ScreenConfig {
            rotation: ScreenRotation::Rotate180,
            mirror_horizontal: false,
            mirror_vertical: false,
        };
        let processor = ScreenProcessor::new(400, 300, config);

        assert_eq!(processor.transform_coordinates(0, 0), (399, 299));
        assert_eq!(processor.transform_coordinates(399, 299), (0, 0));
        assert_eq!(processor.transform_coordinates(200, 150), (199, 149));
    }

    #[test]
    fn test_transform_coordinates_rotate_270() {
        let config = ScreenConfig {
            rotation: ScreenRotation::Rotate270,
            mirror_horizontal: false,
            mirror_vertical: false,
        };
        let processor = ScreenProcessor::new(400, 300, config);

        // (0, 0) -> (299, 0)
        assert_eq!(processor.transform_coordinates(0, 0), (299, 0));
        // (0, 299) -> (0, 0)
        assert_eq!(processor.transform_coordinates(0, 299), (0, 0));
        // (399, 299) -> (0, 399)
        assert_eq!(processor.transform_coordinates(399, 299), (0, 399));
    }

    #[test]
    fn test_transform_coordinates_mirror_horizontal() {
        let config = ScreenConfig {
            rotation: ScreenRotation::Rotate0,
            mirror_horizontal: true,
            mirror_vertical: false,
        };
        let processor = ScreenProcessor::new(400, 300, config);

        assert_eq!(processor.transform_coordinates(0, 0), (399, 0));
        assert_eq!(processor.transform_coordinates(399, 0), (0, 0));
        assert_eq!(processor.transform_coordinates(200, 150), (199, 150));
    }

    #[test]
    fn test_transform_coordinates_mirror_vertical() {
        let config = ScreenConfig {
            rotation: ScreenRotation::Rotate0,
            mirror_horizontal: false,
            mirror_vertical: true,
        };
        let processor = ScreenProcessor::new(400, 300, config);

        assert_eq!(processor.transform_coordinates(0, 0), (0, 299));
        assert_eq!(processor.transform_coordinates(0, 299), (0, 0));
        assert_eq!(processor.transform_coordinates(200, 150), (200, 149));
    }

    #[test]
    fn test_transform_coordinates_mirror_both() {
        let config = ScreenConfig {
            rotation: ScreenRotation::Rotate0,
            mirror_horizontal: true,
            mirror_vertical: true,
        };
        let processor = ScreenProcessor::new(400, 300, config);

        assert_eq!(processor.transform_coordinates(0, 0), (399, 299));
        assert_eq!(processor.transform_coordinates(399, 299), (0, 0));
        assert_eq!(processor.transform_coordinates(200, 150), (199, 149));
    }
}
