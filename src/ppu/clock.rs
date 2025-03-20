pub const CPU_CLOCK_DIVIDER: u32 = 3; // PPU runs at 3x CPU clock rate for NTSC
pub const MASTER_CLOCK_DIVIDER: u32 = 4; // PPU divides master clock by 4

pub const NTSC_SCANLINES: u16 = 262;
pub const PAL_SCANLINES: u16 = 312;
pub const DENDY_SCANLINES: u16 = 312;

pub const NTSC_CYCLES_PER_SCANLINE: u16 = 341;
pub const PAL_CYCLES_PER_SCANLINE: u16 = 341;
pub const DENDY_CYCLES_PER_SCANLINE: u16 = 341;
