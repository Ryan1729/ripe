/// A colour represented as a byte for each of Alpha, Red, Green, and Blue.
/// Alpha is the most significant byte, blue is the least:
/// `0xaarrggbb`
pub type ARGB = u32;

pub const GFX_WIDTH: usize = 768;
pub const GFX_HEIGHT: usize = 256;
pub const GFX_LENGTH: usize = GFX_WIDTH * GFX_HEIGHT;