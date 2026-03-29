use anyhow::Result;

/// Unified Display trait for EPD (Electronic Paper Display) chips.
///
/// This trait provides a simplified interface for EPD operations.
/// Each implementation handles chip-specific details internally.
pub trait EpdDisplay {
    /// Initialize the display.
    /// Internally decides whether to use normal or fast mode based on update count.
    fn init(&mut self) -> Result<()>;

    /// Full screen update: write data, refresh, and enter deep sleep.
    fn full_update(&mut self, data: &[u8]) -> Result<()>;

    /// Partial update: write partial region data and refresh.
    /// For SSD1683: writes data then triggers refresh
    /// For SSD2683: write_partial already includes refresh
    fn partial_update(&mut self, x: u16, y: u16, width: u16, height: u16, data: &[u8]) -> Result<()>;
}
