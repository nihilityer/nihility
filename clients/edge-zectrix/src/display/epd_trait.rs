use anyhow::Result;

/// Unified Display trait for EPD (Electronic Paper Display) chips.
///
/// # Design Philosophy
/// This trait defines common operations that EPD chips can perform.
/// Each implementation maps these operations to chip-specific command sequences.
///
/// # Important Behavioral Differences
/// Implementors MUST document these differences clearly:
///
/// 1. **`update()` semantics differ**:
///    - SSD1683: Sends DisplayUpdateControl2 + MasterActivation (no power cycling)
///    - SSD2683: Complete power cycle (PowerOn → DisplayRefresh → PowerOff)
///
/// 2. **Partial update flow**:
///    - SSD1683: `write_partial()` + `update_partial()` (TWO separate calls required)
///    - SSD2683: `write_partial()` alone completes the entire update
///
/// 3. **Red channel handling**:
///    - SSD1683: `write_all(bw, red)` sends `red` to red RAM
///    - SSD2683: `write_all(bw, red)` ignores `red`, always sends white (0xFF)
///
/// 4. **Data bit order**:
///    - SSD1683: LSB on right (each byte's LSB = rightmost pixel)
///    - SSD2683: MSB on left (each byte's MSB = leftmost pixel)
///    - Implementations handle bit order conversion internally
pub trait EpdDisplay {
    /// Returns the display width in pixels.
    fn width(&self) -> usize;

    /// Returns the display height in pixels.
    fn height(&self) -> u16;

    /// Initialize the display for normal (full) refresh mode.
    /// This performs hardware reset, soft reset, and chip-specific initialization.
    fn init(&mut self) -> Result<()>;

    /// Initialize the display for fast refresh mode.
    ///
    /// # Parameters
    /// - `use_otp`: true = OTP-based initialization, false = temperature LUT initialization
    ///
    /// # Note
    /// For SSD2683, `init_fast(true)` calls both `otp_init()` and `normal_init()`.
    fn init_fast(&mut self, use_otp: bool) -> Result<()>;

    /// Write full frame data to the display.
    ///
    /// # Parameters
    /// - `black_white`: Black/white pixel data (length = width * height / 8)
    /// - `red`: Red channel data (length = width * height / 8)
    ///
    /// # Behavior differences
    /// - SSD1683: Writes both `black_white` and `red` to their respective RAM
    /// - SSD2683: Writes `black_white` to B/W RAM, ignores `red` (always writes 0xFF to red RAM)
    fn write_all(&mut self, black_white: &[u8], red: &[u8]) -> Result<()>;

    /// Trigger a full display refresh.
    ///
    /// # Warning - Semantics differ between chips!
    /// - SSD1683: Sends DisplayUpdateControl2 + MasterActivation, waits for busy
    ///            (no power cycling, call AFTER write_all())
    /// - SSD2683: Complete power cycle: PowerOn → DisplayRefresh → PowerOff
    ///            (write_all() already includes the refresh sequence)
    fn update(&mut self) -> Result<()>;

    /// Write partial region data.
    ///
    /// # Parameters
    /// - `x`, `y`: Region origin in pixels
    /// - `width`, `height`: Region dimensions in pixels (width must be multiple of 8)
    /// - `data`: Pixel data for the region (length = width * height / 8)
    ///
    /// # Note
    /// - SSD1683: Sets window and transfers data. You MUST call `update_partial()` after.
    /// - SSD2683: Completes entire partial update including power cycle.
    fn write_partial(&mut self, x: u16, y: u16, width: u16, height: u16, data: &[u8])
    -> Result<()>;

    /// Trigger a partial display refresh.
    ///
    /// # Important
    /// - SSD1683: Call AFTER `write_partial()` to trigger the refresh
    /// - SSD2683: Not applicable - `write_partial()` already includes refresh
    fn update_partial(&mut self) -> Result<()>;

    /// Enter deep sleep mode.
    /// Call this before powering off the display.
    fn deep_sleep(&mut self) -> Result<()>;
}
