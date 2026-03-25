use anyhow::{anyhow, Result};
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, Output};
use esp_hal::spi::master::Spi;
use esp_hal::Blocking;

pub mod ssd1683;
mod ssd2683;

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

    pub fn reset(&mut self, delay: &Delay, is_long: bool) -> Result<()> {
        if is_long {
            self.reset.set_high();
            delay.delay_millis(10);
            self.reset.set_low();
            delay.delay_millis(20);
            self.reset.set_high();
            delay.delay_millis(10);
        } else {
            self.reset.set_low();
            delay.delay_millis(10);
            self.reset.set_high();
            delay.delay_millis(10);
        }
        Ok(())
    }

    fn busy_wait_high(&self) {
        while self.busy.is_low() {}
    }

    fn busy_wait_low(&self) {
        while self.busy.is_high() {}
    }
}
