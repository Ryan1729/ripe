///! S.W.O.R.D.: Staff Whacking Ordeal Required, Duh

use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use vec1::{Vec1, vec1};
use xs::Xs;

use std::collections::{BTreeMap, HashMap};
use std::num::{NonZeroU8, NonZeroU16};

type Index = usize;

/*
    Notes on wall/floor tiles and their various edge types:

    There are tiles that cannot be walked on, called wall tiles,
    and there are ones that can be walked on, called floor tiles.

    We want to have interesting looking edges between wall tiles
    and floor tiles, including visible corners, while ultimately
    rendering things in tiles that abut each other exactly.

    Another property we want is to not have any disconnected edges
    of the patterns on the tiles. Said another way, for any two
    possible tiles at positions (X, Y) and (X + 2, Y), a tile must
    exist to place at position (X + 1, Y) that lines up with the
    relevant edges of the first two tiles. This applies not just
    for (X, Y) and (X + 2, Y), but for any pair of tile that are
    two orthogonal steps away from each other, and all the tiles
    one can cross using those two steps. For example, there must
    be tiles placeable to connect (X, Y) and (X - 1, Y + 1), at
    both (X - 1, Y) and (X, Y + 1).

    We have thus far declined to implement rotation in the
    renderer, so we need distinct tiles for each possible rotation.
    (Possibly this aspect will cause us to decide to actually
    implement rotation.)

    There are two types of tile, as mentioend before: wall and floor.
    The edges of a tile are determined by the type of its eight
    neighbors, and the type of the tile itself. This is nine bits of
    possible variance and thus 512 distinct tiles!

    It seems plausible that some kind of pattern would emerge that
    makes a smaller number of tiles work, even without rotation, but
    it's not clear at this time.

    A data structure for an index into these tiles seems clear though:
*/
pub type NeighborMask = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileIndex {
    Wall(NeighborMask),
    Floor,
}

impl Default for TileIndex {
    fn default() -> Self {
        Self::Wall(<_>::default())
    }
}

impl TileIndex {
    pub fn is_floor(self) -> bool {
        match self {
            TileIndex::Wall(..) => false,
            TileIndex::Floor => true,
        }
    }
}

pub type NeighborFlag = NonZeroU8;

// SAFETY: The value is not 0.
pub const UPPER_LEFT: NeighborFlag = unsafe { NeighborFlag::new_unchecked(1 << 0) };
// SAFETY: The value is not 0.
pub const UPPER_MIDDLE: NeighborFlag = unsafe { NeighborFlag::new_unchecked(1 << 1) };
// SAFETY: The value is not 0.
pub const UPPER_RIGHT: NeighborFlag = unsafe { NeighborFlag::new_unchecked(1 << 2) };
// SAFETY: The value is not 0.
pub const LEFT_MIDDLE: NeighborFlag = unsafe { NeighborFlag::new_unchecked(1 << 3) };
// SAFETY: The value is not 0.
pub const RIGHT_MIDDLE: NeighborFlag = unsafe { NeighborFlag::new_unchecked(1 << 4) };
// SAFETY: The value is not 0.
pub const LOWER_LEFT: NeighborFlag = unsafe { NeighborFlag::new_unchecked(1 << 5) };
// SAFETY: The value is not 0.
pub const LOWER_MIDDLE: NeighborFlag = unsafe { NeighborFlag::new_unchecked(1 << 6) };
// SAFETY: The value is not 0.
pub const LOWER_RIGHT: NeighborFlag = unsafe { NeighborFlag::new_unchecked(1 << 7) };

/*
    ... So one path to investigate this, without actually making 512
    separate tiles, would be to make the space for them, then write the
    indexing code, and then attempt to render the 512 apparently separate
    tiles in a test, filling them in as needed, and see if any turn out
    to be the same.
*/

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

    impl From<XY> for offset::Point {
        fn from(XY { x, y }: XY) -> Self {
            (offset::Inner::from(x.0), offset::Inner::from(y.0))
        }
    }

    impl XY {
        pub fn checked_push(self, dir: impl Into<crate::Dir8>) -> Option<XY> {
            use crate::Dir8::*;
            Some(match dir.into() {
                UpLeft => XY { x: self.x.checked_sub(W::ONE)?, y: self.y.checked_sub(H::ONE)? },
                Up => XY { x: self.x, y: self.y.checked_sub(H::ONE)? },
                UpRight => XY { x: self.x.checked_add(W::ONE)?, y: self.y.checked_sub(H::ONE)? },
                Right => XY { x: self.x.checked_add(W::ONE)?, y: self.y },
                DownRight => XY { x: self.x.checked_add(W::ONE)?, y: self.y.checked_add(H::ONE)? },
                Down => XY { x: self.x, y: self.y.checked_add(H::ONE)? },
                DownLeft => XY { x: self.x.checked_sub(W::ONE)?, y: self.y.checked_add(H::ONE)? },
                Left => XY { x: self.x.checked_sub(W::ONE)?, y: self.y },
            })
        }
    }

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

struct FromTo<A> {
    from: A,
    to: A,
}

fn dir_to(FromTo { from, to }: FromTo<XY>) -> Option<Dir8> {
    let x_diff: xy::Diff = xy::Diff::from(to.x) - xy::Diff::from(from.x);
    let y_diff: xy::Diff = xy::Diff::from(to.y) - xy::Diff::from(from.y);

    use std::cmp::Ordering::*;

    match (x_diff.cmp(&0), y_diff.cmp(&0)) {
        (Less, Less) => Some(Dir8::UpLeft),
        (Less, Equal) => Some(Dir8::Left),
        (Less, Greater) => Some(Dir8::DownLeft),
        (Equal, Less) => Some(Dir8::Up),
        (Equal, Equal) => None,
        (Equal, Greater) => Some(Dir8::Down),
        (Greater, Less) => Some(Dir8::UpRight),
        (Greater, Equal) => Some(Dir8::Right),
        (Greater, Greater) => Some(Dir8::DownRight),
    }
}

type SwordTileSpriteInner = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileSprite {
    Sword(SwordTileSpriteInner),
    ToggleWall(NeighborMask)
}

impl Default for TileSprite {
    fn default() -> Self {
        TileSprite::Sword(<_>::default())
    }
}

impl TileSprite {
    const fn sword_inner_or_0(self) -> SwordTileSpriteInner {
        match self {
            TileSprite::Sword(inner) => inner,
            _ => 0,
        }
    }

    fn is_stairs(&self) -> bool {
        for tile_sprite in EXIT_SPRITES {
            if self.sword_inner_or_0() == tile_sprite.sword_inner_or_0() { return true }
        }

        false
    }
}

const PLAYER_BASE: TileSprite = TileSprite::Sword(0);
const STAFF_BASE: TileSprite = TileSprite::Sword(1);
const DOWN_STAIRS_TOP_LEFT_EDGE: TileSprite = TileSprite::Sword(2);
#[allow(unused)]
const DOWN_STAIRS_TOP_EDGE: TileSprite = TileSprite::Sword(DOWN_STAIRS_TOP_LEFT_EDGE.sword_inner_or_0() + 1);
const DOWN_STAIRS_TOP_RIGHT_EDGE: TileSprite = TileSprite::Sword(DOWN_STAIRS_TOP_LEFT_EDGE.sword_inner_or_0() + 2);

const UP_STAIRS_TOP_LEFT_EDGE: TileSprite = TileSprite::Sword(7);
#[allow(unused)]
const UP_STAIRS_TOP_EDGE: TileSprite = TileSprite::Sword(UP_STAIRS_TOP_LEFT_EDGE.sword_inner_or_0() + 1);
const UP_STAIRS_TOP_RIGHT_EDGE: TileSprite = TileSprite::Sword(UP_STAIRS_TOP_LEFT_EDGE.sword_inner_or_0() + 2);

const ROACH_BASE: TileSprite = TileSprite::Sword(12);

const RIGHT_STAIRS_TOP_EDGE: TileSprite = TileSprite::Sword(45);
const RIGHT_STAIRS_MIDDLE_EDGE: TileSprite = TileSprite::Sword(50);
const RIGHT_STAIRS_BOTTOM_EDGE: TileSprite = TileSprite::Sword(55);

const LEFT_STAIRS_TOP_LEFT_EDGE: TileSprite = TileSprite::Sword(46);
const LEFT_STAIRS_MIDDLE_EDGE: TileSprite = TileSprite::Sword(51);
const LEFT_STAIRS_BOTTOM_EDGE: TileSprite = TileSprite::Sword(56);

const SWITCH_BASE: TileSprite = TileSprite::Sword(40);
const SWITCH_HIT: TileSprite = TileSprite::Sword(SWITCH_BASE.sword_inner_or_0() + 1);

const EXIT_SPRITES: [TileSprite; 12] = [
    UP_STAIRS_TOP_LEFT_EDGE,
    UP_STAIRS_TOP_EDGE,
    UP_STAIRS_TOP_RIGHT_EDGE,

    DOWN_STAIRS_TOP_LEFT_EDGE,
    DOWN_STAIRS_TOP_EDGE,
    DOWN_STAIRS_TOP_RIGHT_EDGE,

    LEFT_STAIRS_TOP_LEFT_EDGE,
    LEFT_STAIRS_MIDDLE_EDGE,
    LEFT_STAIRS_BOTTOM_EDGE,

    RIGHT_STAIRS_TOP_EDGE,
    RIGHT_STAIRS_MIDDLE_EDGE,
    RIGHT_STAIRS_BOTTOM_EDGE,
];

const UP_STAIRS_TOP_LEFT_EDGE_INDEX: Index = 0;
const DOWN_STAIRS_TOP_LEFT_EDGE_INDEX: Index = 3;
const LEFT_STAIRS_TOP_LEFT_EDGE_INDEX: Index = 6;
const RIGHT_STAIRS_TOP_LEFT_EDGE_INDEX: Index = 9;

pub type TileFlags = u16;
pub const TILE_REQUIRED: TileFlags = 1 << 0;

fn to_tile_flags(proto_tile_flags: ProtoTileFlags) -> TileFlags {
    TileFlags::from(proto_tile_flags >> 4)
}

#[test]
fn to_tile_flags_maps_skip_to_tile_required() {
    assert_eq!(to_tile_flags(SKIP), TILE_REQUIRED);
}

#[test]
fn to_tile_flags_maps_all_dir_flags_to_0() {
    let mut all_dir_flags = 0;
    for dir in Dir::ALL {
        all_dir_flags |= dir.flag();
    }

    assert_eq!(to_tile_flags(all_dir_flags), 0);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tile {
    pub sprite_index: TileIndex,
    pub flags: TileFlags,
}

impl Tile {
    pub fn is_floor(&self) -> bool {
        self.sprite_index.is_floor()
    }
}

mod position {
    use super::XY;

    #[derive(Clone, Copy, Debug, Default)]
    pub struct Position {
        xy: XY,
        offset: offset::XY,
    }

    impl From<XY> for Position {
        fn from(xy: XY) -> Self {
            Self {
                xy,
                ..<_>::default()
            }
        }
    }

    impl Position {
        pub fn new(
            xy: XY,
            offset: offset::XY,
        ) -> Self {
            Self {
                xy,
                offset,
            }
        }

        pub fn xy(&self) -> XY {
            self.xy
        }

        pub fn set_xy(&mut self, xy: XY) {
            self.offset = offset::XY::from_old_and_new(
                offset::Point::from(self.xy),
                offset::Point::from(xy),
            );
            self.xy = xy;
        }

        pub fn offset(&self) -> offset::XY {
            self.offset
        }

        pub fn decay(&mut self) {
            self.offset.decay();
        }
    }
}
use position::Position;

type ToggleWallSpecWidth = NonZeroU8;

/// We have the default as wall elsewhere, so let's be consistent.
type IsFloorFlag = bool;
const IS_WALL: IsFloorFlag = false;
const IS_FLOOR: IsFloorFlag = true;

pub struct ToggleWallSpec {
    pub width: ToggleWallSpecWidth,
    // TODO? pack these tightly? Does this live long enough for us to care?
    pub tiles: Vec1<IsFloorFlag>,
    pub base_wh: WH,
}

pub type ToggleGroupId = u8;
const NULL_GROUP: ToggleGroupId = 0;
const FIRST_GROUP: ToggleGroupId = 1;

pub type EntityFlags = u8;

const GONE: EntityFlags = 1 << 0;
const RENDER_FACING: EntityFlags = 1 << 1;

#[derive(Clone, Debug, Default)]
pub struct Entity {
    pub tile_sprite: TileSprite,
    pub toggle_group_id: ToggleGroupId,
    pub flags: EntityFlags,
    pub facing: Dir8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub xy: XY,
}

mod entities {
    use super::*;

    pub struct Mutation {
        pub key: Key,
        pub facing: Dir8,
        pub target_xy: XY,
    }

    #[derive(Clone, Debug, Default)]
    struct Entry {
        offset: offset::XY,
        entity: Entity,
    }

    #[derive(Clone, Debug, Default)]
    pub struct Entities {
        entities: BTreeMap<Key, Entry>,
    }

    impl Entities {
        pub fn get(&self, key: Key) -> Option<&Entity> {
            self.entities.get(&key).map(|entry| &entry.entity)
        }

        pub fn get_mut(&mut self, key: Key) -> Option<&mut Entity> {
            self.entities.get_mut(&key).map(|entry| &mut entry.entity)
        }

        pub fn insert(&mut self, key: Key, entity: Entity) {
            self.entities.insert(
                key, 
                Entry { offset: <_>::default(), entity }
            );
        }

        pub fn remove(&mut self, key: Key) {
            self.entities.remove(&key);
        }

        pub fn decay_positions(&mut self) {
            for Entry { offset, .. } in self.entities.values_mut() {
                offset.decay();
            }
        }

        pub fn all(&self) -> impl Iterator<Item = (Position, &Entity)> {
            self.entities.iter()
                .map(|(key, Entry { offset, entity })| {
                    (Position::new(key.xy, *offset), entity)
                })
        }

        pub fn entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
            self.entities.values_mut()
                .map(|Entry { entity, .. }| {
                    entity
                })
        }

        pub fn apply(&mut self, mutation: Mutation) {
            if let Some(Entry { entity, .. }) = self.entities.get_mut(&mutation.key) {
                entity.facing = mutation.facing;
            }

            let old_key = mutation.key;
            let new_key = Key { xy: mutation.target_xy };
            if old_key.xy != new_key.xy {
                if let None = self.entities.get(&new_key)
                && let Some(Entry { entity, .. }) = self.entities.remove(&old_key) {
                    self.entities.insert(
                        new_key,
                        Entry {
                            offset: offset::XY::from_old_and_new(
                                offset::Point::from(old_key.xy),
                                offset::Point::from(new_key.xy),
                            ),
                            entity
                        }
                    );
                }
            }
        }
    }
}
use entities::Entities;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dir8 {
    #[default]
    UpLeft,
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
}

impl Dir8 {
    fn index(self) -> u8 {
        use Dir8::*;

        match self {
            UpLeft => 0,
            Up => 1,
            UpRight => 2,
            Right => 3,
            DownRight => 4,
            Down => 5,
            DownLeft => 6,
            Left => 7,
        }
    }

    fn clockwise(self) -> Dir8 {
        use Dir8::*;

        match self {
            UpLeft => Up,
            Up => UpRight,
            UpRight => Right,
            Right => DownRight,
            DownRight => Down,
            Down => DownLeft,
            DownLeft => Left,
            Left => UpLeft,
        }
    }

    fn counter_clockwise(self) -> Dir8 {
        use Dir8::*;

        match self {
            UpLeft => Left,
            Up => UpLeft,
            UpRight => Up,
            Right => UpRight,
            DownRight => Right,
            Down => DownRight,
            DownLeft => Down,
            Left => DownLeft,
        }
    }

    fn moves_in_x(self) -> bool {
        use Dir8::*;

        match self {
            UpLeft | UpRight | Right | DownRight | DownLeft | Left => true,
            Up | Down => false,
        }
    }

    fn moves_in_y(self) -> bool {
        use Dir8::*;

        match self {
            UpLeft | Up | UpRight | DownRight | Down | DownLeft => true,
            Right | Left => false,
        }
    }
}

impl From<Dir> for Dir8 {
    fn from(dir: Dir) -> Self {
        use Dir8::*;

        match dir {
            Dir::Up => Up,
            Dir::Right => Right,
            Dir::Down => Down,
            Dir::Left => Left,
        }
    }
}

#[derive(Debug)]
enum EdgeHitKind {
    Neither,
    X,
    Y,
    Both
}

fn xy_in_dir(xy: XY, dir: Dir8) -> (XY, EdgeHitKind) {
    use Dir8::*;

    let x = xy.x;
    let y = xy.y;

    let (new_x, new_y) = match dir {
        UpLeft => (x.dec(), y.dec()),
        Up => (x, y.dec()),
        UpRight => (x.inc(), y.dec()),
        Right => (x.inc(), y),
        DownRight => (x.inc(), y.inc()),
        Down => (x, y.inc()),
        DownLeft => (x.dec(), y.inc()),
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

pub fn i_to_xy(width: impl Into<TilesWidth>, index: usize) -> XY {
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

pub fn xy_to_i(width: impl Into<TilesWidth>, xy: XY) -> Result<usize, XYToIError> {
    let width = width.into();
    let width_usize = usize::from(width.get());

    let x_usize = xy.x.usize();
    if x_usize >= width_usize {
        return Err(XYToIError::XPastWidth);
    }

    Ok(xy.y.usize() * width_usize + x_usize)
}

pub type TilesWidthInner = u16;
pub type TilesWidth = NonZeroU16;

#[derive(Clone, Debug)]
pub struct Tiles {
    pub width: TilesWidth,
    pub tiles: Vec1<Tile>
}

fn can_walk_onto_tile(tiles: &Tiles, xy: XY) -> bool {
    let Ok(i) = xy_to_i(tiles.width, xy) else {
        return false
    };

    tiles.tiles.get(i)
        .map(|t| t.is_floor())
        .unwrap_or(false)
}

fn can_walk_onto(mobs: &Entities, tiles: &Tiles, key: Key) -> bool {
    can_walk_onto_tile(tiles, key.xy) && {
        match mobs.get(key) {
            Some(mob) => {
                // Can walk onto things that are gone.
                (mob.flags & GONE) == GONE
                || mob.tile_sprite.is_stairs()
            },
            None => true,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AnimationKind {
    Reset,
}

pub type Frames = u16;

#[derive(Clone, Debug)]
pub struct Animation {
    pub kind: AnimationKind,
    pub target_key: Key,
    pub frames_left: Frames,
}

const RESET_ANIMATION_LENGTH: Frames = 12;

impl Animation {
    pub fn reset(target_key: Key) -> Self {
        Self {
            kind: AnimationKind::Reset,
            target_key,
            frames_left: RESET_ANIMATION_LENGTH,
        }
    }
}

pub type Animations = Vec<Animation>;

pub type TilesLength = usize;

pub struct Sizes {
    pub tiles_width: TilesWidth,
    pub tiles_length: TilesLength,
    pub proto_width: ProtoTilesWidth,
    pub proto_length: TilesLength,
}

impl Sizes {
    pub fn new(w: u16, h: u16) -> Self {
        let tiles_length = (w * h).into();

        let proto_width = ProtoTilesWidth(TilesWidth::new((w / 2).saturating_sub(1)).unwrap_or(TilesWidth::MIN));
        let proto_height = TilesWidth::new((h / 2).saturating_sub(1)).unwrap_or(TilesWidth::MIN);
        let proto_length = usize::from(proto_width.get()) * usize::from(proto_height.get());

        let tiles_width = TilesWidth::new(w).unwrap_or(TilesWidth::MIN);

        Sizes {
            tiles_width,
            tiles_length,
            proto_width,
            proto_length,
        }
    }
}

type ProtoTileFlags = u8;

/// A flag that is outside the range of the Dir flags, which is meant to indicate that the given cell
/// should not be filled at all.
const SKIP: ProtoTileFlags = 1 << (Dir::ALL.len());

fn maze_via_backtracking(
    proto_tiles: &mut [ProtoTileFlags],
    rng: &mut Xs,
    proto_width: ProtoTilesWidth,
    current_xy: XY
) {
    let mut dirs = Dir::ALL;
    xs::shuffle(rng, &mut dirs);

    for dir in dirs {
        if let Some(new_xy) = current_xy.checked_push(dir) {
            if let (Ok(current_index), Ok(new_index))
                = (xy_to_i(proto_width.0, current_xy), xy_to_i(proto_width.0, new_xy))
            {
                if let Ok([flags, adjacent_flags])
                    = proto_tiles.get_disjoint_mut([current_index, new_index])
                {
                    // Don't revisit previously visited spots
                    if *adjacent_flags != 0 { continue }

                    *flags |= dir.flag();
                    *adjacent_flags |= dir.opposite().flag();
                    maze_via_backtracking(proto_tiles, rng, proto_width, new_xy);
                }
            }
        }
    }
}

#[allow(unused)]
fn print_proto_tiles(
    tiles: &[ProtoTileFlags],
    ProtoTilesWidth(width): ProtoTilesWidth,
) {
    let mut output = String::with_capacity(tiles.len());

    output.push(' ');
    for _ in 0..(width.get() * 2 - 1) {
        output.push('_');
    }
    output.push('\n');

    let height = calc_height(width, tiles);

    for y in 0..height {
        output.push('|');
        for x in 0..width.get() {
            let xy = XY { x: xy::x(x), y: xy::y(y) };

            let Ok(i) = xy_to_i(width, xy) else { continue };

            let tile = tiles[i];

            output.push(if tile & Dir::Down.flag() != 0 { ' ' } else { '_' });

            if tile & Dir::Right.flag() != 0 {
                output.push(
                    if (tile | tiles.get(i + 1).cloned().unwrap_or(0)) & Dir::Down.flag() != 0 {
                        ' '
                    } else {
                        '_'
                    }
                );
            } else {
                output.push('|');
            }
        }

        output.push('\n');
    }

    eprintln!("{output}");
}

#[cfg(test)]
mod maze_via_backtracking_connects_all_cells_on {
    use super::*;

    pub(crate) fn are_all_cells_connected_options(
        proto_tiles: &mut Vec1<ProtoTileFlags>,
        width: ProtoTilesWidth,
        skip_mask: ProtoTileFlags,
    ) -> bool {
        print_proto_tiles(proto_tiles, width);
        use std::collections::HashSet;
        let mut seen = HashSet::with_capacity(proto_tiles.len());

        let mut to_see = vec![XY::default()];

        while let Some(xy) = to_see.pop() {
            if let Ok(i) = xy_to_i(width, xy) {
                let tile = proto_tiles[i];

                if tile & skip_mask != 0 { continue }

                // Don't even look at ones that should be skipped.
                seen.insert(i);

                for dir in Dir::ALL {
                    if tile & dir.flag() != 0
                    && let Some(new_xy) = xy.checked_push(dir)
                    && let Ok(new_i) = xy_to_i(width, new_xy)
                    && new_i < proto_tiles.len()
                    && !seen.contains(&new_i) {
                        to_see.push(new_xy);
                    }
                }
            }
        }

        let mut skip_count = 0;

        for i in 0..proto_tiles.len() {
            let tile = proto_tiles[i];

            if tile & skip_mask != 0 { skip_count += 1 }
        }

        seen.len() == (proto_tiles.len() - skip_count)
    }

    pub(crate) fn are_all_cells_connected(
        proto_tiles: &mut Vec1<ProtoTileFlags>,
        width: ProtoTilesWidth,
    ) -> bool {
        are_all_cells_connected_options(proto_tiles, width, 0)
    }

    // Test predicate test
    #[test]
    fn are_all_cells_connected_returns_false_sometimes() {
        use Dir::*;

        let width = ProtoTilesWidth::new(4).unwrap();

        let rd = Right.flag() | Down.flag();
        let ru = Right.flag() | Up.flag();
        let rl = Right.flag() | Left.flag();
        let ld =  Left.flag() | Down.flag();
        let lu =  Left.flag() | Up.flag();

        // All walls
        let mut tiles = vec1![0; 16usize];

        assert!(!are_all_cells_connected(&mut tiles, width));

        // Top half
        let mut tiles = vec1![
            rd, rl, rl, ld,
            ru, rl, rl, lu,
             0,  0,  0,  0,
             0,  0,  0,  0,
        ];

        assert!(!are_all_cells_connected(&mut tiles, width));

        // Disjoint top and bottom
        let mut tiles = vec1![
            rd, rl, rl, ld,
            ru, rl, rl, lu,

            rd, rl, rl, ld,
            ru, rl, rl, lu,
        ];

        assert!(!are_all_cells_connected(&mut tiles, width));
    }

    #[test]
    fn are_all_cells_connected_options_respects_the_skip_flag() {
        use Dir::*;

        let f = Up.flag() | Down.flag() | Right.flag() | Left.flag();

        let width = ProtoTilesWidth::new(4).unwrap();

        // All floor
        let mut tiles = vec1![f; 16usize];

        assert!(are_all_cells_connected_options(&mut tiles, width, SKIP));

        // Top half
        let mut tiles = vec1![
             f,  f,  f,  f,
             f,  f,  f,  f,
             SKIP,  SKIP,  SKIP,  SKIP,
             SKIP,  SKIP,  SKIP,  SKIP,
        ];

        assert!(are_all_cells_connected_options(&mut tiles, width, SKIP));

        // Disjoint top and bottom
        let mut tiles = vec1![
            f,  f,  f,  f,

            SKIP,  SKIP,  SKIP,  SKIP,
            SKIP,  SKIP,  SKIP,  SKIP,

            f,  f,  f,  f,
        ];

        assert!(!are_all_cells_connected_options(&mut tiles, width, SKIP));
    }

    #[test]
    fn this_small_example() {
        let width = ProtoTilesWidth::new(10).unwrap();
        let mut tiles = vec1![0; 100usize];
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        assert!(!are_all_cells_connected(&mut tiles, width));

        maze_via_backtracking(&mut tiles, &mut rng, width, <_>::default());

        assert!(are_all_cells_connected(&mut tiles, width));
    }
}

mod random {
    use super::*;
    use std::num::TryFromIntError;

    pub type Index = usize;

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

    // `(min_corner_xy, max_corner_xy)` Inclusive in both cases.
    // TODO? Is there a struct we already have in this project that would work here?
    pub type Rect = (XY, XY);

    pub fn non_edge_rect(width: TilesWidth, tiles_len: Index) -> Result<Rect, NonEdgeError> {
        if width.get() < 3 {
            return Err(NonEdgeError::WidthTooSmall);
        }

        // The min/max non-edge corners; The corners of the rectangle of non-edge pieces.
        let min_corner_xy = xy::XY { x: xy::x(1), y: xy::y(1) };
        let height = xy::Inner::try_from(tiles_len)? / width.get();
        if height < 3 {
            return Err(NonEdgeError::TilesTooShort);
        }

        // -2 because -1 to get to the last index, then another to go to the second last index.
        let max_corner_xy = xy::XY { x: xy::x(width.get() - 2), y: xy::y(height - 2) };

        Ok((min_corner_xy, max_corner_xy))
    }

    // TODO use this where we can.
    #[derive(Debug)]
    pub struct TilesSpec {
        pub width: TilesWidth,
        pub len: Index,
    }

    // Written for an assert
    #[allow(unused)]
    pub fn is_non_edge_index(TilesSpec { width, len }: TilesSpec, index_to_check: Index) -> bool {
        non_edge_rect(width, len)
            .map(|(min_corner_xy, max_corner_xy)| {
                let xy = i_to_xy(width, index_to_check);

                xy.x >= min_corner_xy.x
                && xy.y >= min_corner_xy.y
                && xy.x <= max_corner_xy.x
                && xy.y <= max_corner_xy.y
            })
            .unwrap_or(false)
    }

    pub fn non_edge_index(width: TilesWidth, len: Index, rng: &mut Xs) -> Result<Index, NonEdgeError> {
        let (min_corner_xy, max_corner_xy) = non_edge_rect(width, len)?;

        let selected_xy = xy::XY {
            x: xy::x(xs::range(rng, min_corner_xy.x.u32()..max_corner_xy.x.u32() + 1) as xy::Inner),
            y: xy::y(xs::range(rng, min_corner_xy.y.u32()..max_corner_xy.y.u32() + 1) as xy::Inner)
        };

        let min_corner_index = xy_to_i(width, min_corner_xy)?;
        let max_corner_index = xy_to_i(width, max_corner_xy)?;

        if max_corner_index < min_corner_index {
            return Err(NonEdgeError::TilesTooShort);
        }

        Ok(xy_to_i(width, selected_xy)?)
    }
}

#[derive(Clone, Copy)]
struct ProtoTilesIndex(Index);

#[derive(Clone, Copy)]
struct ProtoTilesWidth(TilesWidth);

impl ProtoTilesWidth {
    fn new(inner: TilesWidthInner) -> Option<Self> {
        TilesWidth::new(inner).map(Self)
    }

    fn get(&self) -> TilesWidthInner {
        self.0.get()
    }
}

impl From<ProtoTilesWidth> for TilesWidth {
    fn from(ProtoTilesWidth(width): ProtoTilesWidth) -> Self {
        width
    }
}

fn generate_maze(
    rng: &mut Xs,
    proto_tiles: &mut [ProtoTileFlags],
    proto_width: ProtoTilesWidth
) -> (ProtoTilesIndex, Dir) {
    let ProtoTilesWidth(width) = proto_width;
    let width_usize = usize::from(width.get());

    //
    // Place the exit first
    //

    // Multiple things in the generation function rely on the starting exit_index being an non-edge tile!
    let exit_index_result = random::non_edge_index(width, proto_tiles.len(), rng);
    debug_assert!(exit_index_result.is_ok(), "got {exit_index_result:?}");
    // Default to the first non-edge tile
    let exit_index = exit_index_result.unwrap_or(width_usize + 2);

    let exit_facing = generate_with_exit_at_index(rng, proto_tiles, proto_width, exit_index);

    (ProtoTilesIndex(exit_index), exit_facing)
}

fn generate_with_exit_at_index(
    rng: &mut Xs,
    proto_tiles: &mut [ProtoTileFlags],
    proto_width: ProtoTilesWidth,
    exit_index: Index,
) -> Dir {
    assert!(
        random::is_non_edge_index(random::TilesSpec { width: proto_width.into(), len: proto_tiles.len() }, exit_index),
        "{:?} {:?}",
        random::TilesSpec { width: proto_width.into(), len: proto_tiles.len() },
        exit_index,
    );

    let exit_xy = i_to_xy(proto_width, exit_index);

    let height = calc_height(proto_width.into(), proto_tiles);

    let exit_facing = 'exit_facing: {
        let mut available_dirs = [
            if exit_xy.y >= xy::y(2) { Some(Dir::Up) } else { None },
            if exit_xy.y < xy::y(height.saturating_sub(2).into()) { Some(Dir::Down) } else { None },
            if exit_xy.x >= xy::x(2) { Some(Dir::Left) } else { None },
            if exit_xy.x < xy::x(proto_width.get().saturating_sub(2).into()) { Some(Dir::Right) } else { None },
        ];

        xs::shuffle(rng, &mut available_dirs);

        for dir_opt in available_dirs {
            if let Some(dir) = dir_opt {
                break 'exit_facing dir;
            }
        }

        unreachable!()
    };

    let (exit_hallway_index, fix_flags) = set_flags_for_exit(
        proto_tiles,
        proto_width,
        exit_index,
        exit_facing
    );

    //
    // Generate the maze in the area we didn't block out
    //

    // TODO Does starting at a random spot affect generation in a useful way?
    maze_via_backtracking(proto_tiles, rng, proto_width, <_>::default());

    //
    // Hook up the maze to the blocked out exit
    //

    proto_tiles[exit_hallway_index] |= fix_flags;
    print_proto_tiles(&proto_tiles, proto_width);

    exit_facing
}

#[cfg(test)]
mod generate_with_exit_at_index_generates_reachable_rooms_on {
    use super::*;
    use std::collections::HashSet;

    // Short for assert. We can be this terse because the scope here is limited.
    macro_rules! a {
        ($proto_tiles: expr, $width: expr, $exit_index: expr $(,)?) => ({
            let proto_tiles = $proto_tiles;
            let width = $width;

            fn is_open(flags: ProtoTileFlags) -> bool {
                // 0b1111 are the dir flags.
                // TODO? Add constant for that?
                flags & 0b1111 != 0
            }

            let mut open_tiles_count = 0;
            for &tile in &proto_tiles {
                if is_open(tile) {
                    open_tiles_count += 1;
                }
            }

            fn get_reachable_from(
                proto_tiles: &[ProtoTileFlags],
                width: ProtoTilesWidth,
                start_index: Index,
            ) -> HashSet<Index> {
                use std::collections::HashSet;
                let mut seen = HashSet::with_capacity(proto_tiles.len() / 2 /* was not thought about too hard */);

                let mut to_see = vec![i_to_xy(width, start_index)];

                while let Some(xy) = to_see.pop() {
                    if let Ok(i) = xy_to_i(width, xy)
                    && let Some(&proto_tile) = proto_tiles.get(i) {
                        if !is_open(proto_tile) { continue }

                        seen.insert(i);

                        for dir in Dir::ALL {
                            if let Some(new_xy) = xy.checked_push(dir)
                            && let Ok(new_i) = xy_to_i(width, new_xy)
                            && !seen.contains(&new_i) {
                                to_see.push(new_xy);
                            }
                        }
                    }
                }

                seen
            }

            let seen = get_reachable_from(
                &proto_tiles,
                width,
                $exit_index
            );

            let reachable_from_exit_count = seen.len();

            assert_eq!(reachable_from_exit_count, open_tiles_count);
        })
    }

    #[test]
    fn these_random_examples_in_the_top_of_a_small_vertical_maze() {
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        let proto_width = ProtoTilesWidth(TilesWidth::new(3).unwrap());
        let exit_index = 4; // The center of the top 3 x 3
        // A 3 x 4 room
        let mut proto_tiles;

        for _ in 0..16 {
            proto_tiles = vec1![0; 12usize];

            generate_with_exit_at_index(&mut rng, &mut proto_tiles, proto_width, exit_index);

            a!(
                proto_tiles,
                proto_width,
                exit_index
            );
        }
    }

    #[test]
    fn these_random_examples_in_the_bottom_of_a_small_vertical_maze() {
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        let proto_width = ProtoTilesWidth(TilesWidth::new(3).unwrap());
        let exit_index = 7; // The center of the bottom 3 x 3
        // A 3 x 4 room
        let mut proto_tiles;

        for _ in 0..16 {
            proto_tiles = vec1![0; 12usize];

            generate_with_exit_at_index(&mut rng, &mut proto_tiles, proto_width, exit_index);

            a!(
                proto_tiles,
                proto_width,
                exit_index
            );
        }
    }

    #[test]
    fn these_random_examples_in_the_left_of_a_small_horizontal_maze() {
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        let proto_width = ProtoTilesWidth(TilesWidth::new(4).unwrap());
        let exit_index = 5; // The center of the left 3 x 3
        // A 4 x 3 room
        let mut proto_tiles;

        for _ in 0..16 {
            proto_tiles = vec1![0; 12usize];

            generate_with_exit_at_index(&mut rng, &mut proto_tiles, proto_width, exit_index);

            a!(
                proto_tiles,
                proto_width,
                exit_index
            );
        }
    }

    #[test]
    fn these_random_examples_in_the_right_of_a_small_horizontal_maze() {
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        let proto_width = ProtoTilesWidth(TilesWidth::new(4).unwrap());
        let exit_index = 6; // The center of the right 3 x 3
        // A 4 x 3 room
        let mut proto_tiles;

        for _ in 0..16 {
            proto_tiles = vec1![0; 12usize];

            generate_with_exit_at_index(&mut rng, &mut proto_tiles, proto_width, exit_index);

            a!(
                proto_tiles,
                proto_width,
                exit_index
            );
        }
    }
}

/// Relies on the exit_index being an non-edge tile!
fn set_flags_for_exit(
    proto_tiles: &mut [ProtoTileFlags],
    proto_width: ProtoTilesWidth,
    exit_index: Index,
    exit_facing: Dir
) -> (Index, DirFlag) {
    let ProtoTilesWidth(width) = proto_width;
    let width_usize = usize::from(width.get());

    let u = Dir::Up.flag();
    let d = Dir::Down.flag();
    let l = Dir::Left.flag();
    let r = Dir::Right.flag();

    proto_tiles[exit_index - width_usize - 1] = SKIP;
    proto_tiles[exit_index - width_usize] = SKIP;
    proto_tiles[exit_index - width_usize + 1] = SKIP;
    proto_tiles[exit_index - 1] = SKIP;
    proto_tiles[exit_index] = SKIP;
    proto_tiles[exit_index + 1] = SKIP;
    proto_tiles[exit_index + width_usize - 1] = SKIP;
    proto_tiles[exit_index + width_usize] = SKIP;
    proto_tiles[exit_index + width_usize + 1] = SKIP;

    let flag = exit_facing.flag();
    let opposite_flag = exit_facing.opposite().flag();

    let (exit_hallway_index, fix_flags) = match exit_facing {
        Dir::Up
        | Dir::Down => {
            proto_tiles[exit_index - 1] |= r | flag;
            proto_tiles[exit_index] |= r | l | flag;
            proto_tiles[exit_index + 1] |= l | flag;

            let exit_hallway_index = if exit_facing == Dir::Up {
                exit_index - width_usize
            } else {
                exit_index + width_usize
            };

            proto_tiles[exit_hallway_index - 1] |= r | opposite_flag;
            // Clear flags so the maze reaches here
            proto_tiles[exit_hallway_index] = 0;
            proto_tiles[exit_hallway_index + 1] |= l | opposite_flag;

            (exit_hallway_index, r | l | opposite_flag)
        },
        Dir::Left
        | Dir::Right => {
            proto_tiles[exit_index - width_usize] |= d | flag;
            proto_tiles[exit_index] |= u | d | flag;
            proto_tiles[exit_index + width_usize] |= u | flag;

            let exit_hallway_index = if exit_facing == Dir::Left {
                exit_index - 1
            } else {
                exit_index + 1
            };

            proto_tiles[exit_hallway_index - width_usize] |= d | opposite_flag;
            // Clear flags so the maze reaches here
            proto_tiles[exit_hallway_index] = 0;
            proto_tiles[exit_hallway_index + width_usize] |= u | opposite_flag;

            (exit_hallway_index, u | d | opposite_flag)
        },
    };

    (exit_hallway_index, fix_flags)
}

#[cfg(test)]
mod set_flags_for_exit_produces_the_exact_result_on {
    use super::*;

    const U: DirFlag = Dir::Up.flag();
    const D: DirFlag = Dir::Down.flag();
    const L: DirFlag = Dir::Left.flag();
    const R: DirFlag = Dir::Right.flag();
    const S: DirFlag = SKIP;

    // Short for assert. We can be this terse because the scope here is limited.
    macro_rules! a {
        ($actual: expr, $expected: expr, $width: expr $(,)?) => {
            let actual = $actual;
            let expected = $expected;

            if actual != expected {
                let width = $width;
                let width_usize = usize::from(width.get());
                println!("actual:");
                print_proto_tiles(&actual, width);
                for i in 0..actual.len() {
                    print!(" {:#04X}", actual[i]);
                    if i % width_usize == width_usize - 1 { println!(); }
                }
                println!();

                println!("expected:");
                print_proto_tiles(&expected, width);
                for i in 0..expected.len() {
                    print!(" {:#04X}", expected[i]);
                    if i % width_usize == width_usize - 1 { println!(); }
                }
                println!();
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn the_minimal_up_case() {
        let proto_width = ProtoTilesWidth(TilesWidth::new(3).unwrap());
        let mut proto_tiles = vec1![0; 9usize];
        let exit_index = 4; // The center of the 3 x 3

        set_flags_for_exit(&mut proto_tiles, proto_width, exit_index, Dir::Up);

        a!(
            proto_tiles,
            // One cell intentionally leaves out the flags
            // to give a place to hook the maze onto.
            vec1![
                S | R | D,             0, S | L | D,
                S | R | U, S | L | R | U, S | L | U,
                        S,             S,         S,
            ],
            proto_width,
        );
    }

    #[test]
    fn the_minimal_down_case() {
        let proto_width = ProtoTilesWidth(TilesWidth::new(3).unwrap());
        let mut proto_tiles = vec1![0; 9usize];
        let exit_index = 4; // The center of the 3 x 3

        set_flags_for_exit(&mut proto_tiles, proto_width, exit_index, Dir::Down);

        a!(
            proto_tiles,
            // One cell intentionally leaves out the S
            // to give a place to hook the maze onto.
            vec1![
                        S,             S,         S,
                S | R | D, S | L | R | D, S | L | D,
                S | R | U,             0, S | L | U,
            ],
            proto_width,
        );
    }

    #[test]
    fn the_minimal_left_case() {
        let proto_width = ProtoTilesWidth(TilesWidth::new(3).unwrap());
        let mut proto_tiles = vec1![0; 9usize];
        let exit_index = 4; // The center of the 3 x 3

        set_flags_for_exit(&mut proto_tiles, proto_width, exit_index, Dir::Left);

        a!(
            proto_tiles,
            // One cell intentionally leaves out the S
            // to give a place to hook the maze onto.
            vec1![
                    S | R | D,     S | L | D, S,
                            0, S | L | U | D, S,
                    S | R | U,     S | L | U, S,
            ],
            proto_width,
        );
    }

    #[test]
    fn the_minimal_right_case() {
        let proto_width = ProtoTilesWidth(TilesWidth::new(3).unwrap());
        let mut proto_tiles = vec1![0; 9usize];
        let exit_index = 4; // The center of the 3 x 3

        set_flags_for_exit(&mut proto_tiles, proto_width, exit_index, Dir::Right);

        a!(
            proto_tiles,
            // One cell intentionally leaves out the S
            // to give a place to hook the maze onto.
            vec1![
                        S,     S | R | D, S | L | D,
                        S, S | R | U | D,         0,
                        S,     S | R | U, S | L | U,
            ],
            proto_width,
        );
    }
}

#[cfg(test)]
mod maze_via_backtracking_allows_blocking_out_areas_on {
    use super::*;
    use maze_via_backtracking_connects_all_cells_on::{are_all_cells_connected, are_all_cells_connected_options};

    #[test]
    fn this_small_example() {
        let width = ProtoTilesWidth::new(10).unwrap();
        let mut tiles = vec1![0; 100usize];

        for i in 0..tiles.len() {
            if i % usize::from(width.get()) > 5 {
                tiles[i] |= SKIP;
            }
        }

        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        assert!(!are_all_cells_connected(&mut tiles, width));
        assert!(!are_all_cells_connected_options(&mut tiles, width, SKIP));

        maze_via_backtracking(&mut tiles, &mut rng, width, <_>::default());

        assert!(!are_all_cells_connected(&mut tiles, width));
        assert!(are_all_cells_connected_options(&mut tiles, width, SKIP));

        for i in 0..tiles.len() {
            if i % usize::from(width.get()) > 5 {
                // The dir flags should all be 0, still
                assert_eq!(tiles[i], SKIP);
            }
        }
    }
}

fn proto_i_to_tile_xy(proto_width: ProtoTilesWidth, proto_i: ProtoTilesIndex) -> XY {
    let proto_xy = i_to_xy(proto_width.0, proto_i.0);

     XY { x: proto_xy.x.double().inc(), y: proto_xy.y.double().inc() }
}

/// Convert the tiles to 1-thick walls
fn to_one_thick(
    proto_tiles: &[ProtoTileFlags],
    proto_width: ProtoTilesWidth,
    tiles_length: TilesLength,
    width: TilesWidth
) -> Vec1<Tile> {
    use TileIndex::*;

    const W: TileIndex = Wall(0);
    const F: TileIndex = Floor;

    let mut tiles = vec1![Tile::default(); tiles_length];

    for proto_i in 0..proto_tiles.len() {
        let proto_tile_flags = proto_tiles[proto_i];

        if proto_tile_flags != 0 {
            // The cell is open on at least one side.
            let tile_xy = proto_i_to_tile_xy(proto_width, ProtoTilesIndex(proto_i));

            if let Ok(tile_i) = xy_to_i(width, tile_xy) {
                if let Some(tile) = tiles.get_mut(tile_i) {
                    tile.sprite_index = F;

                    tile.flags = to_tile_flags(proto_tile_flags);
                }
            }

            if proto_tile_flags & Dir::Right.flag() != 0 {
                if let Ok(tile_right_i) = xy_to_i(width, tile_xy + W::ONE) {
                    if let Some(el) = tiles.get_mut(tile_right_i).map(|t| &mut t.sprite_index) { *el = F; }
                }
            }

            if proto_tile_flags & Dir::Down.flag() != 0 {
                if let Ok(tile_down_i) = xy_to_i(width, tile_xy + H::ONE) {
                    if let Some(el) = tiles.get_mut(tile_down_i).map(|t| &mut t.sprite_index) { *el = F; }
                }
            }
        }
    }

    tiles
}

#[allow(unused)]
fn print_tiles(
    tiles: &[Tile],
    width: TilesWidth,
) {
   print_tiles_options(tiles, width, <_>::default())
}

fn calc_height<A>(
    width: TilesWidth,
    tiles: &[A],
) -> xy::Inner {
    calc_height_len(width, tiles.len())
}

fn calc_height_len(
    width: TilesWidth,
    tiles_len: usize,
) -> xy::Inner {
    xy::Inner::try_from(tiles_len).map(|len| len / width.get()).unwrap_or(xy::Inner::MAX)
}

#[allow(unused)]
fn print_tiles_options(
    tiles: &[Tile],
    width: TilesWidth,
    tags: HashMap<usize, char>,
) {
    let mut output = String::with_capacity(tiles.len());

    let height = calc_height(width, tiles);

    let space_count = 3;

    for y in 0..height {
        for x in 0..width.get() {
            let xy = XY { x: xy::x(x), y: xy::y(y) };

            let Ok(i) = xy_to_i(width, xy) else { continue };

            let tile = tiles[i];

            if let TileIndex::Wall(index) = tile.sprite_index {
                // default (space_count = 1)
                //'#'

                // decimal digits (space_count = 3)
                let hundreds = index as u32/100;
                let tens = (index as u32 - hundreds * 100)/10;
                let ones = (index as u32 - hundreds * 100 - tens * 10);
                output.push(char::from_digit(hundreds, 10).unwrap_or('?'));
                output.push(char::from_digit(tens, 10).unwrap_or('?'));
                output.push(char::from_digit(ones, 10).unwrap_or('?'));

                // Braille (space_count = 1)
                //output.push(char::from_u32(0x2800 + index as u32).unwrap_or('?'));
            } else {
                let ch = tags.get(&i).cloned().unwrap_or(' ');

                for _ in 0..space_count {
                    output.push(ch);
                }
            }


        }

        output.push('\n');
    }

    eprintln!("{output}");
}

#[cfg(test)]
mod to_one_thick_connects_all_cells_on {
    use super::*;
    use maze_via_backtracking_connects_all_cells_on::are_all_cells_connected as are_all_proto_cells_connected;

    fn are_all_one_floor_tiles_connected(
        tiles: &[Tile],
        width: TilesWidth
    ) -> bool {
        print_tiles(tiles, width);
        use TileIndex::Floor;

        let mut expected = 0;

        let mut start_floor_i = None;

        for i in 0..tiles.len() {
            if tiles[i].sprite_index == Floor {
                expected += 1;

                if start_floor_i.is_none() {
                    start_floor_i = Some(i);
                }
            }
        }

        if expected == 0 {
            return true
        }

        let start_floor_i = start_floor_i.unwrap();

        use std::collections::HashSet;
        let mut seen = HashSet::with_capacity(tiles.len() / 2 /* was not thought about too hard */);

        let mut to_see = vec![i_to_xy(width, start_floor_i)];

        while let Some(xy) = to_see.pop() {
            if let Ok(i) = xy_to_i(width, xy) {
                let tile = tiles[i];

                if tile.sprite_index != Floor { continue }

                seen.insert(i);

                for dir in Dir::ALL {
                    if let Some(new_xy) = xy.checked_push(dir)
                    && let Ok(new_i) = xy_to_i(width, new_xy)
                    && !seen.contains(&new_i) {
                        to_see.push(new_xy);
                    }
                }
            }
        }

        seen.len() == expected
    }

    #[test]
    fn this_generated_example() {
        let sizes = Sizes::new(8, 8);

        let mut proto_tiles = vec1![0; sizes.proto_length];
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        assert!(!are_all_proto_cells_connected(&mut proto_tiles, sizes.proto_width));

        maze_via_backtracking(&mut proto_tiles, &mut rng, sizes.proto_width, <_>::default());

        assert!(are_all_proto_cells_connected(&mut proto_tiles, sizes.proto_width));

        let tiles = to_one_thick(
            &proto_tiles,
            sizes.proto_width,
            sizes.tiles_length,
            sizes.tiles_width,
        );

        assert!(are_all_one_floor_tiles_connected(&tiles, sizes.tiles_width));
    }

    #[test]
    fn this_larger_non_square_example() {
        let sizes = Sizes::new(30, 20);

        let mut proto_tiles = vec1![0; sizes.proto_length];
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        assert!(!are_all_proto_cells_connected(&mut proto_tiles, sizes.proto_width));

        maze_via_backtracking(&mut proto_tiles, &mut rng, sizes.proto_width, <_>::default());

        assert!(are_all_proto_cells_connected(&mut proto_tiles, sizes.proto_width));

        let tiles = to_one_thick(
            &proto_tiles,
            sizes.proto_width,
            sizes.tiles_length,
            sizes.tiles_width,
        );

        assert!(are_all_one_floor_tiles_connected(&tiles, sizes.tiles_width));
    }
}

/// Set the indexes from the surrounding tiles.
fn set_indexes(tiles: &mut [Tile], width: TilesWidth) {
    for index in 0..tiles.len() {
        if !tiles[index].sprite_index.is_floor() {
            let width = usize::from(width.get());

            // Assume everything not set is a wall, for maximum merging.
            let mut output_mask = 0;

            macro_rules! set {
                (-, $subtrahend: expr, $mask: ident) => {
                    if let Some(tile) = index.checked_sub($subtrahend)
                        .and_then(|i| tiles.get(i).map(|t| t.sprite_index)) {

                        // TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
                        // we can use highest_one instead.
                        let shift = NeighborFlag::BITS - 1 - $mask.leading_zeros();

                        output_mask |= (tile.is_floor() as NeighborMask) << shift;
                    }
                };
                (+, $addend: expr, $mask: ident) => {
                    if let Some(tile) = index.checked_add($addend)
                        .and_then(|i| tiles.get(i).map(|t| t.sprite_index)) {

                        // TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
                        // we can use highest_one instead.
                        let shift = NeighborFlag::BITS - 1 - $mask.leading_zeros();

                        output_mask |= (tile.is_floor() as NeighborMask) << shift;
                    }
                };
            }

            set!(-, width + 1, UPPER_LEFT);
            set!(-, width, UPPER_MIDDLE);
            set!(-, width - 1, UPPER_RIGHT);
            set!(-, 1, LEFT_MIDDLE);

            set!(+, 1, RIGHT_MIDDLE);
            set!(+, width - 1, LOWER_LEFT);
            set!(+, width, LOWER_MIDDLE);
            set!(+, width + 1, LOWER_RIGHT);

            if let TileIndex::Wall(mask_ref) = &mut tiles[index].sprite_index {
                *mask_ref = output_mask
            } else {
                unreachable!("Tile changed while we were looking at it?!");
            }
        }
    }
}

#[cfg(test)]
mod set_indexes_works_on {
    use super::*;

    /// Only returns walls with unset (zero) indexes.
    fn three_by_three_walls_from_index(index: u8) -> Vec1<Tile> {
        let w: Tile = Tile {
            sprite_index: TileIndex::Wall(0),
            ..Tile::default()
        };
        let f: Tile = Tile {
            sprite_index: TileIndex::Floor,
            ..Tile::default()
        };

        let mut output = vec1![
            w, w, w,
            w, w, w,
            w, w, w,
        ];

        for i in 0..8 {
            let mask = 1 << i;

            if index & mask != 0 {
                output[if i < 4 { i } else { i + 1 } as usize] = f;
            }
        }

        output
    }

    #[test]
    fn the_one_floor_cases() {
        let width = TilesWidth::new(3).unwrap();
        for i in 0..8 {
            let index = 0b1u8.rotate_left(i);

            let mut tiles = three_by_three_walls_from_index(index);

            set_indexes(&mut tiles, width);

            // The middle tile
            assert_eq!(tiles[4].sprite_index, TileIndex::Wall(index), "i = {i}, tiles = {tiles:?}");
        }
    }

    #[test]
    fn the_adjacent_two_floor_cases() {
        let width = TilesWidth::new(3).unwrap();
        for i in 0..8 {
            let index = 0b11u8.rotate_left(i);

            let mut tiles = three_by_three_walls_from_index(index);

            set_indexes(&mut tiles, width);

            // The middle tile
            assert_eq!(tiles[4].sprite_index, TileIndex::Wall(index), "i = {i}, tiles = {tiles:?}");
        }
    }

    #[test]
    fn the_one_apart_two_floor_cases() {
        let width = TilesWidth::new(3).unwrap();
        for i in 0..8 {
            let index = 0b101u8.rotate_left(i);

            let mut tiles = three_by_three_walls_from_index(index);

            set_indexes(&mut tiles, width);

            // The middle tile
            assert_eq!(tiles[4].sprite_index, TileIndex::Wall(index), "i = {i}, tiles = {tiles:?}");
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HitEffect {
    NoOp,
    Toggle(ToggleGroupId),
    RemoveRoach,
}

fn get_hit_effect(key: Key, mobs: &Mobs) -> HitEffect {
    use HitEffect::*;
    let mut effect = HitEffect::NoOp;

    match mobs.get(key) {
        Some(hit_mob) if hit_mob.tile_sprite == SWITCH_BASE => {
            // Toggle relevant mobs
            if hit_mob.toggle_group_id != NULL_GROUP {
                effect = Toggle(hit_mob.toggle_group_id);
            }
        },
        Some(hit_mob) if hit_mob.tile_sprite == ROACH_BASE => {
            effect = RemoveRoach;
        },
        Some(_) | None => {}
    }

    effect
}

#[cfg(test)]
mod get_hit_effect_works_on {
    use super::*;

    #[test]
    fn these_roach_examples() {
        use HitEffect::*;

        let key_1 = Key { xy: <_>::default() };
        let key_2 = Key { xy: XY { x: xy::x(1), y: xy::y(1), } };

        let mut mobs: Mobs = <_>::default();

        mobs.insert(key_1, Entity {
            tile_sprite: ROACH_BASE,
            position: key_1.xy.into(),
            ..<_>::default()
        });

        let mut position_in_motion = Position::from(XY { x: xy::x(0), y: xy::y(1), });

        position_in_motion.set_xy(key_2.xy);
        position_in_motion.decay();

        // Oh wait, I bet we messed up mutating mobs, and the key and positions don't match.
        // Get this building, then prevent that from happening with a module.
        mobs.insert(key_2, Entity {
            tile_sprite: ROACH_BASE,
            position: position_in_motion,
            ..<_>::default()
        });

        assert_eq!(
            RemoveRoach,
            get_hit_effect(key_1, &mobs),
        );

        assert_eq!(
            RemoveRoach,
            get_hit_effect(key_2, &mobs),
        );
    }
}

type Mobs = Entities;

#[derive(Clone, Debug)]
pub struct State {
    pub rng: Xs,
    pub player: Entity,
    pub player_position: Position,
    pub mobs: Mobs,
    pub tiles: Tiles,
    pub animations: Animations,
}

impl State {
    pub fn new(rng: &mut Xs, wall_spec: &sprite::Spec<sprite::Wall>) -> Self {
        let seed = xs::new_seed(rng);

        let mut rng = xs::from_seed(seed);

        let mut player = Entity::default();
        player.tile_sprite = PLAYER_BASE;
        player.flags |= RENDER_FACING;
        let player_position;

        let mut mobs = Mobs::default();

        macro_rules! insert_entity {
            ($xy: expr, $entity: expr) => ({
                let entity = $entity;
                mobs.insert(
                    Key {
                        xy: $xy,
                    },
                    entity
                );
            })
        }

        use TileIndex::*;

        let (mut max_tile_w, mut max_tile_h) = wall_spec.max_tile_counts();

        // temp to debug
        //max_tile_w = max_tile_w.saturating_sub(1);

        if max_tile_w == 0 {
            max_tile_w = 1;
        }
        if max_tile_h == 0 {
            max_tile_h = 1;
        }

        assert_eq!(player.flags & GONE, 0, "The player should never be gone!");
        assert_eq!(player.flags & RENDER_FACING, RENDER_FACING, "The player should always be rendered taking facing into account!");

        let width = TilesWidth::new(max_tile_w).expect("Don't set a 0 width!");
        let mut tiles = {
            // Proposal for tiles generation:
            //     Overall idea: Start with a completable level and add compliciations to it, that keep it solvable.
            // Rough Algorithm:
            // Start with a starting spot and an exit, right next to each other.
            // Pick a random list of the following operations to do, in sequence:
            //    * Add a longer hallway between them, maybe with a bend in the road.
            //    * Add a toggle door, and a switch on the starting side.
            //        * Sub steps:
            //            * Pick a point in the hallway to have a door.
            //            * Place the door there, keeping track of the toggle group id.
            //            * Pick a point between the door and the starting spot
            //            * Place the switch there, and make it toggle the door
            //
            // If we end up wanting harder puzzles, come up with examples of them, say
            // some small gadget where you have to unswitch a switch to get to another
            // switch, to unlock the next door and make inserting that a step to choose
            // from.

            // Might end up doing a Dijkstra's algorithm thing that counts the number of steps,
            // so we can place doors that block access to all places N steps along all paths.

            // Suggested steps for the implementation itself (obsoleted, read on):
            // * Start generating random start and end locations, right next to each other. ✔
            // * Start extending the hallway by a random amount ✔
            //    * use the loop implied by the rough algorithm above, with a space for a `match`
            // * Implement bounds checking and corner turning for the hallway
            // * Place doors, or some other recognizable thing at random spots along the hallway
            // * Place the switches for those doors.
            // * Evaluate whether this feels like enough

            // Hmm. Just doing random movement collects in one spot on average. I think we need a different approach.
            // This describes some options for dungeon generation: https://journal.stuffwithstuff.com/2014/12/21/rooms-and-mazes/
            // The most relevant seeming idea there is to make a perfect maze, then fill stuff back in.
            // Let's try something like that.
            //
            // Most maze algorithms generate mazes with thin walls so we'll need to convert to one thick walls.
            // This page describes that conversion: https://gamedev.stackexchange.com/a/142525
            //
            // New suggested steps:
            // * Hand define an example map in a format we will be able to generate using an algorithm description
            // * Write and test a conversion from that format to 1 thick walls, setting the tiles to the output
            // * Write the generation code, and output the result to the tiles
            // * Place doors, or some other recognizable thing at random spots along the hallway
            // * Place the switches for those doors.
            // * Evaluate whether this feels like enough

            // This description seems like a good one: https://weblog.jamisbuck.org/2010/12/27/maze-generation-recursive-backtracking

            //
            // End of planning/proposals

            let sizes = Sizes::new(max_tile_w, max_tile_h);

            let width_usize = usize::from(width.get());

            let mut proto_tiles = vec1![0; sizes.proto_length];

            let (proto_exit_index, exit_facing) = generate_maze(&mut rng, &mut proto_tiles, sizes.proto_width);

            const W: TileIndex = Wall(0);
            const F: TileIndex = Floor;

            let mut tiles = to_one_thick(&proto_tiles, sizes.proto_width, sizes.tiles_length, sizes.tiles_width);

            let exit_index = xy_to_i(sizes.tiles_width, proto_i_to_tile_xy(sizes.proto_width, proto_exit_index))
                // Default to the first non-edge tile
                .unwrap_or(width_usize + 2);

            //
            // Pick sections for things to be placed in
            //

            let start_index = {
                let mut start_index = exit_index;
                start_index += tiles.len() / 2;

                while start_index >= tiles.len() {
                    start_index -= tiles.len();
                }

                while tiles[start_index].sprite_index != F
                || tiles[start_index].flags & TILE_REQUIRED != 0
                {
                    start_index += 1;

                    while start_index >= tiles.len() {
                        start_index -= tiles.len();
                    }
                }

                start_index
            };

            let start_xy = i_to_xy(width, start_index);
            let exit_xy = i_to_xy(width, exit_index);

            let mut paths = Vec::with_capacity(16 /* not thought about too hard */);

            type Path = Vec<Index>;

            fn find_all_paths(
                tiles: &[Tile],
                width: TilesWidth,
                current_xy: XY,
                exit_xy: XY,
                mut current_path: Path,
                paths: &mut Vec<Path>,
            ) {
                if let Ok(current_i) = xy_to_i(width, current_xy)
                && !current_path.contains(&current_i)
                && let Some(Floor) = tiles.get(current_i).map(|t| t.sprite_index) {
                    current_path.push(current_i);

                    if current_xy == exit_xy {
                        paths.push(current_path);
                    } else {
                        if let Some(xy) = current_xy.checked_push(Dir::Left) {
                            find_all_paths(&tiles, width, xy, exit_xy, current_path.clone(), paths);
                        }
                        if let Some(xy) = current_xy.checked_push(Dir::Right) {
                            find_all_paths(&tiles, width, xy, exit_xy, current_path.clone(), paths);
                        }
                        if let Some(xy) = current_xy.checked_push(Dir::Up) {
                            find_all_paths(&tiles, width, xy, exit_xy, current_path.clone(), paths);
                        }
                        if let Some(xy) = current_xy.checked_push(Dir::Down) {
                            find_all_paths(&tiles, width, xy, exit_xy, current_path/* take ownership */, paths);
                        }
                    }
                }
            }

            find_all_paths(&tiles, sizes.tiles_width, start_xy, exit_xy, vec![], &mut paths);

            // Currently there's always only one path. Might pick the longest path among multiple later.
            if paths.is_empty() {
                print_tiles_options(
                    &tiles,
                    width,
                    {
                        let mut tags = HashMap::default();
                        tags.insert(start_index, 's');
                        tags.insert(exit_index, 'e');
                        tags
                    }
                );
                eprintln!("{:?} to {:?}", start_xy, exit_xy);
                assert!(!paths.is_empty());
            }
            let path: Path = paths.swap_remove(0);

            // There's a few types of indexes flying around in this part of the code, and it feels like mistakes are
            // likely to happen. So we define some index types and wrap the relevant collections with structs to
            // ensure that certain plausible mistakes are compile errors.

            // To ensure compile errors, we just need to have all the relevant types be distinct from each other,
            // so we can leave one as the common index type.
            type TilesIndex = Index;


            #[derive(Default)]
            struct PathEdgeIndexes {
                _indexes: [TilesIndex; 3],
            }
            struct PathEdgeI(Index);

            const PEI_0: PathEdgeI = PathEdgeI(0);
            const PEI_1: PathEdgeI = PathEdgeI(1);
            const PEI_2: PathEdgeI = PathEdgeI(2);

            impl std::ops::Index<PathEdgeI> for PathEdgeIndexes {
                type Output = TilesIndex;

                fn index(&self, PathEdgeI(i): PathEdgeI) -> &Self::Output {
                    &self._indexes[i]
                }
            }
            impl std::ops::IndexMut<PathEdgeI> for PathEdgeIndexes {
                fn index_mut(&mut self, PathEdgeI(i): PathEdgeI) -> &mut Self::Output {
                    &mut self._indexes[i]
                }
            }

            #[derive(Clone)]
            struct PathWrapper {
                _path: Vec<TilesIndex>,
            }
            impl PathWrapper {
                fn len(&self) -> usize { self._path.len() }
                fn contains(&self, index: &TilesIndex) -> bool { self._path.contains(index) }
                fn push(&mut self, element: TilesIndex) { self._path.push(element); }

                fn iter(&self) -> impl Iterator<Item = &TilesIndex> {
                    self._path.iter()
                }
            }

            #[derive(PartialEq, Eq, PartialOrd, Ord)]
            struct PathI(Index);

            impl std::ops::Index<PathI> for PathWrapper {
                type Output = TilesIndex;

                fn index(&self, PathI(i): PathI) -> &Self::Output {
                    &self._path[i]
                }
            }

            impl From<PathI> for Index {
                fn from(PathI(i): PathI) -> Index { i }
            }

            let path = PathWrapper{ _path: path };

            // Replace all floor tiles that are not on the path
            // with walls.
            // TODO? Maybe leave some there for flavor?
            for i in 0..tiles.len() {
                let tile = &mut tiles[i];

                if (
                    tile.sprite_index == Floor
                    || tile.flags & TILE_REQUIRED == 0
                ) && !path.contains(&i) {
                    // The indexes are set later
                    tile.sprite_index = Wall(0);
                }
            }

            let mut floor_indexes = path.clone();

            //
            // Place Exit
            //

            // Relies on the exit_index being an non-edge tile!

            let exit_indexes = match exit_facing {
                Dir::Up
                | Dir::Down => [exit_index - 1, exit_index, exit_index + 1],
                Dir::Left
                | Dir::Right => [exit_index - width_usize, exit_index, exit_index + width_usize],
            };

            let exit_sprites_index = match exit_facing {
                Dir::Up => UP_STAIRS_TOP_LEFT_EDGE_INDEX,
                Dir::Down => DOWN_STAIRS_TOP_LEFT_EDGE_INDEX,
                Dir::Left => LEFT_STAIRS_TOP_LEFT_EDGE_INDEX,
                Dir::Right => RIGHT_STAIRS_TOP_LEFT_EDGE_INDEX,
            };

            let mut offset = 0;
            for index in exit_indexes {
                tiles[index].sprite_index = F;
                if !floor_indexes.contains(&index) {
                    floor_indexes.push(index);
                } else {
                    debug_assert_eq!(exit_index, index);
                    debug_assert!(path.contains(&index));
                }

                let exit_xy = i_to_xy(width, index);

                insert_entity!(
                    exit_xy,
                    Entity {
                        tile_sprite: TileSprite::Sword(
                            EXIT_SPRITES[exit_sprites_index + offset].sword_inner_or_0()
                        ),
                        ..<_>::default()
                    }
                );
                offset += 1;
            }

            let exit_front_indexes = match exit_facing {
                Dir::Up => [exit_index - width_usize - 1, exit_index - width_usize, exit_index - width_usize + 1],
                Dir::Down => [exit_index + width_usize - 1, exit_index + width_usize, exit_index + width_usize + 1],
                Dir::Left => [exit_index - width_usize - 1, exit_index - 1, exit_index + width_usize - 1],
                Dir::Right => [exit_index - width_usize + 1, exit_index + 1, exit_index + width_usize + 1],
            };

            for index in exit_front_indexes {
                tiles[index].sprite_index = F;
            }

            //
            // Perform random complication actions that preserve the solvabilty.
            //

            let mut free_group_id = FIRST_GROUP;

            let target_complication_count = 3;
            let mut current_complication_count = 0;
            let max_bad_complication_count = 16;
            let mut bad_complication_attempts_count = 0;

            macro_rules! abort_complication_attempt {
                () => {
                    bad_complication_attempts_count += 1;
                    if bad_complication_attempts_count >= max_bad_complication_count {
                        // Just move on and live without a complication
                        current_complication_count += 1;
                    }

                    continue
                }
            }

            enum Complication {
                // Can we extend the path in an intereting way? Perhaps from the middle?
                //ExtendPath,
                AddSwitchDoor,
                //MoveSwitch,
                //MoveDoor,
            }

            while current_complication_count < target_complication_count {
                // TODO define multiple and pick randomly
                let complication = Complication::AddSwitchDoor;

                match complication {
                    Complication::AddSwitchDoor => {
                        // * Pick a point in the hallway to have a door.

                        // We want an index in the middle of the path, not right
                        // at the ends where the exit and the start are, so it
                        // doesn't make the puzzle trival or impossible.

                        let mut door_indexes = PathEdgeIndexes::default();

                        const SLICED_OFF_START: usize = 3;
                        const SLICED_OFF_END: usize = 3;
                        assert!(path.len() > SLICED_OFF_START + SLICED_OFF_END);
                        door_indexes[PEI_1] = path[PathI(
                            xs::index(&mut rng, SLICED_OFF_START..(path.len() - SLICED_OFF_END))
                        )];

                        // Look for the adjacent walls
                        // Try x first
                        door_indexes[PEI_0] = door_indexes[PEI_1].saturating_sub(1);
                        door_indexes[PEI_2] = door_indexes[PEI_1].saturating_add(1);

                        match (
                            tiles.get(door_indexes[PEI_0]).map(|t| t.sprite_index),
                            tiles.get(door_indexes[PEI_2]).map(|t| t.sprite_index)
                        ) {
                            (Some(Wall(_)), Some(Wall(_))) => { /* keep these */ }
                            _ => {
                                // Try y now
                                door_indexes[PEI_0] = door_indexes[PEI_1].saturating_sub(width_usize);
                                door_indexes[PEI_2] = door_indexes[PEI_1].saturating_add(width_usize);

                                match (
                                    tiles.get(door_indexes[PEI_0]).map(|t| t.sprite_index),
                                    tiles.get(door_indexes[PEI_2]).map(|t| t.sprite_index)
                                ) {
                                    (Some(Wall(_)), Some(Wall(_))) => { /* keep these */ }
                                    _ => {
                                        // Probably got unlucky and picked a spot a hallway had already spread from
                                        abort_complication_attempt!();
                                    }
                                }
                            }
                        }
                        // * Pick a point between the door and the starting spot for the switch

                        let switch_range =
                            PathI(3)..PathI(
                                path.iter().position(|&i| i == door_indexes[PEI_1]).expect("Door index not found in path?!")
                            );
                        if switch_range.is_empty() {
                            abort_complication_attempt!();
                        }
                        // The index on the path relating to where the switch will be
                        let switch_on_path_i: TilesIndex = path[
                            PathI(xs::index(&mut rng, switch_range.start.into()..switch_range.end.into()))
                        ];

                        // Look for the adjacent walls
                        // Try x first
                        let switch_on_path_i_minus_1: TilesIndex = switch_on_path_i.saturating_sub(1);
                        let switch_on_path_i_plus_1: TilesIndex = switch_on_path_i.saturating_add(1);

                        let mut switch_i: TilesIndex = match (
                            tiles.get(switch_on_path_i_minus_1).map(|t| t.sprite_index),
                            tiles.get(switch_on_path_i_plus_1).map(|t| t.sprite_index)
                        ) {
                            (Some(Wall(_)), _) => switch_on_path_i_minus_1,
                            (None, Some(Wall(_))) => switch_on_path_i_plus_1,
                            _ => {
                                let switch_on_path_i_minus_w: TilesIndex = switch_on_path_i.saturating_sub(width_usize);
                                let switch_on_path_i_plus_w: TilesIndex = switch_on_path_i.saturating_add(width_usize);

                                match (
                                    tiles.get(switch_on_path_i_minus_w).map(|t| t.sprite_index),
                                    tiles.get(switch_on_path_i_plus_w).map(|t| t.sprite_index)
                                ) {
                                    (Some(Wall(_)), _) => switch_on_path_i_minus_w,
                                    (None, Some(Wall(_))) => switch_on_path_i_plus_w,
                                    _ => {
                                        // Probably got unlucky and picked a spot a hallway had already spread from
                                        abort_complication_attempt!();
                                    }
                                }
                            },
                        };

                        // TODO Attempt to drill a hallway into the wall to make the switch farther away.
                        // (And maybe recurse this switch placement onto the resulting path, if it seems long enough!)

                        tiles[switch_i].sprite_index = F;

                        struct Targeting {
                            source: TilesIndex,
                            target: TilesIndex,
                            width: TilesWidth,
                        }

                        let mut possible_new_switch_i = switch_i;

                        macro_rules! is_wall_or_source {
                            ($source: ident, $index_opt: expr) => {
                                // Note: We mus accept invalid indexes here without crashing,
                                // but while returning false, because it simplies the rest
                                // of the relevant code.
                                $index_opt
                                    .map(|i| i == $source || matches!(tiles.get(i).map(|t| t.sprite_index), Some(Wall(_))))
                                    .unwrap_or(false)
                            };
                        }

                        macro_rules! is_acceptable_to_drill_from {
                            ($targeting: expr) => ({
                                // Note: We mus accept invalid indexes here without crashing,
                                // but while returning false, because it simplies the rest
                                // of the relevant code.
                                let Targeting{ source, target, width } = $targeting;

                                if let Some(Wall(_)) = tiles.get(target).map(|t| t.sprite_index)
                                && is_wall_or_source!(source, target.checked_sub(width_usize))
                                && is_wall_or_source!(source, target.checked_add(width_usize))
                                && is_wall_or_source!(source, target.checked_sub(1))
                                && is_wall_or_source!(source, target.checked_add(1))
                                {
                                    let source_xy = i_to_xy(width, source);
                                    let target_xy = i_to_xy(width, target);

                                    // TODO? Should we just work more in XY and only calculate indexes when needed?
                                    (source_xy.x == target_xy.x) || (source_xy.y == target_xy.y)
                                } else {
                                    false
                                }
                            })
                        }

                        let mut last_dir = Dir::ALL[0];
                        for dir in Dir::ALL {
                            if let Some(i) = match dir {
                                Dir::Up => possible_new_switch_i.checked_sub(width_usize),
                                Dir::Down => possible_new_switch_i.checked_add(width_usize),
                                Dir::Left => possible_new_switch_i.checked_sub(1),
                                Dir::Right => possible_new_switch_i.checked_add(1),
                            } && is_acceptable_to_drill_from!(Targeting{ source: possible_new_switch_i, target: i, width }) {
                                possible_new_switch_i = i;
                                last_dir = dir;
                                break
                            }
                        }

                        while let Wall(_) = tiles[possible_new_switch_i].sprite_index {
                            tiles[possible_new_switch_i].sprite_index = F;
                            if xs::zero_to_one(&mut rng) < 0.125 {
                                // Even if we break here, `tiles[possible_new_switch_i] == F`
                                break
                            }

                            if xs::zero_to_one(&mut rng) < 0.125 {
                                let mut dirs = Dir::ALL;
                                xs::shuffle(&mut rng, &mut dirs);

                                last_dir = dirs[0];
                            }

                            if let Some(i) = match last_dir {
                                Dir::Up => possible_new_switch_i.checked_sub(width_usize),
                                Dir::Down => possible_new_switch_i.checked_add(width_usize),
                                Dir::Left => possible_new_switch_i.checked_sub(1),
                                Dir::Right => possible_new_switch_i.checked_add(1),
                            } && is_acceptable_to_drill_from!(Targeting{ source: possible_new_switch_i, target: i, width }) {
                                possible_new_switch_i = i;
                            }
                        }
                        assert_eq!(tiles[possible_new_switch_i].sprite_index, F);
                        switch_i = possible_new_switch_i;

                        // * Place the door
                        floor_indexes.push(door_indexes[PEI_0]);
                        floor_indexes.push(door_indexes[PEI_2]);

                        // End of section where indexing bugs are relevant.
                        let door_indexes = door_indexes._indexes;

                        for index in door_indexes {
                            // Assume everything not set is a floor, to avoid merging
                            // with the tile walls.
                            let mut output_mask = 0b1111_1111;

                            // Check if the id is a door index and use that to decide the mask
                            macro_rules! set {
                                (-, $subtrahend: expr, $mask: ident) => {
                                    if let Some(i) = index.checked_sub($subtrahend) {
                                        if door_indexes.contains(&i) {
                                            // TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
                                            // we can use highest_one instead.
                                            let shift = NeighborFlag::BITS - 1 - $mask.leading_zeros();

                                            output_mask &= !(1 << shift);
                                        }
                                    }
                                };
                                (+, $addend: expr, $mask: ident) => {
                                    if let Some(i) = index.checked_add($addend) {
                                        if door_indexes.contains(&i) {
                                            // TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
                                            // we can use highest_one instead.
                                            let shift = NeighborFlag::BITS - 1 - $mask.leading_zeros();

                                            output_mask &= !(1 << shift);
                                        }
                                    }
                                };
                            }

                            set!(-, width_usize + 1, UPPER_LEFT);
                            set!(-, width_usize, UPPER_MIDDLE);
                            set!(-, width_usize - 1, UPPER_RIGHT);
                            set!(-, 1, LEFT_MIDDLE);

                            set!(+, 1, RIGHT_MIDDLE);
                            set!(+, width_usize - 1, LOWER_RIGHT);
                            set!(+, width_usize, LOWER_MIDDLE);
                            set!(+, width_usize + 1, LOWER_LEFT);

                            let xy = i_to_xy(width, index);

                            insert_entity!(
                                xy,
                                Entity {
                                    tile_sprite: TileSprite::ToggleWall(output_mask),
                                    toggle_group_id: free_group_id,
                                    ..<_>::default()
                                }
                            );
                        }

                        // * Place the switch
                        let switch_xy = i_to_xy(width, switch_i);

                        insert_entity!(
                            switch_xy,
                            Entity {
                                tile_sprite: SWITCH_BASE,
                                toggle_group_id: free_group_id,
                                ..<_>::default()
                            }
                        );

                        free_group_id += 1;
                        current_complication_count += 1;
                        // Count attempts per current_complication_count.
                        bad_complication_attempts_count = 0;
                    }
                }
            }

            //
            // Place enemy mobs
            //

            for i in 0..tiles.len() {
                if tiles[i].is_floor()
                && xs::range(&mut rng, 0..8) == 0
                {
                    let enemy_xy = i_to_xy(width, i);

                    // Don't replace other mobs
                    if let Some(_) = mobs.get(
                        Key {
                            xy: enemy_xy,
                        }
                    ) {
                        continue
                    }

                    insert_entity!(
                        enemy_xy,
                        Entity {
                            tile_sprite: ROACH_BASE,
                            flags: RENDER_FACING,
                            ..<_>::default()
                        }
                    );
                }
            }

            player_position = start_xy.into();

            tiles
        };

        set_indexes(&mut tiles, width);

        print_tiles(&tiles, width);

        assert_eq!(player.flags & GONE, 0, "The player should never be gone!");
        assert_eq!(player.flags & RENDER_FACING, RENDER_FACING, "The player should always be rendered taking facing into account!");

        Self {
            rng,
            player,
            player_position,
            mobs,
            tiles: Tiles {
                width,
                tiles,
            },
            animations: <_>::default(),
        }
    }

    pub fn is_complete(&self) -> bool {
        let key = Key { xy: self.player_position.xy() };

        if let Some(mob) = self.mobs.get(key) {
            return mob.tile_sprite.is_stairs();
        }
        false
    }

    fn staff_xy_pair(&self) -> (XY, EdgeHitKind) {
        xy_in_dir(self.player_position.xy(), self.player.facing)
    }

    fn tick(&mut self) {
        let staff_xy_pair = self.staff_xy_pair();

        //
        // Advance timers
        //

        self.mobs.decay_positions();

        for i in (0..self.animations.len()).rev() {
            let animation = &mut self.animations[i];
            if animation.frames_left > 0 {
                animation.frames_left -= 1;
            } else {
                let mut animation = self.animations.swap_remove(i);

                // Handle any final actions
                match animation.kind {
                    AnimationKind::Reset => {
                        enum PostAction {
                            NoOp,
                            RedoAnimation,
                        }
                        let mut post_action = PostAction::NoOp;

                        if let Some(mob) = self.mobs.get_mut(animation.target_key) {
                            match mob.tile_sprite {
                                SWITCH_HIT => {
                                    let should_redo_animation = if let (staff_xy, EdgeHitKind::Neither) = staff_xy_pair {
                                        staff_xy == animation.target_key.xy
                                    } else {
                                        false
                                    };

                                    if should_redo_animation {
                                        post_action = PostAction::RedoAnimation;
                                    } else {
                                        mob.tile_sprite = SWITCH_BASE;
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            debug_assert!(false, "No mob found at {:?}", animation.target_key);
                        }

                        match post_action {
                            PostAction::NoOp => {}
                            PostAction::RedoAnimation => {
                                animation.frames_left = RESET_ANIMATION_LENGTH;
                                self.animations.push(animation);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        sword_spec: &sprite::Spec::<sprite::SWORD>,
        wall_spec: &sprite::Spec::<sprite::Wall>,
        floor_spec: &sprite::Spec::<sprite::Floor>,
        toggle_wall_spec: &sprite::Spec::<sprite::ToggleWall>,
        input: Input,
        _speaker: &mut Speaker,
    ) {
        //
        // Update
        //

        self.tick();

        let mut player_moved = false;
        if let Some(dir) = input.dir_pressed_this_frame() {
            // Walk
            let (new_xy, _) = xy_in_dir(self.player_position.xy(), dir.into());

            if can_walk_onto(&self.mobs, &self.tiles, Key { xy: new_xy }) {
                self.player_position.set_xy(new_xy);
                player_moved = true;
            }
        } else if input.pressed_this_frame(Button::A) {
            self.player.facing = self.player.facing.counter_clockwise();
            player_moved = true;
        } else if input.pressed_this_frame(Button::B) {
            self.player.facing = self.player.facing.clockwise();
            player_moved = true;
        }

        let staff_xy_pair = self.staff_xy_pair();

        if let &(staff_xy, EdgeHitKind::Neither) = &staff_xy_pair {
            let key = Key { xy: staff_xy };

            use HitEffect::*;

            match get_hit_effect(key, &self.mobs) {
                NoOp => {},
                Toggle(group_id) => {
                    if let Some(hit_mob) = self.mobs.get_mut(key) {
                        hit_mob.tile_sprite = SWITCH_HIT;

                        // Start animation timer
                        self.animations.push(Animation::reset(key));

                        // TODO Good place for SFX
                    }

                    // TODO? Is it worth building an acceleration structure for this lookup?
                    for mob in self.mobs.entities_mut() {
                        if mob.toggle_group_id == group_id {
                            match mob.tile_sprite {
                                TileSprite::ToggleWall(..) => {
                                    mob.flags ^= GONE;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                RemoveRoach => {
                    // TODO Good place for SFX
                    // TODO? Put a splat on the ground in this spot instead?
                    self.mobs.remove(key);
                }
            }
        }

        if player_moved {
            // Each mob takes their turn to move
            let mut mob_mutations = Vec::with_capacity(16);

            for (position, mob) in self.mobs.all() {
                if mob.flags & GONE == GONE {
                    continue
                }

                match mob.tile_sprite {
                    ROACH_BASE => {
                        assert_eq!(mob.flags & RENDER_FACING, RENDER_FACING);

                        let mob_xy = position.xy();

                        fn manhattan_distance(a: XY, b: XY) -> xy::Diff {
                            (xy::Diff::from(a.x) - xy::Diff::from(b.x)).abs() + (xy::Diff::from(a.y) - xy::Diff::from(b.y)).abs()
                        }

                        let open_adjacent_spots = Dir::ALL
                            .into_iter()
                            .filter_map(|dir| {
                                if let (xy, EdgeHitKind::Neither) = xy_in_dir(mob_xy, dir.into()) {
                                    Some(xy)
                                } else {
                                    None
                                }
                            }).filter_map(|xy| {
                                let target_key = Key { xy };
                                if self.mobs.get(target_key).is_none()
                                && let Ok(tile_i) = xy_to_i(self.tiles.width, xy)
                                && self.tiles.tiles.get(tile_i).map(|t| t.is_floor()).unwrap_or(false) {
                                    Some(xy)
                                } else {
                                    None
                                }
                            });

                        if let Some(target_xy) = open_adjacent_spots.min_by_key(|target_xy|
                            manhattan_distance(*target_xy, self.player_position.xy())
                        )
                        && let Some(facing) = dir_to(FromTo { from: mob_xy, to: target_xy }) {
                            mob_mutations.push(entities::Mutation{
                                key: Key { xy: mob_xy },
                                facing,
                                target_xy,
                            });
                        }
                    },
                    _ => {} // This type of mob doesn't move
                }
            }

            for mutation in mob_mutations {
                self.mobs.apply(mutation);
            }
        }

        //
        // Render
        //

        // Render tiles

        for i in 0..self.tiles.tiles.len() {
            use TileIndex::*;
            let tile = self.tiles.tiles[i];

            let spec_tile = match tile.sprite_index {
                Wall(..) => wall_spec.tile(),
                Floor => floor_spec.tile(),
            };

            let tile_w = spec_tile.w;
            let tile_h = spec_tile.h;

            let xy = i_to_xy(self.tiles.width, i);

            let base_xy = unscaled::XY {
                x: unscaled::X(unscaled::Inner::from(xy.x.0) * tile_w.get()),
                y: unscaled::Y(unscaled::Inner::from(xy.y.0) * tile_h.get())
            };

            let (rect, s_xy) = match tile.sprite_index {
                Wall(index) => (
                    wall_spec.rect(base_xy),
                    wall_spec.xy_from_tile_sprite(index),
                ),
                Floor => (
                    floor_spec.rect(base_xy),
                    floor_spec.xy_from_tile_sprite(0u16),
                ),
            };

            commands.sspr(
                s_xy,
                command::Rect::from_unscaled(rect),
            );
        }

        // Render mobs

        let tiles_per_row = sword_spec.tiles_per_row();

        let tile = sword_spec.tile();
        let tile_w = tile.w;
        let tile_h = tile.h;

        let mut draw_at_position_pieces = |xy: XY, offset_xy, tile_sprite| {
            let base_xy = unscaled::XY {
                x: unscaled::X(unscaled::Inner::from(xy.x.0) * tile_w.get()),
                y: unscaled::Y(unscaled::Inner::from(xy.y.0) * tile_h.get())
            };

            match tile_sprite {
                TileSprite::Sword(t_s) => {
                    commands.sspr(
                        sword_spec.xy_from_tile_sprite(t_s),
                        command::Rect::from_unscaled(sword_spec.offset_rect(offset_xy, base_xy)),
                    );
                },
                TileSprite::ToggleWall(t_s) => {
                    commands.sspr(
                        toggle_wall_spec.xy_from_tile_sprite(t_s),
                        command::Rect::from_unscaled(toggle_wall_spec.offset_rect(offset_xy, base_xy)),
                    );
                },
            }
        };

        let mut draw_at_position = |position: Position, tile_sprite| {
            draw_at_position_pieces(position.xy(), position.offset(), tile_sprite)
        };

        macro_rules! sprite_with_facing {
            ($base: expr, $facing: expr) => {
                TileSprite::Sword(
                    $base.sword_inner_or_0() + tiles_per_row as SwordTileSpriteInner * $facing.index() as SwordTileSpriteInner
                )
            }
        }

        for (position, entity) in self.mobs.all() {
            // Don't draw things that are gone.
            if entity.flags & GONE == GONE { continue }

            let sprite = if entity.flags & RENDER_FACING == RENDER_FACING {
                sprite_with_facing!(entity.tile_sprite, entity.facing)
            } else {
                entity.tile_sprite
            };

            draw_at_position(position, sprite);
        }

        assert_eq!(self.player.flags & GONE, 0, "The player should never be gone!");
        assert_eq!(self.player.flags & RENDER_FACING, RENDER_FACING, "The player should always be rendered taking facing into account!");

        draw_at_position(
            self.player_position,
            sprite_with_facing!(self.player.tile_sprite, self.player.facing),
        );

        if let (staff_xy, EdgeHitKind::Neither) = staff_xy_pair {
            draw_at_position_pieces(
                staff_xy,
                self.player_position.offset(),
                sprite_with_facing!(STAFF_BASE, self.player.facing),
            );
        }
    }
}
