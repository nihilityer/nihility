use crate::display::epd::EpdInterface;
use anyhow::Result;

/// 填充数据缓冲区（数组）并返回一个包含命令和填充缓冲区中适当大小切片的元组。
/// ```
/// let mut buf = [0u8; 4];
/// let (command, data) = pack!(buf, 0x3C, [0x12, 0x34]);
/// ```
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

#[derive(Clone, Copy)]
pub enum IncrementAxis {
    /// X 方向
    Horizontal,
    /// Y 方向
    Vertical,
}

#[derive(Clone, Copy)]
pub enum DataEntryMode {
    DecrementXDecrementY,
    IncrementXDecrementY,
    DecrementXIncrementY,
    IncrementYIncrementX,
}

#[derive(Clone, Copy)]
pub enum RamOption {
    Normal = 0x00,
    Bypass = 0x04,
    Invert = 0x08,
}

#[derive(Clone, Copy)]
pub enum DeepSleepMode {
    /// 非睡眠模式
    Normal,
    /// 深度睡眠并保留 RAM
    PreserveRAM,
    /// 深度睡眠不保留 RAM
    DiscardRAM,
}

/// 可以发送到控制器的命令。
#[derive(Clone, Copy)]
pub enum Command {
    /// 0x01: 设置栅极线的 MUX、扫描序列和方向
    /// 0: 最大栅极线数
    /// 1: 栅极扫描序列和方向
    DriverOutputControl(u16, u8),
    /// 0x03: 设置栅极驱动电压。
    GateDrivingVoltage(u8),
    /// 0x04: 设置源极驱动电压。
    SourceDrivingVoltage(u8, u8, u8),
    /// 0x0C: Booster Soft start Control
    BoosterSoftStart(u8, u8, u8, u8),
    /// 0x10: 设置深度睡眠模式
    DeepSleepMode(DeepSleepMode),
    /// 0x11: 设置数据输入模式和增量轴
    DataEntryMode(DataEntryMode, IncrementAxis),
    /// 0x12: 执行软复位，并将所有参数重置为默认值
    /// 执行期间 BUSY 信号将保持高电平。
    SoftReset,
    /// 0x18: 读取温度传感器寄存器
    /// a[7:0]: 0x48=外部温度传感器；0x80=内部温度传感器
    ReadTemperatureSensor(u8),
    /// 0x1A: 写入温度传感器寄存器
    WriteTemperatureSensor(u8),
    /// 0x20: 激活显示更新序列。执行期间 BUSY 信号将保持高电平。
    MasterActivation,
    /// 0x21: 显示更新控制1
    /// 7654 3210
    /// a[7:4]: 红色 RAM 选项
    /// a[3:0]: 黑/白 RAM 选项
    /// b[4]: 是否为单芯片应用
    DisplayUpdateControl1(u16),
    /// 0x22: 显示更新控制2
    DisplayUpdateControl2(u8),
    /// 0x24: 写入 黑白 RAM
    /// 发生这个指令后所有数据都有效，直到下一条发送下一条命令
    WriteRamBW,
    /// 0x26: 写入 红色 RAM
    /// 发生这个指令后所有数据都有效，直到下一条发送下一条命令
    /// 对于红色像素：写入 RAM(RED) 的内容 = 1；对于非红色像素（黑色或白色）：写入 RAM(RED) 的内容 = 0
    WriteRamRed,
    /// 0x2C: 写入 VCOM 寄存器
    WriteVCOM(u8),
    /// 0x3C: 为 VBD 选择边框波形
    BorderWaveform(u8),
    /// 0x44: 设置 X 方向窗口地址的起始和结束位置
    StartEndXPosition(u8, u8),
    /// 0x45: 设置 Y 方向窗口地址的起始和结束位置
    StartEndYPosition(u16, u16),
    /// 0x4E: 设置 RAM X 地址
    XAddress(u8),
    /// 0x4F: 设置 RAM Y 地址
    YAddress(u16),
    // 0x7F: 用于终止帧存储器读取
    Nop,
}

impl Command {
    pub fn execute(&self, interface: &mut EpdInterface) -> Result<()> {
        use self::Command::*;

        let mut buf = [0u8; 4];
        let (command, data) = match *self {
            DriverOutputControl(gate_lines, scanning_seq_and_dir) => {
                let [upper, lower] = gate_lines.to_be_bytes();
                pack!(buf, 0x01, [lower, upper, scanning_seq_and_dir])
            }
            GateDrivingVoltage(voltages) => pack!(buf, 0x03, [voltages]),
            SourceDrivingVoltage(vsh1, vsh2, vsl) => pack!(buf, 0x04, [vsh1, vsh2, vsl]),
            BoosterSoftStart(phase1, phase2, phase3, duration) => {
                pack!(buf, 0x0C, [phase1, phase2, phase3, duration])
            }
            DeepSleepMode(mode) => {
                let mode = match mode {
                    self::DeepSleepMode::Normal => 0b00,
                    self::DeepSleepMode::PreserveRAM => 0b01,
                    self::DeepSleepMode::DiscardRAM => 0b11,
                };

                pack!(buf, 0x10, [mode])
            }
            DataEntryMode(data_entry_mode, increment_axis) => {
                let mode = match data_entry_mode {
                    self::DataEntryMode::DecrementXDecrementY => 0b00,
                    self::DataEntryMode::IncrementXDecrementY => 0b01,
                    self::DataEntryMode::DecrementXIncrementY => 0b10,
                    self::DataEntryMode::IncrementYIncrementX => 0b11,
                };
                let axis = match increment_axis {
                    IncrementAxis::Horizontal => 0b000,
                    IncrementAxis::Vertical => 0b100,
                };
                pack!(buf, 0x11, [axis | mode])
            }
            SoftReset => pack!(buf, 0x12, []),
            ReadTemperatureSensor(data) => pack!(buf, 0x18, [data]),
            WriteTemperatureSensor(data) => {
                pack!(buf, 0x1A, [data])
            }
            MasterActivation => pack!(buf, 0x20, []),
            DisplayUpdateControl1(data) => {
                let [upper, lower] = data.to_be_bytes();
                pack!(buf, 0x21, [upper, lower])
            }
            DisplayUpdateControl2(value) => pack!(buf, 0x22, [value]),
            WriteRamBW => pack!(buf, 0x24, []),
            WriteRamRed => pack!(buf, 0x26, []),
            WriteVCOM(value) => pack!(buf, 0x2C, [value]),
            BorderWaveform(border_waveform) => pack!(buf, 0x3C, [border_waveform]),
            StartEndXPosition(start, end) => pack!(buf, 0x44, [start, end]),
            StartEndYPosition(start, end) => {
                let [start_upper, start_lower] = start.to_be_bytes();
                let [end_upper, end_lower] = end.to_be_bytes();
                pack!(buf, 0x45, [start_lower, start_upper, end_lower, end_upper])
            }
            XAddress(address) => pack!(buf, 0x4E, [address]),
            YAddress(address) => {
                let [upper, lower] = address.to_be_bytes();
                pack!(buf, 0x4F, [lower, upper])
            }
            Nop => pack!(buf, 0x7F, []),
        };

        interface.send_command(command)?;
        if data.is_empty() {
            Ok(())
        } else {
            interface.send_data(data)
        }
    }
}
