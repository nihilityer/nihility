use crate::display::epd::ssd1683::{Command, DataEntryMode, DeepSleepMode, IncrementAxis};
use crate::display::epd::EpdInterface;
use crate::display::epd_trait::EpdDisplay;
use anyhow::Result;
use esp_hal::delay::Delay;

/// 基于数据驱动的显示驱动
pub struct Display {
    interface: EpdInterface,
    delay: Delay,
    width: usize,
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
            width,
            height,
        }
    }

    /// 全局正常刷新初始化
    pub fn normal_init(&mut self) -> Result<()> {
        // 硬件重置
        self.interface.reset(&self.delay)?;
        self.interface.busy_wait();

        // 软复位
        Command::SoftReset.execute(&mut self.interface)?;
        self.interface.busy_wait();

        Command::DriverOutputControl(self.height - 1, 0x00).execute(&mut self.interface)?;
        Command::DisplayUpdateControl1(0x4000).execute(&mut self.interface)?;
        Command::BorderWaveform(0x05).execute(&mut self.interface)?;
        Command::DataEntryMode(
            DataEntryMode::IncrementXDecrementY,
            IncrementAxis::Horizontal,
        )
        .execute(&mut self.interface)?;

        // 设置显示窗口
        Command::StartEndXPosition(0x00, (self.width / 8 - 1) as u8)
            .execute(&mut self.interface)?;
        Command::StartEndYPosition(self.height - 1, 0x00).execute(&mut self.interface)?;

        // 设置 RAM 地址
        Command::XAddress(0x00).execute(&mut self.interface)?;
        Command::YAddress(self.height - 1).execute(&mut self.interface)?;
        self.interface.busy_wait();

        Ok(())
    }

    /// 全局快速刷新初始化
    /// more_fast为true时，大约1.0s,反之1.5s
    pub fn fast_init(&mut self, more_fast: bool) -> Result<()> {
        // 硬件重置
        self.interface.reset(&self.delay)?;
        self.interface.busy_wait();

        // 软复位
        Command::SoftReset.execute(&mut self.interface)?;
        self.interface.busy_wait();

        Command::DisplayUpdateControl1(0x4000).execute(&mut self.interface)?;
        Command::BorderWaveform(0x05).execute(&mut self.interface)?;

        // 设置温度寄存器
        if more_fast {
            Command::WriteTemperatureSensor(0x5A).execute(&mut self.interface)?;
        } else {
            Command::WriteTemperatureSensor(0x6E).execute(&mut self.interface)?;
        }

        // 加载温度值
        Command::DisplayUpdateControl2(0x91).execute(&mut self.interface)?;
        Command::MasterActivation.execute(&mut self.interface)?;
        self.interface.busy_wait();

        // 配置数据输入模式
        Command::DataEntryMode(
            DataEntryMode::IncrementXDecrementY,
            IncrementAxis::Horizontal,
        )
        .execute(&mut self.interface)?;

        // 设置显示窗口
        Command::StartEndXPosition(0x00, (self.width / 8 - 1) as u8)
            .execute(&mut self.interface)?;
        Command::StartEndYPosition(self.height - 1, 0x00).execute(&mut self.interface)?;

        // 设置 RAM 地址
        Command::XAddress(0x00).execute(&mut self.interface)?;
        Command::YAddress(self.height - 1).execute(&mut self.interface)?;
        self.interface.busy_wait();

        Ok(())
    }

    /// 执行全局正常刷新
    pub fn normal_update(&mut self) -> Result<()> {
        Command::DisplayUpdateControl2(0xF7).execute(&mut self.interface)?;
        Command::MasterActivation.execute(&mut self.interface)?;
        self.interface.busy_wait();
        Ok(())
    }

    /// 执行全局快速刷新
    pub fn fast_update(&mut self) -> Result<()> {
        Command::DisplayUpdateControl2(0xC7).execute(&mut self.interface)?;
        Command::MasterActivation.execute(&mut self.interface)?;
        self.interface.busy_wait();
        Ok(())
    }

    /// 执行局部刷新
    pub fn part_update(&mut self) -> Result<()> {
        Command::DisplayUpdateControl2(0xFF).execute(&mut self.interface)?;
        Command::MasterActivation.execute(&mut self.interface)?;
        self.interface.busy_wait();
        Ok(())
    }

    /// 写入图像数据
    ///
    /// # 参数
    /// - `data`: 显示数据（长度必须等于 width * height / 8）
    ///
    /// # 数据格式
    /// - 每个字节代表 8 个水平像素，LSB 在右侧
    /// - 0 = 黑色，1 = 白色
    pub fn write_all(&mut self, data: &[u8]) -> Result<()> {
        let expected_len = (self.width * self.height as usize) / 8;
        if data.len() != expected_len {
            panic!(
                "Data length mismatch: expected {}, got {}",
                expected_len,
                data.len()
            );
        }

        // 写入黑白数据
        Command::WriteRamBW.execute(&mut self.interface)?;
        self.interface.send_data(data)?;

        // 写入红色数据（全白，即无红色）
        Command::WriteRamRed.execute(&mut self.interface)?;
        self.interface.send_data(data)?;

        Ok(())
    }

    /// 局部刷新（只刷新指定区域，速度最快但可能有残影）
    ///
    /// # 参数
    /// - `x`: 区域起始 X 坐标（像素）
    /// - `y`: 区域起始 Y 坐标（像素）
    /// - `width`: 区域宽度（像素，必须是 8 的倍数）
    /// - `height`: 区域高度（像素）
    /// - `data`: 区域数据（长度必须等于 width * height / 8）
    ///
    /// # 约束
    /// - width 必须是 8 的倍数
    /// - data 长度必须等于 width * height / 8
    ///
    /// # 注意
    /// - 局部刷新适合小范围更新，速度最快
    /// - 长期使用可能产生残影，建议定期执行全屏正常刷新
    pub fn part_write(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        data: &[u8],
    ) -> Result<()> {
        if !width.is_multiple_of(8) {
            panic!("Region width must be multiple of 8");
        }

        let expected_len = (width as usize * height as usize) / 8;
        if data.len() != expected_len {
            panic!(
                "Region data length mismatch: expected {}, got {}",
                expected_len,
                data.len()
            );
        }

        // 计算字节列位置
        let start_byte_col = (x / 8) as u8;
        let end_byte_col = ((x + width) / 8 - 1) as u8;

        // 计算Y坐标（官方示例中需要减1）
        let y_start = y.saturating_sub(1);
        let y_end = y_start + height - 1;

        // 硬件重置
        self.interface.reset(&self.delay)?;

        // 配置边框波形和显示更新控制（局部刷新配置）
        Command::BorderWaveform(0x80).execute(&mut self.interface)?;
        Command::DisplayUpdateControl1(0x0000).execute(&mut self.interface)?;

        // 设置更新窗口
        Command::StartEndXPosition(start_byte_col, end_byte_col).execute(&mut self.interface)?;
        Command::StartEndYPosition(y_end, y_start).execute(&mut self.interface)?;

        // 设置 RAM 地址
        Command::XAddress(start_byte_col).execute(&mut self.interface)?;
        Command::YAddress(y_start).execute(&mut self.interface)?;

        // 写入区域数据
        Command::WriteRamBW.execute(&mut self.interface)?;
        self.interface.send_data(data)?;

        Ok(())
    }

    /// 手动进入深度睡眠模式
    pub fn deep_sleep(&mut self, mode: DeepSleepMode) -> Result<()> {
        Command::DeepSleepMode(mode).execute(&mut self.interface)?;
        self.delay.delay_millis(100);
        Ok(())
    }

    /// 写入图像数据（双RAM版本）
    ///
    /// # 参数
    /// - `bw`: 黑白数据
    /// - `red`: 红色数据
    pub fn write_all_with_red(&mut self, bw: &[u8], red: &[u8]) -> Result<()> {
        let expected_len = (self.width * self.height as usize) / 8;
        if bw.len() != expected_len || red.len() != expected_len {
            panic!("Data length mismatch");
        }

        Command::WriteRamBW.execute(&mut self.interface)?;
        self.interface.send_data(bw)?;

        Command::WriteRamRed.execute(&mut self.interface)?;
        self.interface.send_data(red)?;

        Ok(())
    }
}

impl EpdDisplay for Display {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn init(&mut self) -> Result<()> {
        self.normal_init()
    }

    fn init_fast(&mut self, use_otp: bool) -> Result<()> {
        // SSD1683: use_otp=false means more_fast=true, use_otp=true means more_fast=false
        self.fast_init(!use_otp)
    }

    fn write_all(&mut self, black_white: &[u8], red: &[u8]) -> Result<()> {
        self.write_all_with_red(black_white, red)
    }

    fn update(&mut self) -> Result<()> {
        self.normal_update()
    }

    fn write_partial(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        data: &[u8],
    ) -> Result<()> {
        self.part_write(x, y, width, height, data)
    }

    fn update_partial(&mut self) -> Result<()> {
        self.part_update()
    }

    fn deep_sleep(&mut self) -> Result<()> {
        // Use DiscardRAM as sensible default
        self.deep_sleep(DeepSleepMode::DiscardRAM)
    }
}
