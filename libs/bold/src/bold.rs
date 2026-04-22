///! B.O.L.D.
/// Boldly Or Leisurely Dashing
/// or
/// Boulders Often Lope Downwards

use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use vec1::{Grid1, Grid1Spec, vec1, Vec1};
use xs::Xs;

use std::collections::BTreeMap;
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

const EXIT_BASE: TileSprite = 28;
const EXIT_FRAME_COUNT: TileSprite = 4;

const GEM_BASE: TileSprite = 32;
const GEM_FRAME_COUNT: u8 = 8;

type GemCount = u8;

mod exit_animation {
    use super::{
        Collection,
        GemCount,
        TileSprite,
        EXIT_BASE,
        EXIT_FRAME_COUNT,
    };

    type StateInner = u8;

    const STATE_MAX: StateInner = 6;

    #[derive(Clone, Copy, Debug, Default)]
    pub struct State(StateInner);

    impl State {
        pub fn advance(&mut self, collection: Collection) {
            if collection.current < collection.target {
                return
            }
            if self.0 < STATE_MAX {
                self.0 += 1;
            }
        }

        pub fn sprite(&self) -> TileSprite {
            match self.0 {
                 0 |  1 => EXIT_BASE,
                 2 |  3 => EXIT_BASE + 1,
                 4 |  5 => EXIT_BASE + 2,
                _ => EXIT_BASE + 3,
            }
        }

        pub fn is_open(&self) -> bool {
            self.sprite() == EXIT_BASE + 3
        }
    }
}

mod gem_animation {
    use super::{
        TileSprite,
        Xs,
        GEM_BASE,
        GEM_FRAME_COUNT,
    };

    type StateInner = u16;    

    const STATE_MAX: StateInner = 180;

    #[derive(Clone, Copy, Debug, Default)]
    pub struct State(StateInner);

    pub fn gen_state(rng: &mut Xs) -> State {
        State(
            (xs::range(rng, 0..STATE_MAX as u32)) as StateInner
        )
    }

    impl State {
        pub fn advance(&mut self) {
            self.0 += 1;
            if self.0 >= STATE_MAX {
                self.0 = 0;
            }
        }

        pub fn sprite(&self) -> TileSprite {
            match self.0 {
                 1 |  2 => GEM_BASE + 1,
                 3 |  4 => GEM_BASE + 2,
                 5 |  6 => GEM_BASE + 3,
                 7 |  8 => GEM_BASE + 4,
                 9 | 10 => GEM_BASE + 5,
                11 | 12 => GEM_BASE + 6,
                13 | 14 => GEM_BASE + 7,
                _ => GEM_BASE,
            }
        }
    }
}

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

    type DirState = u8;
    const DIR_STATE_COUNT: DirState = 7;

    const LEFT_SPRITES: [TileSprite; DIR_STATE_COUNT as usize] = [
        8, 9, 10, 11, 12, 13, 14,
    ];

    const RIGHT_SPRITES: [TileSprite; DIR_STATE_COUNT as usize] = [
        16, 17, 18, 19, 20, 21, 22,
    ];

    #[derive(Clone, Copy, Debug)]
    pub enum State {
        Idle(IdleState),
        Left(DirState),
        Right(DirState),
    }

    impl Default for State {
        fn default() -> State {
            State::Idle(<_>::default())
        }
    }

    impl State {
        pub fn idle(&mut self) {
            if let State::Idle(_) = self {
                return
            }

            *self = State::Idle(<_>::default());
        }

        pub fn right(&mut self) {
            if let State::Right(_) = self {
                return
            }

            *self = State::Right(<_>::default());
        }

        pub fn left(&mut self) {
            if let State::Left(_) = self {
                return
            }

            *self = State::Left(<_>::default());
        }

        pub fn advance(&mut self) {
            use State::*;
            match self {
                Idle(state) => {
                    *state = state.wrapping_add(1);
                },
                Left(state) => {
                    *state = state.wrapping_add(1);

                    if *state >= DIR_STATE_COUNT {
                        *state = 0;
                    }
                },
                Right(state) => {
                    *state = state.wrapping_add(1);

                    if *state >= DIR_STATE_COUNT {
                        *state = 0;
                    }
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

                Left(s) => LEFT_SPRITES[(*s) as usize],
                Right(s) => RIGHT_SPRITES[(*s) as usize],
            }
        }
    }

}

fn can_walk_onto(
    tiles: &Tiles,
    mobs: &Mobs,
    xy: XY
) -> bool {
    (
        // Are not blocked by tiles
        if let Ok(i) = xy_to_i(tiles.width, xy)
        && let Some(tile) = tiles.get(i)
        && tile & IS_WALL == 0 {
            true
        } else {
            false
        }
    ) && (
        if let Some(mob) = mobs.get(xy) {
            // Is it a collectable.
            (mob.is_dirt() || mob.is_gem())
            || (mob.is_exit() && mob.exit_animation_state.is_open())
            // TODO checking whether boulder can be pushed
        } else {
            true
        }
    )
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

#[derive(Clone, Debug, Default)]
pub struct Entity {
    pub tile_sprite: TileSprite,
    pub gem_animation_state: gem_animation::State,
    pub exit_animation_state: exit_animation::State,
}

macro_rules! predicates_def {
    (
        $(fn $fn_name: ident($sprite: ident: TileSprite) -> bool $code: block )+
    ) => {
        $(
            fn $fn_name($sprite: TileSprite) -> bool {
                $code
            }
        )+

        impl Entity {
            $(
                fn $fn_name(&self) -> bool {
                    $fn_name(self.tile_sprite)
                }
            )+
        }
    }
}

predicates_def!{
    fn is_exit(s: TileSprite) -> bool {
        s >= EXIT_BASE && s < (EXIT_BASE + EXIT_FRAME_COUNT as TileSprite)
    }

    fn is_dirt(s: TileSprite) -> bool {
        s == DIRT
    }

    fn is_gem(s: TileSprite) -> bool {
        s >= GEM_BASE && s < (GEM_BASE + GEM_FRAME_COUNT as TileSprite)
    }
}

pub type Key = XY;

mod mobs {
    use super::*;

    #[derive(Clone, Debug, Default)]
    pub struct Mobs {
        entities: BTreeMap<Key, Entity>,
    }

    impl Mobs {
        pub fn get(&self, key: Key) -> Option<&Entity> {
            self.entities.get(&key)
        }

        pub fn get_mut(&mut self, key: Key) -> Option<&mut Entity> {
            self.entities.get_mut(&key)
        }

        pub fn remove(&mut self, key: Key) -> Option<Entity> {
            self.entities.remove(&key)
        }

        pub fn insert(&mut self, key: Key, entity: Entity) {
            self.entities.insert(
                key,
                entity
            );
        }

        pub fn entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
            self.entities.values_mut()
        }

        pub fn all(&self) -> impl Iterator<Item = (&Key, &Entity)> {
            self.entities.iter()
        }

        pub fn keys(&self) -> impl Iterator<Item = &Key> {
            self.entities.keys()
        }


    }
}
use mobs::Mobs;

#[derive(Clone, Copy, Debug)]
pub struct Collection {
    pub current: GemCount,
    pub target: GemCount,
}

#[derive(Clone, Debug)]
pub struct State {
    pub rng: Xs,
    pub tiles: Tiles,
    pub mobs: Mobs,
    pub player_xy: XY,
    pub player_animation_state: player_animation::State,
    pub collection: Collection,
    pub left_was_last_x_dir_pressed: bool,
    pub gem_hud_buffer: String,
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

        for x in 0..max_tile_w {
            for y in 0..max_tile_h {
                let xy = XY{ x: X(x), y: Y(y) };

                if !xyxy.contains(xy) {
                    let Ok(i) = xy_to_i(width, xy) else {
                        continue
                    };

                    tiles.cells[i] |= IS_WALL;
                }
            }
        }

        let mut mobs = Mobs::default();

        let mut placed: GemCount = 0;

        for x in 0..max_tile_w {
            for y in 0..max_tile_h {
                if xs::range(rng, 0..4) > 0 {
                    continue
                }

                let xy = XY{ x: X(x), y: Y(y) };

                let Ok(i) = xy_to_i(width, xy) else {
                    continue
                };

                if tiles.cells[i] & IS_WALL == IS_WALL {
                    continue
                }

                if mobs.get(xy).is_some() {
                    continue
                }

                if xs::range(rng, 0..2) > 0 {
                    mobs.insert(
                        xy,
                        Entity {
                            tile_sprite: GEM_BASE,
                            gem_animation_state: gem_animation::gen_state(rng),
                            exit_animation_state: <_>::default(),
                        }
                    );
                    placed += 1;
                } else {
                    mobs.insert(
                        xy,
                        Entity {
                            tile_sprite: BOULDER,
                            gem_animation_state: <_>::default(),
                            exit_animation_state: <_>::default(),
                        }
                    );
                }
            }
        }

        let mut exit_i = xs::range(rng, 0..length as u32) as usize;
        let mut exit_xy;
        // Assignment is meant here
        while { exit_xy = i_to_xy(width, exit_i); false }
        || tiles.cells[exit_i] & IS_WALL == IS_WALL
        || mobs.get(exit_xy).is_some()
        {
            exit_i += 1;
            if exit_i >= length as usize {
                exit_i = 0;
            }
        }

        mobs.insert(
            exit_xy,
            Entity {
                tile_sprite: EXIT_BASE,
                gem_animation_state: <_>::default(),
                exit_animation_state: <_>::default(),
            }
        );

        let mut player_i = xs::range(rng, 0..length as u32) as usize;
        let mut player_xy;

        // Assignment is meant here
        while { player_xy = i_to_xy(width, player_i); false }
        || tiles.cells[player_i] & IS_WALL == IS_WALL
        || mobs.get(player_xy).is_some()
        {
            player_i += 1;
            if player_i >= length as usize {
                player_i = 0;
            }
        }

        for x in 0..max_tile_w {
            for y in 0..max_tile_h {
                let xy = XY{ x: X(x), y: Y(y) };

                if xy == player_xy {
                    continue
                }

                if mobs.get(xy).is_some() {
                    continue
                }

                let Ok(i) = xy_to_i(width, xy) else {
                    continue
                };

                if tiles.cells[i] & IS_WALL == IS_WALL {
                    continue
                }

                mobs.insert(
                    xy,
                    Entity {
                        tile_sprite: DIRT,
                        gem_animation_state: <_>::default(),
                        exit_animation_state: <_>::default(),
                    }
                );
            }
        }

        // TODO Display Have gems / Need gems
        // TODO place the exit and player in such a way that guarantees the level is solvable
        //    Presumably by tracing a path past the rocks and placing the player and exit on the ends
        //        Or maybe trace a path, then place the rocks?
        // TODO implment enemies; place them sparsely, and not along the path traced to place other things

        Self {
            rng: xs::from_seed(xs::new_seed(rng)),
            tiles,
            mobs,
            collection: Collection {
                current: 0,
                target: placed / 2,
            },
            player_xy,
            player_animation_state: <_>::default(),
            left_was_last_x_dir_pressed: false,
            gem_hud_buffer: String::with_capacity(16),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.mobs.get(self.player_xy)
            .map(|mob| mob.exit_animation_state.is_open())
            .unwrap_or(false)
    }

    fn is_clear(&self, xy: XY) -> bool {
        xy != self.player_xy
        && {
            let Ok(i) = xy_to_i(self.tiles.width, xy) else { return false };

            self.tiles.get(i).map(|t| t & IS_WALL == 0).unwrap_or(false)
            && self.mobs.get(xy).is_none()
        }
    }

    fn apply_gravity(&mut self, xy: XY) {
        if let (below, EdgeHitKind::Neither) = xy_in_dir(xy, Dir::Down)
        && self.is_clear(below)
        && let Some(mob) = self.mobs.remove(xy)
        {
            // Falling downward

            // TODO? Instead, collect all movments then apply them all afterwards?
            // If we do, avoid the `collect` call above.
            self.mobs.insert(below, mob);
        } else if let left_down = {
            if let (left, EdgeHitKind::Neither) = xy_in_dir(xy, Dir::Left)
            && let (left_down, EdgeHitKind::Neither) = xy_in_dir(left, Dir::Down)
            {
                (self.is_clear(left) && self.is_clear(left_down)).then_some(left_down)
            } else {
                None
            }
        }
        && let right_down = {
            if let (right, EdgeHitKind::Neither) = xy_in_dir(xy, Dir::Right)
            && let (right_down, EdgeHitKind::Neither) = xy_in_dir(right, Dir::Down)
            {
                (self.is_clear(right) && self.is_clear(right_down)).then_some(right_down)
            } else {
                None
            }
        }
        && (left_down.is_some() || right_down.is_some())
        && let Some(mob) = self.mobs.remove(xy)
        {
            // Rolling to the side of a pile
            // TODO Go all the way until we hit something, and count the distance to decide
            //      which way to roll?
            match (left_down, right_down) {
                (Some(target), _) => {
                    // Roll left first. At the moment, determinism seems better than randomness.
                    self.mobs.insert(target, mob);
                },
                (_, Some(target)) => {
                    self.mobs.insert(target, mob);
                },
                (None, None) => unreachable!("We checked this already"),
            }
        } else {
            // Stay still
        }
    }

    fn tick(&mut self) {
        let keys = self.mobs.keys().cloned().collect::<Vec<Key>>();

        for xy in keys {
            let Some(current_mob) = self.mobs.get(xy).map(|t| t.tile_sprite) else {
                continue
            };

            match current_mob {
                t_s if is_gem(t_s) => {
                    if let Some(mut mob) = self.mobs.remove(xy) {
                        mob.gem_animation_state.advance();

                        self.mobs.insert(xy, mob);
                    }

                    self.apply_gravity(xy);
                }
                t_s if is_exit(t_s) => {
                    if let Some(mut mob) = self.mobs.remove(xy) {
                        mob.exit_animation_state.advance(self.collection);

                        self.mobs.insert(xy, mob);
                    }
                }
                DIRT => {}
                BOULDER => {
                    self.apply_gravity(xy);
                }
                _ => { debug_assert!(false, "Unhandled mob kind in tick"); }
            }
        }
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

        if let Some(dir) = input.dir_pressed_this_frame()
        // Walk
        && let (new_xy, _) = xy_in_dir(self.player_xy, dir)
        && can_walk_onto(&self.tiles, &self.mobs, new_xy)
        {
            self.player_xy = new_xy;
            if dir == Dir::Left {
                self.left_was_last_x_dir_pressed = true;
            }
            if dir == Dir::Right {
                self.left_was_last_x_dir_pressed = false;
            }

            if self.left_was_last_x_dir_pressed {
                self.player_animation_state.left()
            } else {
                self.player_animation_state.right()
            }

            match self.mobs.remove(self.player_xy) {
                Some(mob) if mob.is_dirt() => {}, // Expected
                Some(mob) if mob.is_gem() => {
                    self.collection.current += 1;
                },
                Some(mob) if mob.is_exit() => {
                    self.mobs.insert(self.player_xy, mob);
                },
                Some(mob) => {
                    debug_assert!(false, "Unexpected mob type removed! {mob:?}");
                }
                None => {} // Expected
            }
        } else if input.contains_dir().is_none() {
            self.player_animation_state.idle();
        } else {
            self.player_animation_state.advance();
        }

        self.tick();

        //
        //
        // Render Section
        //
        //

        let tile = bold_spec.tile();
        let tile_w = tile.w;
        let tile_h = tile.h;

        let mut draw_tile_sprite = |xy: XY, sprite: TileSprite| {
            let base_xy = unscaled::XY {
                x: unscaled::X(unscaled::Inner::from(xy.x.0) * tile_w.get()),
                y: unscaled::Y(unscaled::Inner::from(xy.y.0) * tile_h.get())
            };

            commands.sspr(
                bold_spec.xy_from_tile_sprite(sprite),
                command::Rect::from_unscaled(bold_spec.rect(base_xy)),
            );
        };

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

            draw_tile_sprite(xy, sprite);
        }

        //
        // Draw Mobs
        //

        for (&xy, mob) in self.mobs.all() {
            let sprite = if mob.is_gem() {
                mob.gem_animation_state.sprite()
            } else if mob.is_exit() {
                mob.exit_animation_state.sprite()
            } else {
                mob.tile_sprite
            };
            draw_tile_sprite(xy, sprite);
        }

        //
        // Draw player
        //

        draw_tile_sprite(self.player_xy, self.player_animation_state.sprite());

        //
        // Draw HUD
        //

        use std::fmt::Write;
        self.gem_hud_buffer.clear();
        // This doesn't actually fail for strings.
        let _ = write!(&mut self.gem_hud_buffer, "{} / {}", self.collection.current, self.collection.target);
        commands.print_line(
            self.gem_hud_buffer.as_bytes(),
            unscaled::XY { x: unscaled::X(1), y: unscaled::Y(1) },
            6
        );
    }
}