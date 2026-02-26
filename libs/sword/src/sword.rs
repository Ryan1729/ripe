///! S.W.O.R.D.: Staff Whacking Ordeal Required, Duh

use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Dir, Input, Speaker};
use vec1::{Vec1, vec1};
use xs::Xs;

use std::collections::BTreeMap;
use std::num::{NonZeroU8, NonZeroU16};

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
    pub fn is_floor_mask(self) -> NeighborMask {
        match self {
            TileIndex::Wall(..) => 0,
            TileIndex::Floor => 1,
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
         self.sword_inner_or_0() >= STAIRS_TOP_LEFT_EDGE.sword_inner_or_0()
             && self.sword_inner_or_0() <= STAIRS_TOP_RIGHT_EDGE.sword_inner_or_0()
    }
}

const PLAYER_BASE: TileSprite = TileSprite::Sword(0);
const STAFF_BASE: TileSprite = TileSprite::Sword(1);
const STAIRS_TOP_LEFT_EDGE: TileSprite = TileSprite::Sword(2);
#[allow(unused)]
const STAIRS_TOP_EDGE: TileSprite = TileSprite::Sword(STAIRS_TOP_LEFT_EDGE.sword_inner_or_0() + 1);
const STAIRS_TOP_RIGHT_EDGE: TileSprite = TileSprite::Sword(STAIRS_TOP_LEFT_EDGE.sword_inner_or_0() + 2);
const SWITCH_BASE: TileSprite = TileSprite::Sword(40);
const SWITCH_HIT: TileSprite = TileSprite::Sword(SWITCH_BASE.sword_inner_or_0() + 1);

type Tile = TileIndex;


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

#[derive(Clone, Debug, Default)]
pub struct Entity {
    pub position: Position,
    pub tile_sprite: TileSprite,
    pub toggle_group_id: ToggleGroupId,
    pub flags: EntityFlags,
}

impl Entity {
    pub fn key(&self) -> Key {
        Key {
            xy: self.position.xy(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub xy: XY,
}

pub type Entities = BTreeMap<Key, Entity>;

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
        .map(|t| t.is_floor_mask() == 1)
        .unwrap_or(false)
}

fn can_walk_onto(mobs: &Entities, tiles: &Tiles, key: Key) -> bool {
    can_walk_onto_tile(tiles, key.xy) && {
        match mobs.get(&key) {
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

#[derive(Clone, Debug)]
pub struct State {
    pub rng: Xs,
    pub player: Entity,
    pub facing: Dir8,
    pub mobs: Entities,
    pub tiles: Tiles,
    pub animations: Animations,
}

impl State {
    pub fn new(rng: &mut Xs, wall_spec: &sprite::Spec<sprite::Wall>) -> Self {
        let seed = xs::new_seed(rng);

        let mut rng = xs::from_seed(seed);

        let mut player = Entity::default();
        player.tile_sprite = PLAYER_BASE;

        let mut mobs = Entities::default();

        let y = xy::Y::default();

        macro_rules! insert_entity {
            ($entity: expr) => ({
                let entity = $entity;
                mobs.insert(
                    Key {
                        xy: entity.position.xy(),
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

            // Start generating random start and end locations, right next to each other.

            let tiles_length = max_tile_w * max_tile_h;

            // TODO confirm this division is right, and doesn't need a + 1 or something.
            let proto_width = TilesWidth::new(max_tile_w / 2).unwrap_or(TilesWidth::MIN);
            let proto_height = TilesWidth::new(max_tile_h / 2).unwrap_or(TilesWidth::MIN);;
            let proto_tiles_length = usize::from(proto_width.get()) * usize::from(proto_height.get());


            let mut proto_tiles = vec1![0; proto_tiles_length];

            for i in 0..proto_tiles.len() {
                let current_xy = i_to_xy(width, i);

                for dir in Dir::ALL {
                    if let Some(new_xy) = current_xy.checked_push(dir) {
                        if let Ok(new_index) = xy_to_i(width, new_xy) {
                            if let Ok([flags, adjacent_flags]) = proto_tiles.get_disjoint_mut([i, new_index]) {
                                *flags |= dir.flag();
                                *adjacent_flags |= dir.opposite().flag();
                            }
                        }
                    }
                }
            }

            const W: Tile = Wall(0);
            const F: Tile = Floor;

            // Convert to 1-thick walls

            let mut tiles = vec1![W; tiles_length];

            for proto_i in 0..proto_tiles.len() {
                let proto_tile_flags = proto_tiles[proto_i];

                if proto_tile_flags != 0 {
                    // The cell is open on at least one side.
                    let proto_xy = i_to_xy(proto_width, proto_i);

                    let tile_xy = XY { x: proto_xy.x.double().inc(), y: proto_xy.y.double().inc() };

                    if let Ok(tile_i) = xy_to_i(width, tile_xy) {
                        if let Some(el) = tiles.get_mut(tile_i) { *el = F; }
                    }

                    if proto_tile_flags & Dir::Right.flag() != 0 {
                        if let Ok(tile_right_i) = xy_to_i(width, tile_xy + W::ONE) {
                            if let Some(el) = tiles.get_mut(tile_right_i) { *el = F; }
                        }
                    }

                    if proto_tile_flags & Dir::Up.flag() != 0 {
                        if let Ok(tile_down_i) = xy_to_i(width, tile_xy + H::ONE) {
                            if let Some(el) = tiles.get_mut(tile_down_i) { *el = F; }
                        }
                    }
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

                pub fn non_edge_index(width: TilesWidth, tiles: &[Tile], rng: &mut Xs) -> Result<Index, NonEdgeError> {
                    if width.get() < 3 {
                        return Err(NonEdgeError::WidthTooSmall);
                    }

                    // The min/max non-edge corners; The corners of the rectangle of non-edge pieces.
                    let min_corner_xy = xy::XY { x: xy::x(1), y: xy::y(1) };
                    let height = xy::Inner::try_from(tiles.len())? / width.get();
                    if height < 3 {
                        return Err(NonEdgeError::TilesTooShort);
                    }

                    // -2 because -1 to get to the last index, then another to go to the second last index.
                    let max_corner_xy = xy::XY { x: xy::x(width.get() - 2), y: xy::y(height - 2) };

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

            let exit_index_result = random::non_edge_index(width, &tiles, &mut rng);
            debug_assert!(exit_index_result.is_ok(), "got {exit_index_result:?}");
            let exit_index = exit_index_result.unwrap_or_default();

            // A lot of things here rely on the starting exit_index being an non-edge tile!

            //
            // Place Exit
            //
            tiles[exit_index - 1] = F;
            tiles[exit_index] = F;
            tiles[exit_index + 1] = F;

            let base_exit_xy = i_to_xy(width, exit_index);

            let mut offset = 0;
            for key in [
                Key {
                    xy: XY { x: base_exit_xy.x - xy::w(1), y: base_exit_xy.y },
                },
                Key {
                    xy: XY { x: base_exit_xy.x, y: base_exit_xy.y },
                },
                Key {
                    xy: XY { x: base_exit_xy.x + xy::w(1), y: base_exit_xy.y },
                },
            ] {
                insert_entity!(Entity {
                    position: Position::from(key.xy),
                    tile_sprite: TileSprite::Sword(STAIRS_TOP_LEFT_EDGE.sword_inner_or_0() + offset),
                    ..<_>::default()
                });
                offset += 1;
            }

            //
            // Select initial spot for start
            //

            let mut start_xy = XY { x: base_exit_xy.x, y: base_exit_xy.y + xy::h(1) };

            macro_rules! floor_at_start {
                () => {
                    let start_index_result = xy_to_i(width, start_xy);

                    debug_assert!(start_index_result.is_ok(), "got {start_index_result:?}");

                    let start_index = start_index_result.unwrap_or_default();

                    tiles[start_index] = F;
                }
            }
            floor_at_start!();

            //
            // Perform random complication actions that preserve the solvabilty.
            //
            let complication_count = 10;

            enum Complication {
                ExtendPath,
            }

            for _ in 0..complication_count {
                // TODO define multiple and pick randomly
                let complication = Complication::ExtendPath;

                match complication {
                    Complication::ExtendPath => {
                        let mut candidate_directions: [Dir; 4] = <_>::default();
                        let mut candidate_directions_len = 0;

                        for dir in Dir::ALL {
                            let mut viable = false;

                            if let Some(new_start_xy) = start_xy.checked_push(dir) {
                                if let Ok(new_start_index) = xy_to_i(width, new_start_xy) {
                                    if let Some(&tile) = tiles.get(new_start_index) {
                                        viable = tile == W;
                                    }
                                }
                            }

                            if viable {
                                candidate_directions[
                                    usize::try_from(candidate_directions_len)
                                    .expect("Not expected to be run on lower than 32 bit systems!")
                                ] = dir;
                                candidate_directions_len += 1;
                            }
                        }

                        if candidate_directions_len > 0 {
                            let i = xs::range(&mut rng, 0..candidate_directions_len);
                            let dir = candidate_directions[
                                usize::try_from(i)
                                    .expect("Not expected to be run on lower than 32 bit systems!")
                            ];

                            if let Some(new_start_xy) = start_xy.checked_push(dir) {
                                start_xy = new_start_xy;
                                floor_at_start!();
                            }
                        }
                    }
                }
            }

            player.position = start_xy.into();

            tiles
        };

        let switch_key = Key {
            xy: XY { x: xy::x(1), y: xy::y(4) },
        };

        // TODO automatically add floor to the tiles,
        // so we don't need to add it in the config file.
        // (Is a column along the right edge always enough?)
        let wall_specs: [ToggleWallSpec; 1] = [
            ToggleWallSpec {
                width: ToggleWallSpecWidth::new(2).expect("Don't set a 0 width!"),
                tiles: {
                    const W: IsFloorFlag = IS_WALL;
                    const F: IsFloorFlag = IS_FLOOR;
                    vec1![
                        W, F,
                        W, F,
                        W, F,
                        W, F,
                    ]
                },
                base_wh: WH { w: xy::w(5), h: xy::h(3) },
            },
        ];

        // Set the indexes from the surrounding tiles.
        for index in 0..tiles.len() {
            if tiles[index].is_floor_mask() == 0 {
                let width = usize::from(width.get());

                // Assume everything not set is a wall, for maximum merging.
                let mut output_mask = 0;

                macro_rules! set {
                    (-, $subtrahend: expr, $mask: ident) => {
                        if let Some(tile) = index.checked_sub($subtrahend)
                            .and_then(|i| tiles.get(i)) {

                            // TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
                            // we can use highest_one instead.
                            let shift = NeighborFlag::BITS - 1 - $mask.leading_zeros();

                            output_mask |= tile.is_floor_mask() << shift;
                        }
                    };
                    (+, $addend: expr, $mask: ident) => {
                        if let Some(tile) = index.checked_add($addend)
                            .and_then(|i| tiles.get(i)) {

                            // TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
                            // we can use highest_one instead.
                            let shift = NeighborFlag::BITS - 1 - $mask.leading_zeros();

                            output_mask |= tile.is_floor_mask() << shift;
                        }
                    };
                }

                set!(-, width + 1, UPPER_LEFT);
                set!(-, width, UPPER_MIDDLE);
                set!(-, width - 1, UPPER_RIGHT);
                set!(-, 1, LEFT_MIDDLE);

                set!(+, 1, RIGHT_MIDDLE);
                set!(+, width - 1, LOWER_RIGHT);
                set!(+, width, LOWER_MIDDLE);
                set!(+, width + 1, LOWER_LEFT);

                if let Wall(mask_ref) = &mut tiles[index] {
                    *mask_ref = output_mask
                } else {
                    unreachable!("Tile changed while we were looking at it?!");
                }
            }
        }

        //// Add switch
        //insert_entity!(Entity {
            //position: Position::from(switch_key.xy),
            //tile_sprite: SWITCH_BASE,
            //toggle_group_id: FIRST_GROUP,
            //..<_>::default()
        //});
//
        //// Add toggleable walls
        //let mut free_group_id = FIRST_GROUP;
        //for wall_spec in wall_specs {
            //for index in 0..wall_spec.tiles.len() {
                //if wall_spec.tiles[index] == IS_WALL {
                    //let width = usize::from(wall_spec.width.get());
//
                    //// Assume everything not set is a floor, to avoid merging
                    //// with walls from other specs.
                    //let mut output_mask = 0b1111_1111;
//
                    //macro_rules! set {
                        //(-, $subtrahend: expr, $mask: ident) => {
                            //if let Some(&tile) = index.checked_sub($subtrahend)
                                //.and_then(|i| wall_spec.tiles.get(i)) {
//
                                //// TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
                                //// we can use highest_one instead.
                                //let shift = NeighborFlag::BITS - 1 - $mask.leading_zeros();
//
                                //if tile == IS_WALL {
                                    //output_mask &= !(1 << shift);
                                //}
                            //}
                        //};
                        //(+, $addend: expr, $mask: ident) => {
                            //if let Some(&tile) = index.checked_add($addend)
                                //.and_then(|i| wall_spec.tiles.get(i)) {
//
                                //// TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
                                //// we can use highest_one instead.
                                //let shift = NeighborFlag::BITS - 1 - $mask.leading_zeros();
//
                                //if tile == IS_WALL {
                                    //output_mask &= !(1 << shift);
                                //}
                            //}
                        //};
                    //}
//
                    //set!(-, width + 1, UPPER_LEFT);
                    //set!(-, width, UPPER_MIDDLE);
                    //set!(-, width - 1, UPPER_RIGHT);
                    //set!(-, 1, LEFT_MIDDLE);
//
                    //set!(+, 1, RIGHT_MIDDLE);
                    //set!(+, width - 1, LOWER_RIGHT);
                    //set!(+, width, LOWER_MIDDLE);
                    //set!(+, width + 1, LOWER_LEFT);
//
                    //let xy = i_to_xy(wall_spec.width, index) + wall_spec.base_wh;
//
                    //insert_entity!(Entity {
                        //position: Position::from(xy),
                        //tile_sprite: TileSprite::ToggleWall(output_mask),
                        //toggle_group_id: free_group_id,
                        //..<_>::default()
                    //});
                //}
            //}
//
            //free_group_id += 1;
        //}

        Self {
            rng,
            player,
            facing: <_>::default(),
            mobs,
            tiles: Tiles {
                width,
                tiles,
            },
            animations: <_>::default(),
        }
    }

    pub fn is_complete(&self) -> bool {
        if let Some(mob) = self.mobs.get(&self.player.key()) {
            return mob.tile_sprite.is_stairs();
        }
        false
    }

    pub fn all_entities(&self) -> impl Iterator<Item=&Entity> {
        std::iter::once(&self.player).chain(self.mobs.values())
    }

    pub fn all_entities_mut(&mut self) -> impl Iterator<Item=&mut Entity> {
        std::iter::once(&mut self.player).chain(self.mobs.values_mut())
    }

    fn staff_xy_pair(&self) -> (XY, EdgeHitKind) {
        xy_in_dir(self.player.position.xy(), self.facing)
    }

    fn tick(&mut self) {
        let staff_xy_pair = self.staff_xy_pair();

        //
        // Advance timers
        //

        for entity in self.all_entities_mut() {
            entity.position.decay();
        }

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

                        if let Some(mob) = self.mobs.get_mut(&animation.target_key) {
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

        if let Some(dir) = input.dir_pressed_this_frame() {
            // Walk
            let (new_xy, _) = xy_in_dir(self.player.position.xy(), dir.into());

            if can_walk_onto(&self.mobs, &self.tiles, Key { xy: new_xy }) {
                self.player.position.set_xy(new_xy);
            }
        } else if input.pressed_this_frame(Button::A) {
            self.facing = self.facing.counter_clockwise();
        } else if input.pressed_this_frame(Button::B) {
            self.facing = self.facing.clockwise();
        }

        let staff_xy_pair = self.staff_xy_pair();

        if let &(staff_xy, EdgeHitKind::Neither) = &staff_xy_pair {
            let key = Key { xy: staff_xy };

            enum PostEffect {
                NoOp,
                Toggle(ToggleGroupId),
            }
            use PostEffect::*;
            let mut effect = PostEffect::NoOp;

            match self.mobs.get_mut(&key) {
                Some(hit_mob) if hit_mob.tile_sprite == SWITCH_BASE => {
                    hit_mob.tile_sprite = SWITCH_HIT;

                    // Start animation timer
                    self.animations.push(Animation::reset(key));

                    // TODO Good place for SFX


                    // Toggle relevant mobs
                    if hit_mob.toggle_group_id != NULL_GROUP {
                        effect = Toggle(hit_mob.toggle_group_id);
                    }
                }
                Some(_) | None => {}
            }

            match effect {
                NoOp => {},
                Toggle(group_id) => {
                    // TODO? Is it worth building an acceleration structure for this loopup?
                    for mob in self.mobs.values_mut() {
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
            }
        }

        //
        // Render
        //

        // Render tiles

        for i in 0..self.tiles.tiles.len() {
            use TileIndex::*;
            let tile = self.tiles.tiles[i];

            let spec_tile = match tile {
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

            let (rect, s_xy) = match tile {
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

        for entity in self.mobs.values() {
            // Don't draw things that are gone.
            if entity.flags & GONE == GONE { continue }

            draw_at_position(entity.position, entity.tile_sprite);
        }

        let facing_index = self.facing.index();

        assert_eq!(self.player.flags & GONE, 0, "The player should never be gone!");

        draw_at_position(
            self.player.position,
            TileSprite::Sword(
                self.player.tile_sprite.sword_inner_or_0() + tiles_per_row as SwordTileSpriteInner * facing_index as SwordTileSpriteInner
            ),
        );

        if let (staff_xy, EdgeHitKind::Neither) = staff_xy_pair {
            draw_at_position_pieces(
                staff_xy,
                self.player.position.offset(),
                TileSprite::Sword(
                    STAFF_BASE.sword_inner_or_0() + tiles_per_row as SwordTileSpriteInner * facing_index as SwordTileSpriteInner
                ),
            );
        }
    }
}
