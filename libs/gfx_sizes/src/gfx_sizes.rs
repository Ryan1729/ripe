/// A colour represented as a byte for each of Alpha, Red, Green, and Blue.
/// Alpha is the most significant byte, blue is the least:
/// `0xaarrggbb`
pub type ARGB = u32;

pub const GFX_WIDTH: usize = 768;
pub const GFX_HEIGHT: usize = 512;
pub const GFX_LENGTH: usize = GFX_WIDTH * GFX_HEIGHT;

/// Small enough to fit on pretty much any reasonable device, at an aspect ratio
/// of 3:2 (1.5), which is a compromise between 4:3 (1.33...) and 16:9 (1.788...).
pub const COMMAND_WIDTH: u16 = 480;
pub const COMMAND_HEIGHT: u16 = 320;

macro_rules! compile_time_assert {
    ($assertion: expr) => (
        #[allow(unknown_lints, clippy::eq_op)]
        // Based on the const_assert macro from static_assertions;
        const _: [(); 0 - !{$assertion} as usize] = [];
    )
}

compile_time_assert!{
    COMMAND_WIDTH as i32 <= i16::MAX as i32
}
compile_time_assert!{
    COMMAND_HEIGHT as i32 <= i16::MAX as i32
}

/// We assert that COMMAND_WIDTH is not too large to fit into an i16.
pub const COMMAND_WIDTH_SIGNED: i16 = COMMAND_WIDTH as _;
pub const COMMAND_HEIGHT_SIGNED: i16 = COMMAND_HEIGHT as _;