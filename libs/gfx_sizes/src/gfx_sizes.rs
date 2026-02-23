/// A colour represented as a byte for each of Alpha, Red, Green, and Blue.
/// Alpha is the most significant byte, blue is the least:
/// `0xaarrggbb`
pub type ARGB = u32;

pub const GFX_WIDTH: usize = 768;
pub const GFX_HEIGHT: usize = 256;
pub const GFX_LENGTH: usize = GFX_WIDTH * GFX_HEIGHT;

/// Small enough to fit on pretty much any reasonable device, at an aspect ratio
/// of 3:2 (1.5), which is a compromise between 4:3 (1.33...) and 16:9 (1.788...).
pub const COMMAND_WIDTH: u16 = 480;
pub const COMMAND_HEIGHT: u16 = 320;