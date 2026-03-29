use anyhow::{anyhow, Result};
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, Output};
use esp_hal::spi::master::Spi;
use esp_hal::Blocking;

// Re-export trait and extension traits
pub use super::epd_trait::{EpdDisplay, Ssd1683DisplayExt, Ssd2683DisplayExt};

// Re-export the Display struct (feature-gated)
#[cfg(feature = "ssd1683")]
pub use ssd1683::Display;
#[cfg(feature = "ssd2683")]
pub use ssd2683::Display;

// Re-export DeepSleepMode (feature-gated)
#[cfg(feature = "ssd1683")]
pub use ssd1683::DeepSleepMode;
#[cfg(feature = "ssd2683")]
pub use ssd2683::DeepSleepMode;

#[cfg(feature = "ssd1683")]
pub mod ssd1683;
#[cfg(feature = "ssd2683")]
pub mod ssd2683;

pub struct EpdInterface {
    /// SPI 接口
    spi: Spi<'static, Blocking>,
    /// Active low busy pin (input)
    busy: Input<'static>,
    /// Pin for reset the controller (output)
    reset: Output<'static>,
    /// Data/Command Control Pin (High for data, Low for command) (output)
    dc: Output<'static>,
}

impl EpdInterface {
    pub fn new(
        spi: Spi<'static, Blocking>,
        busy: Input<'static>,
        reset: Output<'static>,
        dc: Output<'static>,
    ) -> Self {
        Self {
            spi,
            busy,
            reset,
            dc,
        }
    }

    pub fn send_command(&mut self, cmd: u8) -> Result<()> {
        self.dc.set_low();
        self.spi
            .write(&[cmd])
            .map_err(|e| anyhow!("spi error: {:?}", e))?;
        Ok(())
    }

    pub fn send_data(&mut self, data: &[u8]) -> Result<()> {
        self.dc.set_high();
        self.spi
            .write(data)
            .map_err(|e| anyhow!("spi error: {:?}", e))?;
        Ok(())
    }

    pub fn receive_data(&mut self) -> Result<u8> {
        let mut buf = [0];
        self.dc.set_high();
        self.spi
            .read(&mut buf)
            .map_err(|e| anyhow!("spi error: {:?}", e))?;
        Ok(buf[0])
    }

    /// 硬件复位
    #[cfg(feature = "ssd1683")]
    pub fn reset(&mut self, delay: &Delay) -> Result<()> {
        self.reset.set_low();
        delay.delay_millis(10);
        self.reset.set_high();
        delay.delay_millis(10);
        Ok(())
    }

    #[cfg(feature = "ssd2683")]
    pub fn reset(&mut self, delay: &Delay) -> Result<()> {
        self.reset.set_high();
        delay.delay_millis(10);
        self.reset.set_low();
        delay.delay_millis(20);
        self.reset.set_high();
        delay.delay_millis(10);
        Ok(())
    }

    /// 等待 BUSY 信号完成
    #[cfg(feature = "ssd1683")]
    pub fn busy_wait(&self) {
        while self.busy.is_high() {}
    }

    #[cfg(feature = "ssd2683")]
    pub fn busy_wait(&self) {
        while self.busy.is_low() {}
    }

    /// 返回当前 busy 信号状态
    pub fn is_busy(&self) -> bool {
        #[cfg(feature = "ssd1683")]
        {
            self.busy.is_high()
        }
        #[cfg(feature = "ssd2683")]
        {
            self.busy.is_low()
        }
    }
}
