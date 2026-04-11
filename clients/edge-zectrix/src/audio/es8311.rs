pub use config::*;
use embedded_hal::i2c::{Error as I2cError, I2c};
pub use error::Error;
use log::info;
use register::Register;

mod config;
mod error;
mod register;

pub struct Es8311<I2C> {
    i2c: I2C,
    address: Address,
}

#[allow(dead_code)]
impl<I2C: I2c<Error = E>, E: I2cError> Es8311<I2C> {
    pub fn new(i2c: I2C, address: Address) -> Self {
        Self { i2c, address }
    }

    pub fn init(&mut self, config: &Config) -> Result<(), Error<E>> {
        let regv = self.read_reg(Register::SystemReg0D)?;
        if regv != 0xFA {
            info!("SystemReg0D: {}", regv);
            self.write_reg(Register::SystemReg0D, 0xFA)?;
        }
        self.write_reg(Register::GpioReg44, 0x08)?;
        self.write_reg(Register::GpioReg44, 0x08)?;

        self.write_reg(Register::ClkManagerReg01, 0x30)?;
        self.write_reg(Register::ClkManagerReg02, 0x00)?;
        self.write_reg(Register::ClkManagerReg03, 0x10)?;
        self.write_reg(Register::AdcReg16, 0x24)?;
        self.write_reg(Register::ClkManagerReg04, 0x10)?;
        self.write_reg(Register::ClkManagerReg05, 0x00)?;
        self.write_reg(Register::SystemReg0B, 0x00)?;
        self.write_reg(Register::SystemReg0C, 0x00)?;
        self.write_reg(Register::SystemReg10, 0x1F)?;
        self.write_reg(Register::SystemReg11, 0x7F)?;
        self.write_reg(Register::ResetReg00, 0x80)?;

        let regv = self.read_reg(Register::ResetReg00)? & 0xBF;
        self.write_reg(Register::ResetReg00, regv)?;

        self.clock_config(config)?;

        self.write_reg(Register::SystemReg13, 0x10)?;
        self.write_reg(Register::AdcReg1B, 0x0A)?;
        self.write_reg(Register::AdcReg1C, 0x6A)?;
        self.write_reg(Register::GpioReg44, 0x58)?;

        self.set_bits_per_sample(config)?;
        self.config_fmt(config)?;
        self.config_sample(config)?;

        self.start()?;
        Ok(())
    }

    fn clock_config(&mut self, config: &Config) -> Result<(), Error<E>> {
        let mut reg01 = 0x3F;
        if config.mclk.is_some() {
            reg01 &= 0x7F;
        } else {
            reg01 |= 0x80;
        }

        if config.mclk_inverted {
            reg01 |= 1 << 6;
        } else {
            reg01 &= !(1 << 6);
        }
        self.write_reg(Register::ClkManagerReg01, reg01)?;

        let mut reg06 = self.read_reg(Register::ClkManagerReg06)?;
        if config.sclk_inverted {
            reg06 |= 1 << 5;
        } else {
            reg06 &= !(1 << 5);
        }
        self.write_reg(Register::ClkManagerReg06, reg06)
    }

    fn set_bits_per_sample(&mut self, config: &Config) -> Result<(), Error<E>> {
        let mut dac_iface = self.read_reg(Register::SdpInReg09)?;
        let mut adc_iface = self.read_reg(Register::SdpOutReg0A)?;
        match config.bits_per_sample {
            Resolution::Resolution24 => {
                dac_iface &= !0x1C;
                adc_iface &= !0x1C;
            }
            Resolution::Resolution32 => {
                dac_iface |= 0x10;
                adc_iface |= 0x10;
            }
            _ => {
                dac_iface |= 0x0C;
                adc_iface |= 0x0C;
            }
        }
        self.write_reg(Register::SdpInReg09, dac_iface)?;
        self.write_reg(Register::SdpOutReg0A, adc_iface)?;
        Ok(())
    }

    fn config_fmt(&mut self, _config: &Config) -> Result<(), Error<E>> {
        let mut dac_iface = self.read_reg(Register::SdpInReg09)?;
        let mut adc_iface = self.read_reg(Register::SdpOutReg0A)?;
        dac_iface &= 0xFC;
        adc_iface &= 0xFC;
        self.write_reg(Register::SdpInReg09, dac_iface)?;
        self.write_reg(Register::SdpOutReg0A, adc_iface)?;
        Ok(())
    }

    fn config_sample(&mut self, config: &Config) -> Result<(), Error<E>> {
        let mclk_fre = match config.mclk {
            None => MclkFreq::try_from_freq(config.sample_frequency.as_freq() * 256)
                .ok_or(Error::InvalidConfiguration)?,
            Some(mclk) => mclk,
        };
        let coefficients = Coefficients::get(mclk_fre, config.sample_frequency)
            .ok_or(Error::InvalidConfiguration)?;
        let mut regv = self.read_reg(Register::ClkManagerReg02)?;
        regv &= 0x7;
        regv |= (coefficients.pre_div - 1) << 5;
        let datmp = match &coefficients.pre_multi {
            &1 => 0,
            &2 => 1,
            &4 => 2,
            &8 => 3,
            _ => 0,
        };
        regv |= datmp << 3;
        self.write_reg(Register::ClkManagerReg02, regv)?;

        let mut regv = 0x00;
        regv |= (coefficients.adc_div - 1) << 4;
        regv |= (coefficients.dac_div - 1) << 0;
        self.write_reg(Register::ClkManagerReg05, regv)?;

        let mut regv = self.read_reg(Register::ClkManagerReg03)?;
        regv &= 0x80;
        regv |= coefficients.fs_mode << 6;
        regv |= coefficients.adc_osr << 0;
        self.write_reg(Register::ClkManagerReg03, regv)?;

        let mut regv = self.read_reg(Register::ClkManagerReg04)?;
        regv &= 0x80;
        regv |= coefficients.dac_osr << 0;
        self.write_reg(Register::ClkManagerReg04, regv)?;

        let mut regv = self.read_reg(Register::ClkManagerReg07)?;
        regv &= 0xC0;
        regv |= coefficients.lrck_h << 0;
        self.write_reg(Register::ClkManagerReg07, regv)?;

        let mut regv = 0x00;
        regv |= coefficients.lrck_l << 0;
        self.write_reg(Register::ClkManagerReg08, regv)?;

        let mut regv = self.read_reg(Register::ClkManagerReg06)?;
        regv &= 0xE0;
        if coefficients.bclk_div < 19 {
            regv |= (coefficients.bclk_div - 1) << 0;
        } else {
            regv |= coefficients.bclk_div << 0;
        }
        self.write_reg(Register::ClkManagerReg06, regv)?;
        Ok(())
    }

    fn start(&mut self) -> Result<(), Error<E>> {
        let mut regv = 0x80;
        regv &= 0xBF;
        self.write_reg(Register::ResetReg00, regv)?;
        regv = 0x3F;
        regv &= 0x7F;
        regv &= !(0x40);
        self.write_reg(Register::ClkManagerReg01, regv)?;

        let mut dac_iface = self.read_reg(Register::SdpInReg09)?;
        let mut adc_iface = self.read_reg(Register::SdpOutReg0A)?;
        dac_iface &= 0xBF;
        adc_iface &= 0xBF;
        dac_iface &= !(1 << 6);
        adc_iface &= !(1 << 6);
        self.write_reg(Register::SdpInReg09, dac_iface)?;
        self.write_reg(Register::SdpOutReg0A, adc_iface)?;

        self.write_reg(Register::AdcReg17, 0xBF)?;
        self.write_reg(Register::SystemReg0E, 0x02)?;
        self.write_reg(Register::SystemReg12, 0x00)?;
        self.write_reg(Register::SystemReg14, 0x1A)?;

        let mut regv = self.read_reg(Register::SystemReg14)?;
        regv &= !0x40;
        self.write_reg(Register::SystemReg14, regv)?;
        self.write_reg(Register::SystemReg0D, 0x01)?;
        self.write_reg(Register::AdcReg15, 0x40)?;
        self.write_reg(Register::DacReg37, 0x08)?;
        self.write_reg(Register::GpReg45, 0x00)?;
        Ok(())
    }

    pub fn set_voice_volume(&mut self, volume: u8) -> Result<(), Error<E>> {
        self.write_reg(Register::DacReg32, volume)
    }

    pub fn voice_volume(&mut self) -> Result<u8, Error<E>> {
        self.read_reg(Register::DacReg32)
    }

    pub fn voice_mute(&mut self, mute: bool) -> Result<(), Error<E>> {
        let mut reg31 = self.read_reg(Register::DacReg31)?;
        const MUTE: u8 = (1 << 6) | (1 << 5);
        if mute {
            reg31 |= MUTE;
        } else {
            reg31 &= !MUTE;
        }
        self.write_reg(Register::DacReg31, reg31)
    }

    pub fn set_mic_gain(&mut self, gain: Gain) -> Result<(), Error<E>> {
        self.write_reg(Register::AdcReg16, gain as u8)
    }

    pub fn set_mic_fade(&mut self, fade: Fade) -> Result<(), Error<E>> {
        self.set_fade(Register::AdcReg16, fade)
    }

    pub fn set_voice_fade(&mut self, fade: Fade) -> Result<(), Error<E>> {
        self.set_fade(Register::DacReg37, fade)
    }

    fn set_fade(&mut self, register: Register, fade: Fade) -> Result<(), Error<E>> {
        let mut reg = self.read_reg(register)?;
        reg &= 0x0F;
        reg |= (fade as u8) << 4;
        self.write_reg(register, reg)
    }

    pub fn dump_regs(&mut self) -> Result<(), Error<E>> {
        use strum::IntoEnumIterator;
        for register in Register::iter() {
            let reg_val = register as u8;
            let val = self.read_reg(register)?;
            info!("register {register:?} at address {reg_val:#02X} with value {val:#02X}")
        }
        Ok(())
    }

    fn read_reg(&mut self, reg: Register) -> Result<u8, Error<E>> {
        use core::array::from_mut;
        let mut value = 0;
        self.i2c
            .write_read(self.address as u8, &[reg as u8], from_mut(&mut value))
            .map_err(Error::BusError)?;
        Ok(value)
    }

    fn write_reg(&mut self, reg: Register, value: u8) -> Result<(), Error<E>> {
        self.i2c
            .write(self.address as u8, &[reg as u8, value])
            .map_err(Error::BusError)
    }
}
