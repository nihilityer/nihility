use crate::display::epd::EpdInterface;
use anyhow::Result;

/// 填充数据缓冲区（数组）并返回一个包含命令和填充缓冲区中适当大小切片的元组。
macro_rules! pack {
    ($buf:ident, $cmd:expr,[]) => {
        ($cmd, &$buf[..0])
    };
    ($buf:ident, $cmd:expr,[$arg0:expr]) => {{
        $buf[0] = $arg0;
        ($cmd, &$buf[..1])
    }};
    ($buf:ident, $cmd:expr,[$arg0:expr, $arg1:expr]) => {{
        $buf[0] = $arg0;
        $buf[1] = $arg1;
        ($cmd, &$buf[..2])
    }};
    ($buf:ident, $cmd:expr,[$arg0:expr, $arg1:expr, $arg2:expr]) => {{
        $buf[0] = $arg0;
        $buf[1] = $arg1;
        $buf[2] = $arg2;
        ($cmd, &$buf[..3])
    }};
    ($buf:ident, $cmd:expr,[$arg0:expr, $arg1:expr, $arg2:expr, $arg3:expr]) => {{
        $buf[0] = $arg0;
        $buf[1] = $arg1;
        $buf[2] = $arg2;
        $buf[3] = $arg3;
        ($cmd, &$buf[..4])
    }};
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum IncrementAxis {
    /// X 方向
    Horizontal,
    /// Y 方向
    Vertical,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum DataEntryMode {
    DecrementXDecrementY,
    IncrementXDecrementY,
    DecrementXIncrementY,
    IncrementYIncrementX,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum DeepSleepMode {
    /// 非睡眠模式
    Normal,
    /// 深度睡眠并保留 RAM
    PreserveRAM,
    /// 深度睡眠不保留 RAM
    DiscardRAM,
}

#[allow(dead_code)]
/// 可以发送到 SSD2683 控制器的命令。
#[derive(Clone, Copy)]
pub enum Command {
    /// 0x00: 面板设置 (PST)
    /// 用于 OTP 初始化
    PanelSetting(u8, u8),
    /// 0x02: 电源关闭
    /// 关闭 DC/DC 电压转换器
    PowerOff,
    /// 0x04: 电源打开
    /// 打开 DC/DC 电压转换器
    PowerOn,
    /// 0x07: 深度睡眠模式
    DeepSleepMode(DeepSleepMode),
    /// 0x0A: 读取 DC_ID
    ReadDcId,
    /// 0x0D: 读取芯片状态
    ReadStatus,
    /// 0x10: 写入 DTM1 (黑白 RAM)
    WriteRamBW,
    /// 0x12: 显示刷新
    DisplayRefresh,
    /// 0x13: 写入 DTM2 (红色 RAM)
    WriteRamRed,
    /// 0x14: 读取 DTM1
    ReadRamBW,
    /// 0x1A: 写入温度传感器
    WriteTemperatureSensor(u8),
    /// 0x20: 写入边框波形
    BorderWaveform(u8),
    /// 0x22: 显示更新控制2
    DisplayUpdateControl2(u8),
    /// 0x24: 写入 RAM 保留
    WriteRamReserve,
    /// 0x26: 读取 RAM 保留
    ReadRamReserve,
    /// 0x37: 读取温度传感器
    ReadTemperatureSensor,
    /// 0x40: 设置 RAM X 地址起始位置
    XAddressInc(u8),
    /// 0x41: 设置 RAM X 地址
    XAddress(u8),
    /// 0x42: 设置 RAM Y 地址
    YAddress(u16),
    /// 0x44: 设置 X 方向窗口地址的起始和结束位置
    StartEndXPosition(u8, u8),
    /// 0x45: 设置 Y 方向窗口地址的起始和结束位置
    StartEndYPosition(u16, u16),
    /// 0x4E: 设置 RAM X 地址
    RamXAddress(u8),
    /// 0x4F: 设置 RAM Y 地址
    RamYAddress(u16),
    /// 0x50: CDI 边框设置
    CdiBorder(u8),
    /// 0x60: TCON 设置
    TconSetting(u8),
    /// 0x61: 目标温度设置
    TargetTemperature(u8),
    /// 0xE0: 温度写入
    TemperatureWrite(u8),
    /// 0xE3: 加载温度值
    LoadTemperature,
    /// 0xE5: 读取温度
    ReadTemperature,
    /// 0xE6: 温度 LUT 设置
    TemperatureLut(u8),
    /// 0xE9: OTP 初始化控制
    OtpInitControl,
    /// 0x7F: NOP
    Nop,
    /// 0x83: 设置局部刷新窗口
    /// HRST[9:8], HRST[7:2], HRED[9:8], HRED[7:2],
    /// VRST[9:8], VRST[7:2], VRED[9:8], VRED[7:2], trigger
    SetPartialWindow {
        hrst_h: u8,
        hrst_l: u8,
        hred_h: u8,
        hred_l: u8,
        vrst_h: u8,
        vrst_l: u8,
        vred_h: u8,
        vred_l: u8,
        trigger: u8,
    },
}

impl Command {
    pub fn execute(&self, interface: &mut EpdInterface) -> Result<()> {
        use self::Command::*;

        let mut buf = [0u8; 4];
        let (command, data) = match *self {
            PanelSetting(res0, res1) => pack!(buf, 0x00, [res0, res1]),
            PowerOff => pack!(buf, 0x02, []),
            PowerOn => pack!(buf, 0x04, []),
            DeepSleepMode(mode) => {
                let mode = match mode {
                    self::DeepSleepMode::Normal => 0b00,
                    self::DeepSleepMode::PreserveRAM => 0b01,
                    self::DeepSleepMode::DiscardRAM => 0b11,
                };
                pack!(buf, 0x07, [mode])
            }
            ReadDcId => pack!(buf, 0x0A, []),
            ReadStatus => pack!(buf, 0x0D, []),
            WriteRamBW => pack!(buf, 0x10, []),
            DisplayRefresh => pack!(buf, 0x12, []),
            WriteRamRed => pack!(buf, 0x13, []),
            ReadRamBW => pack!(buf, 0x14, []),
            WriteTemperatureSensor(data) => pack!(buf, 0x1A, [data]),
            BorderWaveform(border_waveform) => pack!(buf, 0x20, [border_waveform]),
            DisplayUpdateControl2(value) => pack!(buf, 0x22, [value]),
            WriteRamReserve => pack!(buf, 0x24, []),
            ReadRamReserve => pack!(buf, 0x26, []),
            ReadTemperatureSensor => pack!(buf, 0x37, []),
            XAddressInc(address) => pack!(buf, 0x40, [address]),
            XAddress(address) => pack!(buf, 0x41, [address]),
            YAddress(address) => {
                let [upper, lower] = address.to_be_bytes();
                pack!(buf, 0x42, [lower, upper])
            }
            StartEndXPosition(start, end) => pack!(buf, 0x44, [start, end]),
            StartEndYPosition(start, end) => {
                let [start_upper, start_lower] = start.to_be_bytes();
                let [end_upper, end_lower] = end.to_be_bytes();
                pack!(buf, 0x45, [start_lower, start_upper, end_lower, end_upper])
            }
            RamXAddress(address) => pack!(buf, 0x4E, [address]),
            RamYAddress(address) => {
                let [upper, lower] = address.to_be_bytes();
                pack!(buf, 0x4F, [lower, upper])
            }
            CdiBorder(cdi) => pack!(buf, 0x50, [cdi]),
            TconSetting(tcon) => pack!(buf, 0x60, [tcon]),
            TargetTemperature(temp) => pack!(buf, 0x61, [temp]),
            TemperatureWrite(data) => pack!(buf, 0xE0, [data]),
            LoadTemperature => pack!(buf, 0xE3, []),
            ReadTemperature => pack!(buf, 0xE5, []),
            TemperatureLut(temp) => pack!(buf, 0xE6, [temp]),
            OtpInitControl => pack!(buf, 0xE9, [0x01]),
            Nop => pack!(buf, 0x7F, []),
            SetPartialWindow {
                hrst_h,
                hrst_l,
                hred_h,
                hred_l,
                vrst_h,
                vrst_l,
                vred_h,
                vred_l,
                trigger,
            } => {
                // 0x83 命令需要 9 个数据字节，超出 pack! 宏支持范围
                interface.send_command(0x83)?;
                interface.send_data(&[
                    hrst_h, hrst_l, hred_h, hred_l, vrst_h, vrst_l, vred_h, vred_l, trigger,
                ])?;
                return Ok(());
            }
        };

        interface.send_command(command)?;
        if data.is_empty() {
            Ok(())
        } else {
            interface.send_data(data)
        }
    }
}
