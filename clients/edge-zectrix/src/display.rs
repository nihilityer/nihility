use anyhow::Result;
use core::cell::Cell;
use critical_section::Mutex;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::peripherals::{GPIO10, GPIO11, GPIO12, GPIO4, GPIO8, GPIO9, SPI2};
use esp_hal::spi::master::Spi;
use esp_hal::spi::{BitOrder, Mode};
use esp_hal::time::Rate;
use esp_hal::Blocking;
use ssd1683::{Display, Interface};

const WIDTH: u16 = 400;
const HEIGHT: u16 = 300;

static DISPLAY: Mutex<
    Cell<
        Option<
            Display<
                ExclusiveDevice<Spi<'static, Blocking>, Output, Delay>,
                Input,
                Output,
                Output,
                Delay,
            >,
        >,
    >,
> = Mutex::new(Cell::new(None));

pub fn init_display(
    busy: GPIO4<'static>,
    reset: GPIO8<'static>,
    dc: GPIO9<'static>,
    cs: GPIO10<'static>,
    spi2: SPI2<'static>,
    sck: GPIO12<'static>,
    mosi: GPIO11<'static>,
) -> Result<()> {
    let busy = Input::new(busy, InputConfig::default());
    let reset = Output::new(reset, Level::Low, OutputConfig::default());
    let dc = Output::new(dc, Level::Low, OutputConfig::default());
    let cs = Output::new(cs, Level::Low, OutputConfig::default());

    let spi = Spi::new(
        spi2,
        esp_hal::spi::master::Config::default()
            .with_mode(Mode::_0)
            .with_frequency(Rate::from_mhz(10))
            .with_write_bit_order(BitOrder::MsbFirst),
    )?
    .with_sck(sck)
    .with_mosi(mosi);
    let spi_device = ExclusiveDevice::new(spi, cs, Delay::new())?;

    let interface = Interface::new(spi_device, busy, reset, dc);
    let display = Display::new(interface, Delay::new(), WIDTH as usize, HEIGHT);

    critical_section::with(|cs| {
        DISPLAY.borrow(cs).replace(Some(display));
    });
    Ok(())
}
