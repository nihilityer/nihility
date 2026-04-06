//! ES8311 音频编解码器驱动
//!
//! 参考: esp-adf components/esp_codec_dev
mod reg;

use crate::audio::es8311::reg::{ClockCoeff, I2sFormat, PaSetting, Reg, Resolution, WorkMode};
use anyhow::Result;
use core::fmt;
use embedded_hal::i2c::I2c as _;
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::i2c::master::I2c;

/// ES8311 I2C 从机地址
pub const ES8311_ADDR: u8 = 0x18;

/// ES8311 配置结构
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Es8311Config {
    /// 工作模式
    pub mode: WorkMode,
    /// 主/从模式 (true: 主模式, false: 从模式)
    pub master_mode: bool,
    /// 是否使用 MCLK (true: 使用外部 MCLK, false: 使用内部时钟)
    pub use_mclk: bool,
    /// 是否反转 MCLK 时钟
    pub invert_mclk: bool,
    /// 是否反转 SCLK 时钟
    pub invert_sclk: bool,
    /// MCLK 分频系数
    pub mclk_div: u32,
    /// PA 引脚号 (-1 表示不使用)
    pub pa_pin: i32,
    /// 是否反转 PA 引脚
    pub pa_reverted: bool,
    /// 数字麦克风模式
    pub digital_mic: bool,
    /// 是否使用 DAC 参考
    pub no_dac_ref: bool,
    /// 硬件增益
    pub hw_gain: f32,
}

impl Default for Es8311Config {
    fn default() -> Self {
        Self {
            mode: WorkMode::Both,
            master_mode: false,
            use_mclk: true,
            invert_mclk: false,
            invert_sclk: false,
            mclk_div: 256,
            pa_pin: -1,
            pa_reverted: false,
            digital_mic: false,
            no_dac_ref: false,
            hw_gain: 0.0,
        }
    }
}

/// ES8311 音频编解码器实例
pub struct Es8311 {
    /// I2C 总线接口
    i2c: RefCellDevice<'static, I2c<'static, esp_hal::Blocking>>,
    /// 配置
    config: Es8311Config,
    /// 是否已打开
    is_open: bool,
    /// 是否已使能
    enabled: bool,
    /// 硬件增益
    hw_gain: f32,
    /// 配对状态 (用于 ADC/DAC 配对管理)
    is_paired: bool,
}

impl Es8311 {
    /// 创建新的 ES8311 实例
    pub fn new(
        i2c: RefCellDevice<'static, I2c<'static, esp_hal::Blocking>>,
        config: Es8311Config,
    ) -> Self {
        Self {
            i2c,
            config,
            is_open: false,
            enabled: false,
            hw_gain: config.hw_gain,
            is_paired: false,
        }
    }

    /// 获取 I2C 总线接口的可变引用
    pub fn i2c_mut(&mut self) -> &mut RefCellDevice<'static, I2c<'static, esp_hal::Blocking>> {
        &mut self.i2c
    }

    /// 获取配置
    pub fn config(&self) -> &Es8311Config {
        &self.config
    }

    /// 检查是否已打开
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// 检查是否已使能
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 写入寄存器
    ///
    /// # Arguments
    /// * `reg` - 寄存器地址
    /// * `value` - 寄存器值
    pub fn write_reg(&mut self, reg: Reg, value: u8) -> Result<()> {
        let addr = ES8311_ADDR;
        let reg_addr = reg.addr();

        // I2C 写操作: [设备地址(写), 寄存器地址, 寄存器值]
        let data = [reg_addr, value];
        self.i2c
            .write(addr, &data)
            .map_err(|e| anyhow::anyhow!("I2C write error: {}", e))?;

        Ok(())
    }

    /// 读取寄存器
    ///
    /// # Arguments
    /// * `reg` - 寄存器地址
    ///
    /// # Returns
    /// 寄存器值
    pub fn read_reg(&mut self, reg: Reg) -> Result<u8> {
        let addr = ES8311_ADDR;
        let reg_addr = reg.addr();

        // I2C 读操作: 先写寄存器地址，再读数据
        let mut buf = [0u8];
        self.i2c
            .write_read(addr, &[reg_addr], &mut buf)
            .map_err(|e| anyhow::anyhow!("I2C read error: {}", e))?;

        Ok(buf[0])
    }

    /// 配置 I2S 数据格式
    ///
    /// # Arguments
    /// * `format` - I2S 数据格式
    pub fn config_format(&mut self, format: I2sFormat) -> Result<()> {
        // 配置 ADC/I2S 输出格式 (寄存器 0x09)
        let sdpout = self.read_reg(Reg::Sdpin)?;
        let sdpout = (sdpout & 0xF3) | ((format as u8) << 2); // Bits[3:2] = format
        self.write_reg(Reg::Sdpin, sdpout)?;

        // 配置 DAC/I2S 输入格式 (寄存器 0x0A)
        let sdpout = self.read_reg(Reg::Sdpout)?;
        let sdpout = (sdpout & 0xF3) | ((format as u8) << 2); // Bits[3:2] = format
        self.write_reg(Reg::Sdpout, sdpout)?;

        Ok(())
    }

    /// 设置采样位深
    ///
    /// # Arguments
    /// * `bits` - 位深 (16, 18, 20, 24, 32)
    pub fn set_bits_per_sample(&mut self, bits: Resolution) -> Result<()> {
        // 配置 ADC 分辨率 (寄存器 0x09)
        let sdpin = self.read_reg(Reg::Sdpin)?;
        let sdpin = match bits {
            Resolution::Bit16 | Resolution::Bit18 | Resolution::Bit20 => {
                sdpin | 0x0c // Bits[3:2] = 3
            }
            Resolution::Bit24 => sdpin & !0x1c, // 清除 bits[4:2]
            Resolution::Bit32 => sdpin | 0x10,  // Bit[4] = 1
        };
        self.write_reg(Reg::Sdpin, sdpin)?;

        // 配置 DAC 分辨率 (寄存器 0x0A)
        let sdpout = self.read_reg(Reg::Sdpout)?;
        let sdpout = match bits {
            Resolution::Bit16 | Resolution::Bit18 | Resolution::Bit20 => {
                sdpout | 0x0c // Bits[3:2] = 3
            }
            Resolution::Bit24 => sdpout & !0x1c, // 清除 bits[4:2]
            Resolution::Bit32 => sdpout | 0x10,  // Bit[4] = 1
        };
        self.write_reg(Reg::Sdpout, sdpout)?;

        Ok(())
    }

    /// 根据 MCLK 和采样率获取时钟分频系数
    ///
    /// # Arguments
    /// * `mclk` - MCLK 时钟频率 (Hz)
    /// * `rate` - 采样率 (Hz)
    ///
    /// # Returns
    /// 时钟分频系数，失败返回 None
    pub fn get_coeff(&self, mclk: u32, rate: u32) -> Option<ClockCoeff> {
        use reg::COEFF_DIV;

        COEFF_DIV
            .iter()
            .find(|c| c.mclk == mclk && c.rate == rate)
            .copied()
    }

    /// 挂起编解码器
    pub fn suspend(&mut self) -> Result<()> {
        // 取消配对
        self.is_paired = false;

        // 如果有其他编解码器在使用，不完全挂起
        // 写入关键寄存器进入低功耗模式
        self.write_reg(Reg::Dac32, 0x00)?; // DAC 音量设为 0
        self.write_reg(Reg::Adc17, 0x00)?; // ADC 寄存器
        self.write_reg(Reg::SystemE, 0xFF)?; // 系统复位
        self.write_reg(Reg::System12, 0x02)?; // 系统寄存器
        self.write_reg(Reg::System14, 0x00)?; // 系统寄存器
        self.write_reg(Reg::SystemD, 0xFA)?; // 系统寄存器 D
        self.write_reg(Reg::Adc15, 0x00)?; // ADC 寄存器
        self.write_reg(Reg::ClkManager2, 0x10)?; // 时钟管理器
        self.write_reg(Reg::Reset, 0x00)?; // 复位
        self.write_reg(Reg::Reset, 0x1F)?; // 复位
        self.write_reg(Reg::ClkManager1, 0x30)?; // 时钟管理器
        self.write_reg(Reg::ClkManager1, 0x00)?; // 关闭时钟
        self.write_reg(Reg::Gp45, 0x00)?; // GPIO
        self.write_reg(Reg::SystemD, 0xFC)?; // 系统寄存器 D
        self.write_reg(Reg::ClkManager2, 0x00)?; // 关闭时钟管理器

        self.enabled = false;
        Ok(())
    }

    /// 启动编解码器
    pub fn start(&mut self) -> Result<()> {
        let mut regv = 0x80;

        // Master 模式配置
        if self.config.master_mode {
            regv |= 0x40;
        } else {
            regv &= 0xBF;
        }
        self.write_reg(Reg::Reset, regv)?;

        // 配置时钟管理器 1 - MCLK 源选择
        regv = 0x3F;
        if self.config.use_mclk {
            regv &= 0x7F;
        } else {
            regv |= 0x80;
        }
        if self.config.invert_mclk {
            regv |= 0x40;
        } else {
            regv &= !0x40;
        }
        self.write_reg(Reg::ClkManager1, regv)?;

        // 读取并配置 ADC/DAC 接口寄存器
        let mut dac_iface = self.read_reg(Reg::Sdpin)?;
        let mut adc_iface = self.read_reg(Reg::Sdpout)?;
        dac_iface &= 0xBF;
        adc_iface &= 0xBF;

        // 根据工作模式配置接口
        let codec_mode = match self.config.mode {
            WorkMode::Adc => 0x01,
            WorkMode::Dac => 0x02,
            WorkMode::Both => 0x03,
            WorkMode::Line => 0x04,
        };

        if codec_mode == 0x01 || codec_mode == 0x03 {
            // ADC 或 Both 模式
            adc_iface &= !0x40; // bit6 = 0
        }
        if codec_mode == 0x02 || codec_mode == 0x03 {
            // DAC 或 Both 模式
            dac_iface &= !0x40; // bit6 = 0
        }

        self.write_reg(Reg::Sdpin, dac_iface)?;
        self.write_reg(Reg::Sdpout, adc_iface)?;

        // ADC 配置
        self.write_reg(Reg::Adc17, 0xBF)?;
        self.write_reg(Reg::SystemE, 0x02)?;

        // DAC 配置 (如果是 DAC 或 Both 模式)
        if codec_mode == 0x02 || codec_mode == 0x03 {
            self.write_reg(Reg::System12, 0x00)?;
        }

        self.write_reg(Reg::System14, 0x1A)?;

        // DMIC 配置
        regv = self.read_reg(Reg::System14)?;
        if self.config.digital_mic {
            regv |= 0x40;
        } else {
            regv &= !0x40;
        }
        self.write_reg(Reg::System14, regv)?;

        self.write_reg(Reg::SystemD, 0x01)?;
        self.write_reg(Reg::Adc15, 0x40)?;
        self.write_reg(Reg::Dac37, 0x08)?;
        self.write_reg(Reg::Gp45, 0x00)?;

        // 标记为已配对
        self.is_paired = true;

        self.enabled = true;
        Ok(())
    }

    /// 设置静音
    ///
    /// # Arguments
    /// * `mute` - true: 静音, false: 取消静音
    pub fn set_mute(&mut self, mute: bool) -> Result<()> {
        let dac31 = self.read_reg(Reg::Dac31)?;
        let dac31 = if mute {
            dac31 | 0x20 // Bit[5] = 1, 静音
        } else {
            dac31 & !0x20 // Bit[5] = 0, 取消静音
        };
        self.write_reg(Reg::Dac31, dac31)?;
        Ok(())
    }

    /// 设置音量
    ///
    /// # Arguments
    /// * `db_value` - 音量值 (dB)
    pub fn set_volume(&mut self, db_value: f32) -> Result<()> {
        // ES8311 音量寄存器范围: 0x00 (最小, -95.5dB) ~ 0xFF (最大, +32dB)
        // 音量曲线接近线性，在 0 ~ 0xD0 范围内
        const VOLUME_MIN_DB: f32 = -95.5;
        const VOLUME_MAX_DB: f32 = 32.0;

        // 限制范围
        let db = db_value.clamp(VOLUME_MIN_DB, VOLUME_MAX_DB);

        // 转换为寄存器值
        // 在 -30dB ~ 0dB 范围内近似线性
        let vol: u8 = if db <= -30.0 {
            let ratio = (db - VOLUME_MIN_DB) / (-30.0 - VOLUME_MIN_DB);
            (ratio * 96.0) as u8
        } else if db <= 0.0 {
            let ratio = (db + 30.0) / 30.0;
            0x60 + (ratio * 32.0) as u8
        } else {
            let ratio = db / VOLUME_MAX_DB;
            0x80 + (ratio * 127.0) as u8
        };

        // 设置 DAC 音量 (寄存器 0x32)
        self.write_reg(Reg::Dac32, vol)?;
        Ok(())
    }

    /// 设置麦克风增益
    ///
    /// # Arguments
    /// * `db` - 增益值 (dB)
    pub fn set_mic_gain(&mut self, db: f32) -> Result<()> {
        use crate::audio::es8311::reg::MicGain;

        let gain = MicGain::from_db(db);
        let gain_val = gain as u8;

        // 设置 ADC 麦克风增益 (寄存器 0x16)
        let adc16 = self.read_reg(Reg::Adc16)?;
        let adc16 = (adc16 & 0xF8) | (gain_val & 0x07); // Bits[2:0] = gain
        self.write_reg(Reg::Adc16, adc16)?;
        Ok(())
    }

    /// 配置 PA 功放
    ///
    /// # Arguments
    /// * `setting` - PA 设置
    fn pa_power(&mut self, setting: PaSetting) {
        match setting {
            PaSetting::Setup => {
                // PA 设置模式 - 配置 GPIO
                let gpio44 = self.read_reg(Reg::Gpio44).unwrap_or(0);
                let gpio44 = gpio44 & !0x10; // Bit[4] = 0, 配置为输出
                let _ = self.write_reg(Reg::Gpio44, gpio44);
            }
            PaSetting::Enable => {
                // PA 使能 - 设置 GPIO 高
                let gp45 = self.read_reg(Reg::Gp45).unwrap_or(0);
                let gp45 = gp45 | 0x10; // Bit[4] = 1, 输出高
                let _ = self.write_reg(Reg::Gp45, gp45);
            }
            PaSetting::Disable => {
                // PA 禁用 - 设置 GPIO 低
                let gp45 = self.read_reg(Reg::Gp45).unwrap_or(0);
                let gp45 = gp45 & !0x10; // Bit[4] = 0, 输出低
                let _ = self.write_reg(Reg::Gp45, gp45);
            }
        }
    }

    /// 配置采样率
    ///
    /// # Arguments
    /// * `sample_rate` - 采样率 (Hz)
    pub fn config_sample_rate(&mut self, sample_rate: u32) -> Result<()> {
        // 计算 MCLK 频率
        let mclk = sample_rate * self.config.mclk_div;

        // 获取时钟系数
        let coeff = self.get_coeff(mclk, sample_rate).ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to get clock coefficient for sample rate {}Hz with MCLK {}Hz",
                sample_rate,
                mclk
            )
        })?;

        // 配置时钟管理器 2 - 前置分频和倍频
        let mut regv = self.read_reg(Reg::ClkManager2)?;
        regv &= 0x7;
        regv |= (coeff.pre_div - 1) << 5;

        // 计算 pre_multi 值
        let datmp = match coeff.pre_multi {
            1 => 0,
            2 => 1,
            4 => 2,
            8 => 3,
            _ => 0,
        };

        // 如果不使用外部 MCLK，使用内部时钟
        if !self.config.use_mclk {
            // 内部 MCLK 配置
            let datmp = if sample_rate == 8000 {
                2 // pre_multi = 4
            } else {
                3 // pre_multi = 8
            };
            regv |= datmp << 3; // 使用 |= 保留之前设置的 pre_div
        } else {
            regv |= datmp << 3;
        }
        self.write_reg(Reg::ClkManager2, regv)?;

        // 配置时钟管理器 5 - ADC/DAC 分频
        regv = 0x00;
        regv |= (coeff.adc_div - 1) << 4;
        regv |= (coeff.dac_div - 1) << 0;
        self.write_reg(Reg::ClkManager5, regv)?;

        // 配置时钟管理器 3 - ADC OSR 和速度模式
        regv = self.read_reg(Reg::ClkManager3)?;
        regv &= 0x80;
        regv |= coeff.fs_mode << 6;
        regv |= coeff.adc_osr << 0;
        self.write_reg(Reg::ClkManager3, regv)?;

        // 配置时钟管理器 4 - DAC OSR
        regv = self.read_reg(Reg::ClkManager4)?;
        regv &= 0x80;
        regv |= coeff.dac_osr << 0;
        self.write_reg(Reg::ClkManager4, regv)?;

        // 配置时钟管理器 7 - LRCK 高位
        regv = self.read_reg(Reg::ClkManager7)?;
        regv &= 0xC0;
        regv |= coeff.lrck_h << 0;
        self.write_reg(Reg::ClkManager7, regv)?;

        // 配置时钟管理器 8 - LRCK 低位
        regv = 0x00;
        regv |= coeff.lrck_l << 0;
        self.write_reg(Reg::ClkManager8, regv)?;

        // 配置时钟管理器 6 - BCLK 分频
        regv = self.read_reg(Reg::ClkManager6)?;
        regv &= 0xE0;
        if coeff.bclk_div < 19 {
            regv |= (coeff.bclk_div - 1) << 0;
        } else {
            regv |= coeff.bclk_div << 0;
        }
        self.write_reg(Reg::ClkManager6, regv)?;

        Ok(())
    }

    /// 打开编解码器
    pub fn open(&mut self) -> Result<()> {
        // 配置 PA (Setup 和 Disable)
        self.pa_power(PaSetting::Setup);
        self.pa_power(PaSetting::Disable);

        // 验证并配置系统寄存器 D (0x0D)
        let regv = self.read_reg(Reg::SystemD)?;
        if regv != 0xFA {
            self.write_reg(Reg::SystemD, 0xFA)?;
        }

        // 增强 ES8311 I2C 抗干扰能力 - 两次写入确保成功
        self.write_reg(Reg::Gpio44, 0x08)?;
        self.write_reg(Reg::Gpio44, 0x08)?;

        // 初始化时钟管理器
        self.write_reg(Reg::ClkManager1, 0x30)?;
        self.write_reg(Reg::ClkManager2, 0x00)?;
        self.write_reg(Reg::ClkManager3, 0x10)?;

        // 初始化 ADC
        self.write_reg(Reg::Adc16, 0x24)?;

        self.write_reg(Reg::ClkManager4, 0x10)?;
        self.write_reg(Reg::ClkManager5, 0x00)?;
        self.write_reg(Reg::SystemB, 0x00)?;
        self.write_reg(Reg::SystemC, 0x00)?;
        self.write_reg(Reg::System10, 0x1F)?;
        self.write_reg(Reg::System11, 0x7F)?;

        // 软复位并配置为 master/slave 模式
        let mut regv = 0x80;
        if self.config.master_mode {
            regv |= 0x40; // Master 模式
        } else {
            regv &= 0xBF; // Slave 模式
        }
        self.write_reg(Reg::Reset, regv)?;

        // 读取复位状态
        regv = self.read_reg(Reg::Reset)?;

        // 选择内部 MCLK 时钟源
        regv = 0x3F;
        if self.config.use_mclk {
            regv &= 0x7F; // 使用外部 MCLK
        } else {
            regv |= 0x80; // 使用内部 MCLK
        }
        if self.config.invert_mclk {
            regv |= 0x40; // 反转 MCLK
        } else {
            regv &= !0x40;
        }
        self.write_reg(Reg::ClkManager1, regv)?;

        // 配置 SCLK 反转
        let mut reg6 = self.read_reg(Reg::ClkManager6)?;
        if self.config.invert_sclk {
            reg6 |= 0x20; // 反转 SCLK
        } else {
            reg6 &= !0x20;
        }
        self.write_reg(Reg::ClkManager6, reg6)?;

        // 额外的系统配置
        self.write_reg(Reg::System13, 0x10)?;
        self.write_reg(Reg::Adc1b, 0x0A)?;
        self.write_reg(Reg::Adc1c, 0x6A)?;

        // 配置 DAC 参考电压
        if !self.config.no_dac_ref {
            // 设置内部参考信号 (ADCL + DACR)
            self.write_reg(Reg::Gpio44, 0x58)?;
        } else {
            self.write_reg(Reg::Gpio44, 0x08)?;
        }

        // 配置 I2S 格式 (默认 Normal I2S)
        self.config_format(I2sFormat::Normal)?;

        // 配置分辨率 (默认 16-bit)
        self.set_bits_per_sample(Resolution::Bit16)?;

        // 配置采样率 (默认 16kHz)
        self.config_sample_rate(16000)?;

        self.is_open = true;
        Ok(())
    }

    /// 关闭编解码器
    pub fn close(&mut self) -> Result<()> {
        // 静音
        self.set_mute(true)?;

        // 禁用 PA
        if self.config.pa_pin >= 0 {
            self.pa_power(PaSetting::Disable);
        }

        // 关闭时钟
        self.write_reg(Reg::ClkManager1, 0x00)?;

        // 复位
        self.write_reg(Reg::Reset, 0xFF)?;

        self.is_open = false;
        self.enabled = false;
        Ok(())
    }

    /// 设置采样格式
    ///
    /// # Arguments
    /// * `_sample_rate` - 采样率 (Hz)
    /// * `bits_per_sample` - 位深
    pub fn set_sample_format(
        &mut self,
        _sample_rate: u32,
        bits_per_sample: Resolution,
    ) -> Result<()> {
        // 配置采样位深
        self.set_bits_per_sample(bits_per_sample)?;

        // 配置 I2S 格式
        self.config_format(I2sFormat::Normal)?;

        Ok(())
    }

    /// 使能/禁用编解码器
    ///
    /// # Arguments
    /// * `enable` - true: 使能, false: 禁用
    pub fn enable(&mut self, enable: bool) -> Result<()> {
        if enable {
            // 使能编解码器
            // 配置系统寄存器 D
            let system_d = self.read_reg(Reg::SystemD)?;
            let system_d = system_d | 0x01; // Bit[0] = 1, 使能
            self.write_reg(Reg::SystemD, system_d)?;

            // 使能 PA
            if self.config.pa_pin >= 0 {
                self.pa_power(PaSetting::Enable);
            }

            // 取消静音
            self.set_mute(false)?;

            self.enabled = true;
        } else {
            // 静音
            self.set_mute(true)?;

            // 禁用 PA
            if self.config.pa_pin >= 0 {
                self.pa_power(PaSetting::Disable);
            }

            // 挂起编解码器
            self.suspend()?;
        }
        Ok(())
    }

    /// 直接设置寄存器
    ///
    /// # Arguments
    /// * `reg` - 寄存器地址
    /// * `value` - 寄存器值
    pub fn set_reg(&mut self, reg: u8, value: u8) -> Result<()> {
        let addr = ES8311_ADDR;

        // I2C 写操作: [设备地址(写), 寄存器地址, 寄存器值]
        let data = [reg, value];
        self.i2c
            .write(addr, &data)
            .map_err(|e| anyhow::anyhow!("I2C write error: {}", e))?;

        Ok(())
    }

    /// 直接读取寄存器
    ///
    /// # Arguments
    /// * `reg` - 寄存器地址
    ///
    /// # Returns
    /// 寄存器值
    pub fn get_reg(&mut self, reg: u8) -> Result<u8> {
        let addr = ES8311_ADDR;

        // I2C 读操作: 先写寄存器地址，再读数据
        let mut buf = [0u8];
        self.i2c
            .write_read(addr, &[reg], &mut buf)
            .map_err(|e| anyhow::anyhow!("I2C read error: {}", e))?;

        Ok(buf[0])
    }

    /// 打印所有寄存器值 (调试用)
    pub fn dump_registers(&mut self) -> Result<()> {
        use reg::ES8311_MAX_REGISTER;

        // 遍历所有寄存器并读取
        for reg_addr in 0..=ES8311_MAX_REGISTER {
            let _value = self.get_reg(reg_addr)?;
            // 使用日志输出 (如果可用)
            // log::info!("Register 0x{:02X}: 0x{:02X}", reg_addr, value);
        }
        Ok(())
    }

    /// 释放 I2C 总线接口
    pub fn release(self) -> RefCellDevice<'static, I2c<'static, esp_hal::Blocking>> {
        self.i2c
    }
}

impl fmt::Debug for Es8311 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Es8311")
            .field("config", &self.config)
            .field("is_open", &self.is_open)
            .field("enabled", &self.enabled)
            .finish()
    }
}
