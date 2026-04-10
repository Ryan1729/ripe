///! B.O.L.D.
/// Boldly Or Leisurely Dashing
/// or
/// Boulders Often Lope Downwards

use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use vec1::{Grid1, Grid1Spec, vec1, Vec1};
use xs::Xs;

use std::num::TryFromIntError;

type Index = usize;

pub mod xy {
    pub type Inner = u16;
    /// A signed type large enough to hold the difference between two Inner
    /// values.
    pub type Diff = i32;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct X(pub Inner);

    pub fn x(inner: Inner) -> X { X(inner) }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Y(pub Inner);

    pub fn y(inner: Inner) -> Y { Y(inner) }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct W(pub Inner);

    pub fn w(inner: Inner) -> W { W(inner) }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct H(pub Inner);

    pub fn h(inner: Inner) -> H { H(inner) }

    macro_rules! def {
        ($($name: path),+) => {
            $(
                impl $name {
                    pub const ONE: Self = Self(1);

                    pub fn dec(&self) -> Self {
                        Self(self.0.saturating_sub(1))
                    }

                    pub fn inc(&self) -> Self {
                        Self(self.0.saturating_add(1))
                    }

                    pub fn double(&self) -> Self {
                        Self(self.0.saturating_mul(2))
                    }

                    pub fn u32(self) -> u32 {
                        u32::from(self.0)
                    }

                    pub fn usize(self) -> usize {
                        usize::from(self.0)
                    }

                    pub fn diff(self) -> Diff {
                        Diff::from(self.0)
                    }
                }

                impl From<$name> for Diff {
                    fn from(value: $name) -> Diff {
                        Diff::from(value.0)
                    }
                }
            )+
        }
    }

    def!{ X, Y, W, H }

    macro_rules! unsigned_paired_impls {
        ($($a_name: ident, $b_name: ident)+) => {$(
            impl core::ops::AddAssign<$b_name> for $a_name {
                fn add_assign(&mut self, other: $b_name) {
                    self.0 += other.0;
                }
            }

            impl core::ops::Add<$b_name> for $a_name {
                type Output = Self;

                fn add(mut self, other: $b_name) -> Self::Output {
                    self += other;
                    self
                }
            }

            impl core::ops::SubAssign<$b_name> for $a_name {
                fn sub_assign(&mut self, other: $b_name) {
                    self.0 -= other.0;
                }
            }

            impl core::ops::Sub<$b_name> for $a_name {
                type Output = Self;

                fn sub(mut self, other: $b_name) -> Self::Output {
                    self -= other;
                    self
                }
            }

            impl $a_name {
                pub fn checked_add(&self, b: $b_name) -> Option<Self> {
                    Some(Self(self.0.checked_add(b.0)?))
                }

                pub fn checked_sub(&self, b: $b_name) -> Option<Self> {
                    Some(Self(self.0.checked_sub(b.0)?))
                }
            }
        )+}
    }

    unsigned_paired_impls!{
        X, W
        Y, H
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct XY {
        pub x: X,
        pub y: Y,
    }

    //impl From<XY> for offset::Point {
        //fn from(XY { x, y }: XY) -> Self {
            //(offset::Inner::from(x.0), offset::Inner::from(y.0))
        //}
    //}
//
    //impl XY {
        //pub fn checked_push(self, dir: impl Into<crate::Dir8>) -> Option<XY> {
            //use crate::Dir8::*;
            //Some(match dir.into() {
                //UpLeft => XY { x: self.x.checked_sub(W::ONE)?, y: self.y.checked_sub(H::ONE)? },
                //Up => XY { x: self.x, y: self.y.checked_sub(H::ONE)? },
                //UpRight => XY { x: self.x.checked_add(W::ONE)?, y: self.y.checked_sub(H::ONE)? },
                //Right => XY { x: self.x.checked_add(W::ONE)?, y: self.y },
                //DownRight => XY { x: self.x.checked_add(W::ONE)?, y: self.y.checked_add(H::ONE)? },
                //Down => XY { x: self.x, y: self.y.checked_add(H::ONE)? },
                //DownLeft => XY { x: self.x.checked_sub(W::ONE)?, y: self.y.checked_add(H::ONE)? },
                //Left => XY { x: self.x.checked_sub(W::ONE)?, y: self.y },
            //})
        //}
    //}
//
    //use crate::{EdgeHitKind, TilesWidth, xy_to_i, xy_in_dir};
//
    //impl pathfinding::XYTrait<TilesWidth, platform_types::Dir> for XY {
        //fn to_i(self, &width: &TilesWidth) -> usize {
            //xy_to_i(width, self).unwrap_or(usize::MAX)
        //}
        //fn apply_dir(self, dir: platform_types::Dir) -> Option<Self> {
            //if let (xy, EdgeHitKind::Neither) = xy_in_dir(self, dir) {
                //Some(xy)
            //} else {
                //None
            //}
        //}
        //fn chebyshev_distance_to(self, other: Self) -> usize {
            //core::cmp::max((other.x.diff() - self.x.diff()).abs(), (other.y.diff() - self.y.diff()).abs())
                //.try_into().unwrap_or(usize::MAX)
        //}
    //}
//
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct WH {
        pub w: W,
        pub h: H,
    }

    impl core::ops::AddAssign<WH> for XY {
        fn add_assign(&mut self, other: WH) {
            self.x += other.w;
            self.y += other.h;
        }
    }

    impl core::ops::Add<WH> for XY {
        type Output = Self;

        fn add(mut self, other: WH) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::AddAssign<W> for XY {
        fn add_assign(&mut self, other: W) {
            self.x += other;
        }
    }

    impl core::ops::Add<W> for XY {
        type Output = Self;

        fn add(mut self, other: W) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::AddAssign<H> for XY {
        fn add_assign(&mut self, other: H) {
            self.y += other;
        }
    }

    impl core::ops::Add<H> for XY {
        type Output = Self;

        fn add(mut self, other: H) -> Self::Output {
            self += other;
            self
        }
    }
}
#[allow(unused_imports)]
use xy::{X, Y, XY, W, H, WH};

pub fn i_to_xy(width: impl Into<TilesWidth>, index: Index) -> XY {
    let width = width.into();
    XY {
        x: xy::x((index % usize::from(width.get())) as _),
        y: xy::y((index / usize::from(width.get())) as _),
    }
}

#[derive(Debug)]
pub enum XYToIError {
    XPastWidth
}

pub fn xy_to_i(width: impl Into<TilesWidth>, xy: XY) -> Result<Index, XYToIError> {
    let width = width.into();
    let width_usize = usize::from(width.get());

    let x_usize = xy.x.usize();
    if x_usize >= width_usize {
        return Err(XYToIError::XPastWidth);
    }

    Ok(xy.y.usize() * width_usize + x_usize)
}

#[derive(Debug)]
pub enum NonEdgeError {
    WidthTooSmall,
    TilesTooShort,
    XYToI(XYToIError),
    TryFromInt(TryFromIntError)
}

impl From<XYToIError> for NonEdgeError {
    fn from(e: XYToIError) -> Self {
        NonEdgeError::XYToI(e)
    }
}

impl From<TryFromIntError> for NonEdgeError {
    fn from(e: TryFromIntError) -> Self {
        NonEdgeError::TryFromInt(e)
    }
}

#[derive(Clone, Copy)]
pub struct XYXY {
    pub min: XY, 
    pub one_past_max: XY,
}

impl XYXY {
    pub fn contains(self, xy: XY) -> bool {
        xy.x >= self.min.x
        && xy.y >= self.min.y
        && xy.x < self.one_past_max.x
        && xy.y < self.one_past_max.y
    }
}

pub fn non_edge_rect(TilesSpec { width, len }: TilesSpec) -> Result<XYXY, NonEdgeError> {
    if width.get() < 3 {
        return Err(NonEdgeError::WidthTooSmall);
    }

    // The min/max non-edge corners; The corners of the rectangle of non-edge pieces.
    let min_corner_xy = xy::XY { x: xy::x(1), y: xy::y(1) };
    let height = xy::Inner::try_from(len)? / width.get();
    if height < 3 {
        return Err(NonEdgeError::TilesTooShort);
    }

    let max_corner_xy = xy::XY { x: xy::x(width.get() - 1), y: xy::y(height - 1) };

    Ok(XYXY{min: min_corner_xy, one_past_max: max_corner_xy})
}

pub type TilesWidthInner = xy::Inner;
pub type TilesWidth = std::num::NonZeroU16;

pub type TileFlags = u8;

const IS_WALL: TileFlags = 1 << 0;

// Suspect we might make a struct later that has TileFlags as a field.
pub type Tile = TileFlags;

pub type Tiles = Grid1<Tile, TilesWidth>;
pub type TilesSpec = Grid1Spec<TilesWidth>;

pub type TileSprite = u16;

const IDLE_OPEN: TileSprite = 0;
const IDLE_SQUINT: TileSprite = 1;
const IDLE_CLOSED: TileSprite = 2;

const LIFT_OPEN: TileSprite = 3;
const LIFT_SQUINT: TileSprite = 4;
const LIFT_CLOSED: TileSprite = 5;
const THUMP: TileSprite = 6;

const FLOOR: TileSprite = 24;
const WALL: TileSprite = 25;
const DIRT: TileSprite = 26;
const BOULDER: TileSprite = 27;

mod player_animation {
    use super::{
        TileSprite,
        IDLE_OPEN,
        IDLE_SQUINT,
        IDLE_CLOSED,
        LIFT_OPEN,
        LIFT_SQUINT,
        LIFT_CLOSED,
        THUMP,
    };

    type IdleState = u8;

    #[derive(Clone, Copy, Debug)]
    pub enum State {
        Idle(IdleState),
    }

    impl Default for State {
        fn default() -> State {
            State::Idle(<_>::default())
        }
    }

    impl State {
        pub fn advance(&mut self) {
            use State::*;
            match self {
                Idle(state) => {
                    *state = state.wrapping_add(1);
                },
            };
        }

        pub fn sprite(&self) -> TileSprite {
            use State::*;

            match self {
                Idle(25) => IDLE_SQUINT,
                Idle(26) => IDLE_CLOSED,
                Idle(27) => IDLE_CLOSED,
                Idle(28) => IDLE_CLOSED,
                Idle(29) => IDLE_SQUINT,

                Idle(49) => IDLE_SQUINT,
                Idle(50) => IDLE_CLOSED,
                Idle(51) => IDLE_CLOSED,
                Idle(52) => IDLE_CLOSED,
                Idle(53) => IDLE_SQUINT,

                Idle(56) => IDLE_SQUINT,
                Idle(57) => IDLE_CLOSED,
                Idle(58) => IDLE_CLOSED,
                Idle(59) => IDLE_CLOSED,
                Idle(60) => IDLE_SQUINT,

                Idle(96) => IDLE_SQUINT,
                Idle(97) => IDLE_CLOSED,
                Idle(98) => IDLE_CLOSED,
                Idle(99) => IDLE_CLOSED,
                Idle(100) => IDLE_SQUINT,


                Idle(134) => LIFT_OPEN,
                Idle(135) => LIFT_OPEN,
                Idle(136) => LIFT_OPEN,
                Idle(137) => LIFT_OPEN,
                Idle(138) => LIFT_OPEN,

                Idle(140) => LIFT_OPEN,
                Idle(141) => LIFT_OPEN,
                Idle(142) => LIFT_OPEN,
                Idle(143) => LIFT_OPEN,
                Idle(144) => LIFT_OPEN,

                Idle(154) => LIFT_OPEN,
                Idle(155) => LIFT_SQUINT,
                Idle(156) => LIFT_CLOSED,
                Idle(157) => LIFT_CLOSED,
                Idle(158) => LIFT_CLOSED,
                Idle(159) => LIFT_CLOSED,
                Idle(160) => LIFT_CLOSED,
                Idle(161) => LIFT_SQUINT,
                Idle(162) => LIFT_OPEN,

                Idle(168) => LIFT_OPEN,
                Idle(169) => LIFT_OPEN,
                Idle(170) => LIFT_OPEN,
                Idle(171) => LIFT_OPEN,
                Idle(172) => LIFT_OPEN,

                Idle(178) => LIFT_OPEN,
                Idle(179) => LIFT_OPEN,
                Idle(180) => LIFT_OPEN,
                Idle(181) => LIFT_OPEN,
                Idle(182) => LIFT_OPEN,

                Idle(213) => LIFT_OPEN,
                Idle(214) => LIFT_SQUINT,
                Idle(215) => LIFT_CLOSED,
                Idle(216) => LIFT_CLOSED,
                Idle(217) => LIFT_CLOSED,
                Idle(218) => LIFT_SQUINT,
                Idle(219) => LIFT_OPEN,

                Idle(223) => LIFT_OPEN,
                Idle(224) => LIFT_SQUINT,
                Idle(225) => LIFT_CLOSED,
                Idle(226) => LIFT_CLOSED,
                Idle(227) => LIFT_CLOSED,
                Idle(228) => LIFT_SQUINT,
                Idle(229) => LIFT_OPEN,

                Idle(s) if *s > 128 => THUMP,
                Idle(_) => IDLE_OPEN,
            }
        }
    }
}

fn can_walk_onto(tiles: &Tiles, xy: XY) -> bool {
    if let Ok(i) = xy_to_i(tiles.width, xy)
    && let Some(tile) = tiles.get(i)
    && tile & IS_WALL == 0 {
        true
    } else {
        false
    }
}

#[derive(Debug)]
enum EdgeHitKind {
    Neither,
    X,
    Y,
    Both
}

fn xy_in_dir(xy: XY, dir: Dir) -> (XY, EdgeHitKind) {
    use Dir::*;

    let x = xy.x;
    let y = xy.y;

    let (new_x, new_y) = match dir {
        Up => (x, y.dec()),
        Right => (x.inc(), y),
        Down => (x, y.inc()),
        Left => (x.dec(), y),
    };

    (
        XY { x: new_x, y: new_y },
        // This can happen due to saturation
        if new_x == x
        && new_y == y {
            EdgeHitKind::Both
        } else if new_x == x && dir.moves_in_x() {
            EdgeHitKind::X
        } else if new_y == y && dir.moves_in_y() {
            EdgeHitKind::Y
        } else {
            EdgeHitKind::Neither
        }
    )
}

#[derive(Clone, Debug)]
pub struct State {
    pub tiles: Tiles,
    pub player_xy: XY,
    pub player_animation_state: player_animation::State,
}

impl State {
    pub fn new(rng: &mut Xs, bold_spec: &sprite::Spec::<sprite::BOLD>) -> Self {
        let (mut max_tile_w, mut max_tile_h) = bold_spec.max_tile_counts();

        if max_tile_w == 0 {
            max_tile_w = 1;
        }
        if max_tile_h == 0 {
            max_tile_h = 1;
        }

        if max_tile_w > xy::Inner::MAX as _ {
            max_tile_w = xy::Inner::MAX as _;
        }
        if max_tile_h > xy::Inner::MAX as _ {
            max_tile_h = xy::Inner::MAX as _;
        }

        let max_tile_w = max_tile_w as xy::Inner;
        let max_tile_h = max_tile_h as xy::Inner;

        let width = TilesWidth::new(max_tile_w).expect("Don't set a 0 width!");

        let length = max_tile_w * max_tile_h;

        let mut tiles = Tiles {
            width,
            cells: vec1![<_>::default(); length],
        };

        let spec = tiles.spec();

        let xyxy = non_edge_rect(spec).expect("Play grid is too small!");

        let mut player_xy = <_>::default();

        for x in 0..max_tile_w {
            for y in 0..max_tile_h {
                let xy = XY{ x: X(x), y: Y(y) };

                if !xyxy.contains(xy) {
                    let Ok(i) = xy_to_i(width, xy) else {
                        continue
                    };

                    tiles.cells[i] |= IS_WALL;
                } else {
                    player_xy = xy; // TODO Pick a random non-wall tile
                }
            }
        }

        Self {
            tiles,
            player_xy,
            player_animation_state: <_>::default(),
        }
    }

    pub fn is_complete(&self) -> bool {
        false
    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        bold_spec: &sprite::Spec::<sprite::BOLD>,
        input: Input,
        _speaker: &mut Speaker,
    ) {
        //
        //
        // Update Section
        //
        //

        if let Some(dir) = input.dir_pressed_this_frame() {
            // Walk
            let (new_xy, _) = xy_in_dir(self.player_xy, dir);

            if can_walk_onto(
                &self.tiles,
                new_xy
            ) {
                self.player_xy = new_xy;
            }
        } else {
            self.player_animation_state.advance();
        }

        //
        //
        // Render Section
        //
        //

        let tile = bold_spec.tile();
        let tile_w = tile.w;
        let tile_h = tile.h;

        //
        // Draw tiles
        //

        for i in 0..self.tiles.cells.len() {
            let tile = self.tiles.cells[i];

            // TODO Seems like it would be faster to avoid the divide in here
            // by iterating over `XY`s instead
            let xy = i_to_xy(self.tiles.width, i);

            let sprite = if tile & IS_WALL == IS_WALL {
                WALL
            } else {
                FLOOR
            };
            
            let base_xy = unscaled::XY {
                x: unscaled::X(unscaled::Inner::from(xy.x.0) * tile_w.get()),
                y: unscaled::Y(unscaled::Inner::from(xy.y.0) * tile_h.get())
            };

            // TODO add the wall sprite and use that here
            commands.sspr(
                bold_spec.xy_from_tile_sprite(sprite),
                command::Rect::from_unscaled(bold_spec.rect(base_xy)),
            );
        }

        //
        // Draw player
        //

        let base_xy = unscaled::XY {
            x: unscaled::X(unscaled::Inner::from(self.player_xy.x.0) * tile_w.get()),
            y: unscaled::Y(unscaled::Inner::from(self.player_xy.y.0) * tile_h.get())
        };

        commands.sspr(
            bold_spec.xy_from_tile_sprite(self.player_animation_state.sprite()),
            command::Rect::from_unscaled(bold_spec.rect(base_xy)),
        );
    }
}