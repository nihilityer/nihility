use crate::display::epd::ssd2683::{Command, DeepSleepMode};
use crate::display::epd::EpdInterface;
use crate::display::epd_trait::EpdDisplay;
use anyhow::Result;
use esp_hal::delay::Delay;
use log::debug;

/// 将两个字节交织为 SSD2683 的 2BPP 格式
///
/// SSD2683 使用 2BPP（每像素2位）格式，每个字节表示4个像素：
/// - 高4位: 像素0-3 (位6,7 = 像素0, 位4,5 = 像素1, 位2,3 = 像素2, 位0,1 = 像素3)
/// - 低4位: 像素4-7
///
/// # 参数
/// - `bytes1`: 前一帧数据（用于波形LUT计算）
/// - `bytes2`: 当前帧数据
///
/// # 输出格式
/// 每2位表示一个像素: [byte1_bit, byte2_bit] = 像素值
/// - 00: 黑, 01: 红, 10: 浮/白, 11: 白
fn bit_interleave(bytes1: u8, bytes2: u8) -> (u8, u8) {
    let mut result: u16 = 0;

    for i in 0..8 {
        // 从 bytes1 取出第 (7-i) 位，放到 result 的第 (2*(7-i)+1) 位
        let bit1 = (bytes1 >> (7 - i)) & 0x01;
        result |= (bit1 as u16) << (2 * (7 - i) + 1);

        // 从 bytes2 取出第 (7-i) 位，放到 result 的第 (2*(7-i)) 位
        let bit2 = (bytes2 >> (7 - i)) & 0x01;
        result |= (bit2 as u16) << (2 * (7 - i));
    }

    // 前4像素在高位，后4像素在低位
    ((result >> 8) as u8, result as u8)
}

/// 基于数据驱动的 SSD2683 显示驱动
/// 分辨率: 300x400
pub struct Display {
    interface: EpdInterface,
    delay: Delay,
    /// 宽度（字节数，即像素数/8）
    width_bytes: usize,
    /// 高度（像素数）
    height: u16,
}

impl Display {
    /// 创建新的显示驱动实例
    pub fn new(interface: EpdInterface, delay: Delay, width: usize, height: u16) -> Self {
        if !width.is_multiple_of(8) {
            panic!("Width must be multiple of 8");
        }
        Self {
            interface,
            delay,
            width_bytes: width / 8,
            height,
        }
    }

    /// 根据温度值获取 LUT 值
    /// 参考 C 代码中的温度查表
    fn get_temp_lut(&self, temp: u8) -> u8 {
        debug!("get_temp_lut {}", temp);
        if temp <= 5 {
            232 // -24
        } else if temp <= 10 {
            235 // -21
        } else if temp <= 20 {
            238 // -18
        } else if temp <= 30 {
            241 // -15
        } else if temp <= 127 {
            244 // -12
        } else {
            232
        }
    }
}

impl EpdDisplay for Display {
    fn init(&mut self) -> Result<()> {
        // 硬件重置
        self.interface.reset(&self.delay)?;
        self.interface.busy_wait();

        // Panel Setting
        Command::PanelSetting(0x2F, 0x0E).execute(&mut self.interface)?;

        // OTP 初始化控制
        Command::OtpInitControl.execute(&mut self.interface)?;
        self.interface.busy_wait();

        Ok(())
    }

    fn full_update(&mut self, data: &[u8]) -> Result<()> {
        let expected_len = self.width_bytes * self.height as usize;
        if data.len() != expected_len {
            panic!(
                "Data length mismatch: expected {}, got {}",
                expected_len,
                data.len()
            );
        }

        // 读取温度传感器
        self.interface.send_command(0x40)?;
        self.interface.busy_wait();
        let temp = self.interface.receive_data()?;

        // 温度补偿
        let temp_lut = self.get_temp_lut(temp);
        Command::TemperatureWrite(0x02).execute(&mut self.interface)?;
        Command::TemperatureLut(temp_lut).execute(&mut self.interface)?;

        // 更新参数
        self.interface.send_command(0xA5)?;
        self.interface.busy_wait();
        self.delay.delay_millis(10);

        // 写入 RAM (2BPP 格式)
        Command::WriteRamBW.execute(&mut self.interface)?;
        for &byte in data {
            let (high, low) = bit_interleave(0xFF, byte);
            self.interface.send_data(&[high, low])?;
        }

        // Power ON
        Command::PowerOn.execute(&mut self.interface)?;
        self.interface.busy_wait();
        self.delay.delay_millis(10);

        // Display Refresh
        Command::DisplayRefresh.execute(&mut self.interface)?;
        self.interface.send_data(&[0x00])?;
        self.delay.delay_millis(10);
        self.interface.busy_wait();

        // Power OFF
        Command::PowerOff.execute(&mut self.interface)?;
        self.interface.send_data(&[0x00])?;
        self.interface.busy_wait();
        self.delay.delay_millis(20);

        // 深度睡眠
        Command::DeepSleepMode(DeepSleepMode::DiscardRAM).execute(&mut self.interface)?;
        self.interface.send_data(&[0xA5])?;
        self.delay.delay_millis(100);

        Ok(())
    }

    fn partial_update(&mut self, x: u16, y: u16, w: u16, h: u16, data: &[u8]) -> Result<()> {
        // 验证数据长度
        let w_bytes = (w + 7) / 8; // 宽度字节数（向上取整）
        let expected_len = w_bytes as usize * h as usize;
        if data.len() != expected_len {
            panic!(
                "Data length mismatch: expected {} (w_bytes={}, h={}), got {}",
                expected_len,
                w_bytes,
                h,
                data.len()
            );
        }

        // 1. 清空全屏数据
        Command::WriteRamBW.execute(&mut self.interface)?;
        self.interface.busy_wait();
        for _ in 0..self.height as usize {
            for _ in 0..self.width_bytes {
                self.interface.send_data(&[0x00])?;
            }
        }

        // 2. 设置局部刷新窗口
        let x_end = x + w - 1;
        let y_end = y + h - 1;
        Command::SetPartialWindow {
            hrst_h: 0x00,
            hrst_l: x as u8,
            hred_h: 0x00,
            hred_l: x_end as u8,
            vrst_h: ((y >> 8) & 0x03) as u8,
            vrst_l: (y & 0xFF) as u8,
            vred_h: ((y_end >> 8) & 0x03) as u8,
            vred_l: (y_end & 0xFF) as u8,
            trigger: 0x01,
        }
        .execute(&mut self.interface)?;

        // 3. 写入局部数据
        Command::WriteRamBW.execute(&mut self.interface)?;
        self.interface.busy_wait();
        for &byte in data {
            let (high, low) = bit_interleave(0xFF, byte);
            self.interface.send_data(&[high, low])?;
        }

        // 4. Power ON
        Command::PowerOn.execute(&mut self.interface)?;
        self.interface.busy_wait();

        // 5. Display Refresh
        Command::DisplayRefresh.execute(&mut self.interface)?;
        self.interface.send_data(&[0x00])?;
        self.interface.busy_wait();

        // 6. Power OFF
        Command::PowerOff.execute(&mut self.interface)?;
        self.interface.send_data(&[0x00])?;
        self.interface.busy_wait();
        self.delay.delay_millis(20);

        // 7. 深度睡眠
        Command::DeepSleepMode(DeepSleepMode::DiscardRAM).execute(&mut self.interface)?;
        self.interface.send_data(&[0xA5])?;
        self.delay.delay_millis(100);

        Ok(())
    }
}
