use crate::display::epd::ssd2683::command::Command;
use crate::display::epd::EpdInterface;
use crate::display::epd_trait::EpdDisplay;
use alloc::vec;
use anyhow::{anyhow, Result};
use esp_hal::delay::Delay;

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

    /// 读取温度传感器值
    fn read_temperature(&mut self) -> Result<u8> {
        Command::ReadTemperatureSensor.execute(&mut self.interface)?;
        self.delay.delay_millis(10);
        Ok(self.interface.receive_data()?)
    }

    /// OTP 初始化（用于快速刷新模式）
    pub fn otp_init(&mut self) -> Result<()> {
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

    /// 全局正常刷新初始化
    pub fn normal_init(&mut self) -> Result<()> {
        // 硬件重置
        self.interface.reset(&self.delay)?;
        self.interface.busy_wait();

        // Panel Setting
        Command::PanelSetting(0x2F, 0x0E).execute(&mut self.interface)?;

        // CDI 边框设置
        Command::CdiBorder(0x77).execute(&mut self.interface)?;

        // 温度 LUT 设置
        Command::TemperatureWrite(0x02).execute(&mut self.interface)?;
        let temp = self.read_temperature()?;
        let temp_lut = self.get_temp_lut(temp);
        Command::TemperatureLut(temp_lut).execute(&mut self.interface)?;

        Ok(())
    }

    /// 执行正常刷新
    pub fn normal_update(&mut self) -> Result<()> {
        Command::PowerOn.execute(&mut self.interface)?;
        self.interface.busy_wait();
        self.delay.delay_millis(10);

        Command::DisplayRefresh.execute(&mut self.interface)?;
        self.delay.delay_millis(10);
        self.interface.busy_wait();

        Command::PowerOff.execute(&mut self.interface)?;
        self.interface.busy_wait();
        self.delay.delay_millis(20);

        Ok(())
    }

    /// 执行快速刷新（使用 OTP 初始化）
    pub fn fast_update(&mut self) -> Result<()> {
        Command::PowerOn.execute(&mut self.interface)?;
        self.interface.busy_wait();
        self.delay.delay_millis(10);

        Command::DisplayRefresh.execute(&mut self.interface)?;
        self.delay.delay_millis(10);
        self.interface.busy_wait();

        Command::PowerOff.execute(&mut self.interface)?;
        self.interface.busy_wait();
        self.delay.delay_millis(20);

        Ok(())
    }

    /// 局部刷新
    ///
    /// # 参数
    /// - `x`: 起始 X 坐标（字节，8像素/字节）
    /// - `y`: 起始 Y 坐标（像素）
    /// - `width_bytes`: 区域宽度（字节）
    /// - `height`: 区域高度（像素）
    /// - `data`: 区域数据
    pub fn part_write(
        &mut self,
        x: u8,
        y: u16,
        width_bytes: u8,
        height: u16,
        data: &[u8],
    ) -> Result<()> {
        let expected_len = width_bytes as usize * height as usize;
        if data.len() != expected_len {
            panic!(
                "Region data length mismatch: expected {}, got {}",
                expected_len,
                data.len()
            );
        }

        let x_end = x + width_bytes - 1;
        let y_start = y;
        let y_end = y + height - 1;

        // 清空窗口
        Command::WriteRamBW.execute(&mut self.interface)?;
        self.interface.busy_wait();

        // 清空整个显示区域
        for _ in 0..self.height as usize {
            for _ in 0..self.width_bytes {
                self.interface.send_data(&[0x00])?;
            }
        }

        // 设置局部刷新窗口
        Command::StartEndXPosition(x, x_end).execute(&mut self.interface)?;
        Command::StartEndYPosition(y_end, y_start).execute(&mut self.interface)?;

        // 写入新数据
        Command::WriteRamBW.execute(&mut self.interface)?;
        self.interface.send_data(data)?;

        Command::PowerOn.execute(&mut self.interface)?;
        self.interface.busy_wait();

        Command::DisplayRefresh.execute(&mut self.interface)?;
        self.interface.busy_wait();

        Command::PowerOff.execute(&mut self.interface)?;
        self.interface.busy_wait();
        self.delay.delay_millis(20);

        Ok(())
    }

    /// 深度睡眠模式
    pub fn deep_sleep(&mut self) -> Result<()> {
        Command::DeepSleepMode(crate::display::epd::ssd2683::command::DeepSleepMode::DiscardRAM)
            .execute(&mut self.interface)?;
        // 发送深度睡眠密钥
        self.interface.send_data(&[0xA5])?;
        self.delay.delay_millis(100);
        Ok(())
    }

    /// 写入图像数据（适配 Display trait 的双参数版本）
    ///
    /// # 参数
    /// - `bw`: 黑白数据
    /// - `_red`: 红色数据（SSD2683 忽略此参数，始终写 0xFF）
    pub fn write_all_with_red(&mut self, bw: &[u8], _red: &[u8]) -> Result<()> {
        let expected_len = self.width_bytes * self.height as usize;
        if bw.len() != expected_len {
            panic!(
                "Data length mismatch: expected {}, got {}",
                expected_len,
                bw.len()
            );
        }

        Command::WriteRamBW.execute(&mut self.interface)?;
        self.interface.send_data(bw)?;

        // SSD2683 always writes white to red RAM
        let white_data = vec![0xFFu8; expected_len];
        Command::WriteRamRed.execute(&mut self.interface)?;
        self.interface.send_data(&white_data)?;

        Ok(())
    }
}

impl EpdDisplay for Display {
    fn width(&self) -> usize {
        self.width_bytes * 8
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn init(&mut self) -> Result<()> {
        self.normal_init()
    }

    fn init_fast(&mut self, use_otp: bool) -> Result<()> {
        if use_otp {
            // SSD2683_Init_For_OTP from C++ reference: otp_init + normal_init
            self.otp_init()?;
            self.normal_init()
        } else {
            self.normal_init()
        }
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
        // Convert pixel coordinates to byte coordinates
        let x_bytes = (x / 8) as u8;
        let width_bytes = (width / 8) as u8;
        self.part_write(x_bytes, y, width_bytes, height, data)
    }

    fn update_partial(&mut self) -> Result<()> {
        // SSD2683 write_partial already includes power cycle, not applicable
        Err(anyhow!(
            "SSD2683 does not need update_partial - write_partial already includes refresh"
        ))
    }

    fn deep_sleep(&mut self) -> Result<()> {
        self.deep_sleep()
    }
}
