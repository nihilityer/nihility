//! ES8311 音频编解码器寄存器定义
//!
//! 参考: esp-adf components/esp_codec_dev

/// ES8311 寄存器地址
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Reg {
    /// 复位寄存器
    Reset = 0x00,
    /// 时钟管理器寄存器 1
    ClkManager1 = 0x01,
    /// 时钟管理器寄存器 2
    ClkManager2 = 0x02,
    /// 时钟管理器寄存器 3
    ClkManager3 = 0x03,
    /// 时钟管理器寄存器 4
    ClkManager4 = 0x04,
    /// 时钟管理器寄存器 5
    ClkManager5 = 0x05,
    /// 时钟管理器寄存器 6
    ClkManager6 = 0x06,
    /// 时钟管理器寄存器 7
    ClkManager7 = 0x07,
    /// 时钟管理器寄存器 8
    ClkManager8 = 0x08,
    /// ADC/I2S 数据输入格式寄存器
    Sdpin = 0x09,
    /// ADC/I2S 数据输出格式寄存器
    Sdpout = 0x0A,
    /// 系统寄存器 B
    SystemB = 0x0B,
    /// 系统寄存器 C
    SystemC = 0x0C,
    /// 系统寄存器 D
    SystemD = 0x0D,
    /// 系统寄存器 E
    SystemE = 0x0E,
    /// 系统寄存器 10
    System10 = 0x10,
    /// 系统寄存器 11
    System11 = 0x11,
    /// 系统寄存器 12
    System12 = 0x12,
    /// 系统寄存器 13
    System13 = 0x13,
    /// 系统寄存器 14
    System14 = 0x14,
    /// ADC 寄存器 15
    Adc15 = 0x15,
    /// ADC 寄存器 16 (麦克风增益)
    Adc16 = 0x16,
    /// ADC 寄存器 17
    Adc17 = 0x17,
    /// ADC 寄存器 1B
    Adc1b = 0x1B,
    /// ADC 寄存器 1C
    Adc1c = 0x1C,
    /// DAC 寄存器 31
    Dac31 = 0x31,
    /// DAC 寄存器 32 (音量)
    Dac32 = 0x32,
    /// DAC 寄存器 37
    Dac37 = 0x37,
    /// GPIO 寄存器 44
    Gpio44 = 0x44,
    /// 通用 GPIO 寄存器 45
    Gp45 = 0x45,
}

impl Reg {
    /// 将寄存器转换为 u8 地址
    pub fn addr(self) -> u8 {
        self as u8
    }
}

/// 麦克风增益级别
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum MicGain {
    /// 0 dB 增益 (默认)
    Gain0Db = 0x00,
    /// 6 dB 增益
    Gain6Db = 0x01,
    /// 12 dB 增益
    Gain12Db = 0x02,
    /// 18 dB 增益
    Gain18Db = 0x03,
    /// 24 dB 增益
    Gain24Db = 0x04,
    /// 30 dB 增益
    Gain30Db = 0x05,
    /// 36 dB 增益
    Gain36Db = 0x06,
    /// 42 dB 增益
    Gain42Db = 0x07,
}

impl MicGain {
    /// 从 dB 值获取对应的增益级别
    pub fn from_db(db: f32) -> Self {
        if db < 3.0 {
            MicGain::Gain0Db
        } else if db < 9.0 {
            MicGain::Gain6Db
        } else if db < 15.0 {
            MicGain::Gain12Db
        } else if db < 21.0 {
            MicGain::Gain18Db
        } else if db < 27.0 {
            MicGain::Gain24Db
        } else if db < 33.0 {
            MicGain::Gain30Db
        } else if db < 39.0 {
            MicGain::Gain36Db
        } else {
            MicGain::Gain42Db
        }
    }

    /// 获取增益的 dB 值
    pub fn to_db(self) -> f32 {
        match self {
            MicGain::Gain0Db => 0.0,
            MicGain::Gain6Db => 6.0,
            MicGain::Gain12Db => 12.0,
            MicGain::Gain18Db => 18.0,
            MicGain::Gain24Db => 24.0,
            MicGain::Gain30Db => 30.0,
            MicGain::Gain36Db => 36.0,
            MicGain::Gain42Db => 42.0,
        }
    }
}

/// 音频位深分辨率
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Resolution {
    /// 16 位
    Bit16 = 16,
    /// 18 位
    Bit18 = 18,
    /// 20 位
    Bit20 = 20,
    /// 24 位
    Bit24 = 24,
    /// 32 位
    Bit32 = 32,
}

/// I2S 数据格式
///
/// 注意: Left 和 Right 在寄存器中共享相同的值 (0x01)
/// 这是 ES8311 硬件层面的设计
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum I2sFormat {
    /// 标准 I2S 格式
    Normal = 0x00,
    /// 左对齐/右对齐格式 (DSP mode = 0)
    LeftRight = 0x01,
    /// DSP 格式 (PCM)
    Dsp = 0x03,
}

/// PA 功放控制设置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PaSetting {
    /// 设置 PA 引脚
    Setup = 0x01,
    /// 使能 PA
    Enable = 0x02,
    /// 关闭 PA
    Disable = 0x04,
}

/// 编解码器工作模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WorkMode {
    /// ADC 模式 (录音)
    Adc = 0x01,
    /// DAC 模式 (播放)
    Dac = 0x02,
    /// ADC + DAC 模式 (双工)
    Both = 0x03,
    /// 线路输入模式 (不支持)
    Line = 0x04,
}

/// 时钟分频系数结构
///
/// 用于配置 ES8311 的时钟分频，以支持不同的采样率
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockCoeff {
    /// MCLK 时钟频率 (Hz)
    pub mclk: u32,
    /// 目标采样率 (Hz)
    pub rate: u32,
    /// 前置分频器 (1-8)
    pub pre_div: u8,
    /// 前置倍频器 (1, 2, 4, 8)
    pub pre_multi: u8,
    /// ADC 时钟分频
    pub adc_div: u8,
    /// DAC 时钟分频
    pub dac_div: u8,
    /// 速度模式: 0 = 单倍速, 1 = 双倍速
    pub fs_mode: u8,
    /// LRCK 分频器高位
    pub lrck_h: u8,
    /// LRCK 分频器低位
    pub lrck_l: u8,
    /// BCLK 时钟分频
    pub bclk_div: u8,
    /// ADC 过采样率
    pub adc_osr: u8,
    /// DAC 过采样率
    pub dac_osr: u8,
}

/// 音量范围结构
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumeRange {
    /// 最小音量寄存器值
    pub min_vol: u8,
    /// 最小音量对应的 dB 值
    pub min_db: f32,
    /// 最大音量寄存器值
    pub max_vol: u8,
    /// 最大音量对应的 dB 值
    pub max_db: f32,
}

impl Default for VolumeRange {
    fn default() -> Self {
        Self {
            min_vol: 0x00,
            min_db: -95.5,
            max_vol: 0xFF,
            max_db: 32.0,
        }
    }
}

/// MCLK 分频默认值
pub const MCLK_DEFAULT_DIV: u32 = 256;

/// ES8311 最大寄存器数量
pub const ES8311_MAX_REGISTER: u8 = 0x46;

/// 时钟系数表 - 基于 ESP-ADF esp_codec_dev
///
/// 格式: {mclk, rate, pre_div, pre_multi, adc_div, dac_div, fs_mode, lrck_h, lrck_l, bclk_div, adc_osr, dac_osr}
pub const COEFF_DIV: &[ClockCoeff] = &[
    // 8k
    ClockCoeff { mclk: 12288000, rate: 8000, pre_div: 0x06, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 18432000, rate: 8000, pre_div: 0x03, pre_multi: 0x02, adc_div: 0x03, dac_div: 0x03, fs_mode: 0x00, lrck_h: 0x05, lrck_l: 0xff, bclk_div: 0x18, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 16384000, rate: 8000, pre_div: 0x08, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 8192000, rate: 8000, pre_div: 0x04, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 6144000, rate: 8000, pre_div: 0x03, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 4096000, rate: 8000, pre_div: 0x02, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 3072000, rate: 8000, pre_div: 0x01, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 2048000, rate: 8000, pre_div: 0x01, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 1536000, rate: 8000, pre_div: 0x03, pre_multi: 0x04, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 1024000, rate: 8000, pre_div: 0x01, pre_multi: 0x02, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    // 16k
    ClockCoeff { mclk: 12288000, rate: 16000, pre_div: 0x03, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 18432000, rate: 16000, pre_div: 0x03, pre_multi: 0x02, adc_div: 0x03, dac_div: 0x03, fs_mode: 0x00, lrck_h: 0x02, lrck_l: 0xff, bclk_div: 0x0c, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 16384000, rate: 16000, pre_div: 0x04, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 8192000, rate: 16000, pre_div: 0x02, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 6144000, rate: 16000, pre_div: 0x03, pre_multi: 0x02, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 4096000, rate: 16000, pre_div: 0x01, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 3072000, rate: 16000, pre_div: 0x03, pre_multi: 0x04, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 2048000, rate: 16000, pre_div: 0x01, pre_multi: 0x02, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 1536000, rate: 16000, pre_div: 0x03, pre_multi: 0x08, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    ClockCoeff { mclk: 1024000, rate: 16000, pre_div: 0x01, pre_multi: 0x04, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x20 },
    // 44.1k
    ClockCoeff { mclk: 11289600, rate: 44100, pre_div: 0x01, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x10 },
    ClockCoeff { mclk: 5644800, rate: 44100, pre_div: 0x01, pre_multi: 0x02, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x10 },
    ClockCoeff { mclk: 2822400, rate: 44100, pre_div: 0x01, pre_multi: 0x04, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x10 },
    ClockCoeff { mclk: 1411200, rate: 44100, pre_div: 0x01, pre_multi: 0x08, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x10 },
    // 48k
    ClockCoeff { mclk: 12288000, rate: 48000, pre_div: 0x01, pre_multi: 0x01, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x10 },
    ClockCoeff { mclk: 18432000, rate: 48000, pre_div: 0x03, pre_multi: 0x02, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x10 },
    ClockCoeff { mclk: 6144000, rate: 48000, pre_div: 0x01, pre_multi: 0x02, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x10 },
    ClockCoeff { mclk: 3072000, rate: 48000, pre_div: 0x01, pre_multi: 0x04, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x10 },
    ClockCoeff { mclk: 1536000, rate: 48000, pre_div: 0x01, pre_multi: 0x08, adc_div: 0x01, dac_div: 0x01, fs_mode: 0x00, lrck_h: 0x00, lrck_l: 0xff, bclk_div: 0x04, adc_osr: 0x10, dac_osr: 0x10 },
];