use anyhow::anyhow;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiDevice;
use log::debug;

const RESET_DELAY_MS: u32 = 10;

pub struct Interface<SPI: SpiDevice, BUSY: InputPin, RESET: OutputPin, DC: OutputPin> {
    /// SPI 接口
    spi: SPI,
    /// Active low busy pin (input)
    busy: BUSY,
    /// Pin for reset the controller (output)
    reset: RESET,
    /// Data/Command Control Pin (High for data, Low for command) (output)
    dc: DC,
}

/// Trait implemented by displays to provide implementation of core functionality.
pub trait DisplayInterface {
    type Error;
    /// 发送指令
    fn send_command(&mut self, command: u8) -> Result<(), Self::Error>;

    /// 发送数据
    fn send_data(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// 重置控制器
    fn reset<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), Self::Error>;

    /// 等待控制器空闲
    fn busy_wait(&mut self);
}

impl<SPI, BUSY, RESET, DC> Interface<SPI, BUSY, RESET, DC>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    RESET: OutputPin,
    DC: OutputPin,
{
    pub fn new(spi: SPI, busy: BUSY, reset: RESET, dc: DC) -> Self {
        Self {
            spi,
            busy,
            reset,
            dc,
        }
    }
}

impl<SPI, BUSY, RESET, DC> DisplayInterface for Interface<SPI, BUSY, RESET, DC>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    RESET: OutputPin,
    DC: OutputPin,
{
    type Error = anyhow::Error;

    fn send_command(&mut self, command: u8) -> Result<(), Self::Error> {
        debug!("Sending command: {:#x}", command);
        self.dc
            .set_low()
            .map_err(|e| anyhow!("dc error: {:?}", e))?;
        self.spi
            .write(&[command])
            .map_err(|e| anyhow!("spi error: {:?}", e))?;
        Ok(())
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        if data.len() < 5 {
            for datum in data.iter() {
                debug!("Sending data: {:#x}", datum);
            }
        }
        self.dc
            .set_high()
            .map_err(|e| anyhow!("dc error: {:?}", e))?;
        self.spi
            .write(data)
            .map_err(|e| anyhow!("spi error: {:?}", e))?;
        Ok(())
    }

    fn reset<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), Self::Error> {
        self.reset
            .set_low()
            .map_err(|e| anyhow!("reset error: {:?}", e))?;
        delay.delay_ms(RESET_DELAY_MS);
        self.reset
            .set_high()
            .map_err(|e| anyhow!("reset error: {:?}", e))?;
        delay.delay_ms(RESET_DELAY_MS);
        Ok(())
    }

    fn busy_wait(&mut self) {
        while self.busy.is_high().unwrap_or_default() {}
    }
}
