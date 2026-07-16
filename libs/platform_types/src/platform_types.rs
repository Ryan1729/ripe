use gfx_sizes::ARGB;
pub use pak_types::*;

pub type PaletteIndex = u8;

pub mod command {
    use xs::Xs;
    use gfx_sizes::ARGB;
    use pak_types::{sprite, unscaled::{self, XD, YD}};

    pub type Inner = u16;
    pub type SignedInner = i16;

    pub const WIDTH: Inner = gfx_sizes::COMMAND_WIDTH;
    pub const HEIGHT: Inner = gfx_sizes::COMMAND_HEIGHT;

    pub const WIDTH_SIGNED: SignedInner = gfx_sizes::COMMAND_WIDTH_SIGNED;
    pub const HEIGHT_SIGNED: SignedInner = gfx_sizes::COMMAND_HEIGHT_SIGNED;

    pub const X_MAX_SIGNED: SignedInner = WIDTH_SIGNED - 1;
    pub const Y_MAX_SIGNED: SignedInner = HEIGHT_SIGNED - 1;

    pub const LENGTH: usize = WIDTH as usize * HEIGHT as usize;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct X(Inner);

    impl X {
        pub const MAX: X = X(WIDTH - 1);

        pub const fn u16(self) -> u16 {
            self.0
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Y(Inner);

    impl Y {
        pub const MAX: Y = Y(HEIGHT - 1);

        pub const fn u16(self) -> u16 {
            self.0
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct W(Inner);

    impl W {
        pub const MAX: W = W(WIDTH - 1);

        pub const fn u16(self) -> u16 {
            self.0
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct H(Inner);

    impl H {
        pub const MAX: H = H(WIDTH - 1);

        pub const fn u16(self) -> u16 {
            self.0
        }
    }

    impl From<X> for Inner {
        fn from(to_convert: X) -> Inner {
            to_convert.0
        }
    }

    impl From<Y> for Inner {
        fn from(to_convert: Y) -> Inner {
            to_convert.0
        }
    }

    impl From<W> for Inner {
        fn from(to_convert: W) -> Inner {
            to_convert.0
        }
    }

    impl From<H> for Inner {
        fn from(to_convert: H) -> Inner {
            to_convert.0
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Rect {
        pub x_min: X,
        pub y_min: Y,
        pub x_max: X,
        pub y_max: Y,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Command {
        sprite_xy: sprite::XY<sprite::Renderable>,
        rect: Rect,
        colour_override: ARGB,
    }

    impl Command {
        /// If this returns None, then there's no useful command to render, because it wouldn't affect any pixels.
        pub fn new(
            mut sprite_xy: sprite::XY<sprite::Renderable>,
            rect: unscaled::Rect,
            colour_override: ARGB,
        ) -> Option<Self> {
            let (x, x_min_clip_amount) = if rect.x.0 == unscaled::Inner::MIN {
                // sprites are not allowed to be large enough to still be on screen if placed here.
                return None
            } else if rect.x.0 < 0 {
                (0, -rect.x.0)
            } else {
                // We can cast because we checked it's not negative
                (rect.x.0 as u16, 0)
            };
            let (y, y_min_clip_amount) = if rect.y.0 == unscaled::Inner::MIN {
                // sprites are not allowed to be large enough to still be on screen if placed here.
                return None
            } else if rect.y.0 < 0 {
                (0, -rect.y.0)
            } else {
                // We can cast because we checked it's not negative
                (rect.y.0 as u16, 0)
            };

            let x_max_raw = x as unscaled::NextUp + (rect.w.get() as unscaled::NextUp - x_min_clip_amount) - 1;
            let y_max_raw = y as unscaled::NextUp + (rect.h.get() as unscaled::NextUp - y_min_clip_amount) - 1;

            let x_max = if x_max_raw > X_MAX_SIGNED {
                X::MAX
            } else if x_max_raw < 0 {
                return None
            } else {
                X(x_max_raw as Inner)
            };

            let y_max = if y_max_raw > Y_MAX_SIGNED {
                Y::MAX
            } else if y_max_raw < 0 {
                return None
            } else {
                Y(y_max_raw as Inner)
            };

            let clipped = Rect {
                x_min: X(x),
                y_min: Y(y),
                x_max,
                y_max,
            };

            if clipped.x_max.0 <= clipped.x_min.0 {
                return None
            }
            if clipped.y_max.0 <= clipped.y_min.0 {
                return None
            }

            sprite_xy.x += unscaled::W::new(x_min_clip_amount);
            sprite_xy.y += unscaled::H::new(y_min_clip_amount);

            Some(Command {
                rect: clipped,
                sprite_xy,
                colour_override,
            })
        }

        pub fn rect(&self) -> Rect { self.rect }
        pub fn sprite_xy(&self) -> sprite::XY<sprite::Renderable> { self.sprite_xy }
        pub fn colour_override(&self) -> ARGB { self.colour_override }
    }
}
pub use command::Command;

#[cfg(test)]
mod command_new_works {
    use super::*;

    #[test]
    fn on_this_found_example() {
        let actual = Command::new(
            sprite::XY {
                x: sprite::x(
                    256,
                ),
                y: sprite::y(
                    432,
                ),
            },
            unscaled::Rect {
                x: unscaled::X(
                    160,
                ),
                y: unscaled::Y(
                    -210,
                ),
                w: unscaled::W::new(
                    56,
                ),
                h: unscaled::H::new(
                    48,
                ),
            },
            0
        );

        assert_eq!(actual, None);
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Input {
    pub gamepad: Button,
    pub previous_gamepad: Button,
}

impl Input {
    #[allow(unused)]
    pub fn pressed_this_frame(&self, buttons: Button) -> bool {
        !self.previous_gamepad.contains(buttons) && self.gamepad.contains(buttons)
    }

    #[allow(unused)]
    pub fn released_this_frame(&self, buttons: Button) -> bool {
        self.previous_gamepad.contains(buttons) && !self.gamepad.contains(buttons)
    }

    pub fn dir_pressed_this_frame(&self) -> Option<Dir> {
        if self.pressed_this_frame(Button::UP) {
            Some(Dir::Up)
        } else if self.pressed_this_frame(Button::DOWN) {
            Some(Dir::Down)
        } else if self.pressed_this_frame(Button::LEFT) {
            Some(Dir::Left)
        } else if self.pressed_this_frame(Button::RIGHT) {
            Some(Dir::Right)
        } else {
            None
        }
    }

    pub fn contains_dir(&self) -> Option<Dir> {
        if self.gamepad.contains(Button::UP) {
            Some(Dir::Up)
        } else if self.gamepad.contains(Button::DOWN) {
            Some(Dir::Down)
        } else if self.gamepad.contains(Button::LEFT) {
            Some(Dir::Left)
        } else if self.gamepad.contains(Button::RIGHT) {
            Some(Dir::Right)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SFX {
    CardPlace,
    CardSlide,
    ButtonPress,
}

pub struct Speaker {
    requests: Vec<SFX>,
}

impl Default for Speaker {
    fn default() -> Self {
        Speaker {
            requests: Vec::with_capacity(8),
        }
    }
}

impl Speaker {
    pub fn clear(&mut self) {
        self.requests.clear();
    }

    pub fn request_sfx(&mut self, sfx: SFX) {
        self.requests.push(sfx);
    }

    pub fn slice(&self) -> &[SFX] {
        &self.requests
    }
}

// These values are deliberately picked to be the same as the ones in NES' input registers.
pub mod button {
    #[cfg(not(feature = "refresh"))]
    type Inner = u8;

    #[cfg(feature = "refresh")]
    type Inner = u16;

    #[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
    pub struct Button(Inner);

    impl Button {
        pub const A     : Self = Self(1 << 0);
        pub const B     : Self = Self(1 << 1);
        pub const SELECT: Self = Self(1 << 2);
        pub const START : Self = Self(1 << 3);
        pub const UP    : Self = Self(1 << 4);
        pub const DOWN  : Self = Self(1 << 5);
        pub const LEFT  : Self = Self(1 << 6);
        pub const RIGHT : Self = Self(1 << 7);

        #[cfg(feature = "refresh")]
        pub const RESET : Self = Self(1 << 8);

        #[cfg(not(feature = "refresh"))]
        pub const ALL : [Self; 8] = [
            Self::A,
            Self::B,
            Self::SELECT,
            Self::START,
            Self::UP,
            Self::DOWN,
            Self::LEFT,
            Self::RIGHT,
        ];

        #[cfg(feature = "refresh")]
        pub const ALL : [Self; 9] = [
            Self::A,
            Self::B,
            Self::SELECT,
            Self::START,
            Self::UP,
            Self::DOWN,
            Self::LEFT,
            Self::RIGHT,
            Self::RESET,
        ];

        pub const fn contains(&self, other: Self) -> bool {
            self.0 & other.0 == other.0
        }

        pub fn insert(&mut self, other: Self) {
            self.0 |= other.0;
        }

        pub fn remove(&mut self, other: Self) {
            self.0 &= !other.0;
        }
    }
}
pub use button::Button;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dir {
    #[default]
    Left,
    Right,
    Up,
    Down,
}

pub type DirFlag = u8;

impl Dir {
    pub const ALL: [Dir; 4] = [
        Dir::Left,
        Dir::Right,
        Dir::Up,
        Dir::Down,
    ];

    pub const OPPOSITE: [Dir; 4] = [
        Dir::Right,
        Dir::Left,
        Dir::Down,
        Dir::Up,
    ];

    pub const FLAG: [DirFlag; 4] = [
        1 << 0,
        1 << 1,
        1 << 2,
        1 << 3,
    ];
    
    pub const fn u8(self) -> u8 {
        self as u8
    }

    pub const fn index(self) -> usize {
        self.u8() as usize
    }

    pub const fn flag(self) -> DirFlag {
        Self::FLAG[self.index()]
    }

    pub const fn opposite(self) -> Dir {
        Self::OPPOSITE[self.index()]
    }

    pub const fn moves_in_x(self) -> bool {
        self.u8() == Dir::Left.u8()
        || self.u8() == Dir::Right.u8()
    }

    pub const fn moves_in_y(self) -> bool {
        self.u8() == Dir::Up.u8()
        || self.u8() == Dir::Down.u8()
    }
}

pub type Logger = Option<fn(&str) -> ()>;

pub trait PakReader
where 
    Self: std::io::Read + std::io::Seek
{}

impl<T: ?Sized> PakReader for T
where
    Self: std::io::Read + std::io::Seek
{}

pub type PakLoader = Option<fn() -> Option<Box<dyn PakReader>>>;

#[derive(Clone, Copy)]
pub struct StateParams {
    pub seed: [u8; 16], 
    pub logger: Logger,
    pub error_logger: Logger, 
    pub pak_loader: PakLoader,
}

// reportedly colourblind friendly colours
// https://twitter.com/ea_accessible/status/968595073184092160
pub mod colours {
    use super::ARGB;

    pub const BLUE: ARGB = 0xFF3352E1;
    pub const GREEN: ARGB = 0xFF30B06E;
    pub const RED: ARGB = 0xFFDE4949;
    pub const YELLOW: ARGB = 0xFFFFB937;
    pub const PURPLE: ARGB = 0xFF533354;
    #[allow(unused)]
    pub const GREY: ARGB = 0xFF5A7D8B;
    #[allow(unused)]
    pub const GRAY: ARGB = GREY;
    pub const WHITE: ARGB = 0xFFEEEEEE;
    pub const BLACK: ARGB = 0xFF222222;
}

pub use colours::*;

pub const PALETTE: [ARGB; 8] = [
    BLUE,
    GREEN,
    RED,
    YELLOW,
    PURPLE,
    GREY,
    WHITE,
    BLACK,
];

pub mod arrow_timer {
    use pak_types::unscaled::{self, W, H};

    /// 64k arrow frames ought to be enough for anybody!
    pub type ArrowTimer = u16;
    
    const MAX: ArrowTimer = 128;
    
    pub fn tick(timer: &mut ArrowTimer) {
        if *timer == 0 {
            *timer = MAX;
        } else {
            *timer = timer.saturating_sub(1);
        }
    }

    /// The max W value that will be returned from `offset`.
    pub const MAX_W: unscaled::W = unscaled::W::ZERO;
    /// The max H value that will be returned from `offset`.
    pub const MAX_H: unscaled::H = unscaled::H::TWO;

    pub fn offset(timer: ArrowTimer) -> unscaled::WH {
        if timer < 32 {
            unscaled::WH{ w: W::ZERO, h: H::TWO }
        } else if timer < 64 {
            unscaled::WH{ w: W::ZERO, h: H::ONE }
        } else if timer < 96 {
            unscaled::WH{ w: W::ZERO, h: H::ZERO }
        } else {
            unscaled::WH{ w: W::ZERO, h: H::ONE }
        }
    }
}