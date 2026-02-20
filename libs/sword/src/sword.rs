///! S.W.O.R.D.: Staff Whacking Ordeal Required, Duh

use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Dir, Input, Speaker};
use vec1::{Vec1, vec1};
use xs::Xs;

use std::collections::BTreeMap;
use std::num::NonZeroU8;

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
    type Inner = u8;

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
                    pub fn dec(&self) -> Self {
                        Self(self.0.saturating_sub(1))
                    }

                    pub fn inc(&self) -> Self {
                        Self(self.0.saturating_add(1))
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
}
use xy::{X, Y, XY, W, H, WH};

type SwordTileSpriteInner = u8;

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Debug, Default)]
pub struct Entity {
    pub position: Position,
    pub tile_sprite: TileSprite,
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

pub fn i_to_xy(width: TilesWidth, index: usize) -> XY {
    XY {
        x: xy::x((index % usize::from(width.get())) as _),
        y: xy::y((index / usize::from(width.get())) as _),
    }
}

pub enum XYToIError {
    XPastWidth
}

pub fn xy_to_i(width: TilesWidth, xy: XY) -> Result<usize, XYToIError> {
    let width_usize = usize::from(width.get());

    let x_usize = xy.x.usize();
    if x_usize >= width_usize {
        return Err(XYToIError::XPastWidth);
    }

    Ok(xy.y.usize() * width_usize + x_usize)
}

pub type TilesWidth = NonZeroU8;

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
        mobs.get(&key).is_none()
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub rng: Xs,
    pub player: Entity,
    pub facing: Dir8,
    pub mobs: Entities,
    pub tiles: Tiles,
}

impl State {
    pub fn new(rng: &mut Xs) -> Self {
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

        let mut offset = 0;
        for key in [
            Key {
                xy: XY { x: xy::X(6), y },
            },
            Key {
                xy: XY { x: xy::X(7), y },
            },
            Key {
                xy: XY { x: xy::X(8), y },
            },
        ] {
            insert_entity!(Entity {
                position: Position::from(key.xy),
                tile_sprite: TileSprite::Sword(STAIRS_TOP_LEFT_EDGE.sword_inner_or_0() + offset),
            });
            offset += 1;
        }

        use TileIndex::*;

        let width = TilesWidth::new(10).expect("Don't set a 0 width!");
        let mut tiles = {
            const W: Tile = Wall(0);
            const F: Tile = Floor;
            vec1![
                F, F, F, F, F, W, F, F, F, F,
                F, F, F, F, F, W, F, F, F, F,
                F, F, F, F, F, W, W, F, W, W,
                F, F, F, F, F, F, F, F, F, F,
                W, F, W, F, F, F, F, F, F, F,
                W, W, W, F, F, F, F, F, F, F,
                W, W, W, F, F, F, F, F, F, F,
            ]
        };

        let switch_key = Key {
            xy: XY { x: xy::x(1), y: xy::y(4) },
        };

        type ToggleWallSpecWidth = NonZeroU8;

        // We have the default as wall elsewhere, so let's be consistent.
        type IsFloorFlag = bool;
        const IS_WALL: IsFloorFlag = false;
        const IS_FLOOR: IsFloorFlag = true;

        struct ToggleWallSpec {
            width: ToggleWallSpecWidth,
            // TODO? pack these tightly? Does this live long enough for us to care?
            tiles: Vec1<IsFloorFlag>,
            base_wh: WH,
        }

        let wall_spec: ToggleWallSpec = ToggleWallSpec {
            width: ToggleWallSpecWidth::new(1).expect("Don't set a 0 width!"),
            tiles: {
                const W: IsFloorFlag = IS_WALL;
                const F: IsFloorFlag = IS_FLOOR;
                vec1![
                    W,
                    W,
                    W,
                    W,
                ]
            },
            base_wh: WH { w: xy::w(5), h: xy::h(3) },
        };

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

        // Add switch
        insert_entity!(Entity {
            position: Position::from(switch_key.xy),
            tile_sprite: SWITCH_BASE,
        });

        // Add toggleable walls
        for index in 0..wall_spec.tiles.len() {
            if wall_spec.tiles[index] == IS_WALL {
                let width = usize::from(width.get());

                // Assume everything not set is a floor, to avoid merging 
                // with walls from other specs.
                let mut output_mask = 0b1111_1111;
    
                macro_rules! set {
                    (-, $subtrahend: expr, $mask: ident) => {
                        if let Some(&tile) = index.checked_sub($subtrahend)
                            .and_then(|i| wall_spec.tiles.get(i)) {
            
                            // TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
                            // we can use highest_one instead.
                            let shift = NeighborFlag::BITS - 1 - $mask.leading_zeros();
    
                            if tile == IS_WALL {
                                output_mask &= !(1 << shift);
                            }
                        }
                    };
                    (+, $addend: expr, $mask: ident) => {
                        if let Some(&tile) = index.checked_add($addend)
                            .and_then(|i| wall_spec.tiles.get(i)) {
            
                            // TODO once https://github.com/rust-lang/rust/issues/145203 is avilable on stable
                            // we can use highest_one instead.
                            let shift = NeighborFlag::BITS - 1 - $mask.leading_zeros();
    
                            if tile == IS_WALL {
                                output_mask &= !(1 << shift);
                            }
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
    
                let xy = i_to_xy(wall_spec.width, index) + wall_spec.base_wh;

                insert_entity!(Entity {
                    position: Position::from(xy),
                    tile_sprite: TileSprite::ToggleWall(output_mask),
                });
            }
        }

        Self {
            rng,
            player,
            facing: <_>::default(),
            mobs,
            tiles: Tiles {
                width,
                tiles,
            },
        }
    }

    pub fn is_complete(&self) -> bool {
        if let Some(mob) = self.mobs.get(&self.player.key()) {
            return mob.tile_sprite.sword_inner_or_0() >= STAIRS_TOP_LEFT_EDGE.sword_inner_or_0()
                    && mob.tile_sprite.sword_inner_or_0() <= STAIRS_TOP_RIGHT_EDGE.sword_inner_or_0();
        }
        false
    }

    pub fn all_entities(&self) -> impl Iterator<Item=&Entity> {
        std::iter::once(&self.player).chain(self.mobs.values())
    }

    pub fn all_entities_mut(&mut self) -> impl Iterator<Item=&mut Entity> {
        std::iter::once(&mut self.player).chain(self.mobs.values_mut())
    }

    fn tick(&mut self) {
        //
        // Advance timers
        //

        for entity in self.all_entities_mut() {
            entity.position.decay();
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
            draw_at_position(entity.position, entity.tile_sprite);
        }

        let facing_index = self.facing.index();

        draw_at_position(
            self.player.position,
            TileSprite::Sword(
                self.player.tile_sprite.sword_inner_or_0() + tiles_per_row as SwordTileSpriteInner * facing_index as SwordTileSpriteInner
            ),
        );

        if let (staff_xy, EdgeHitKind::Neither) = xy_in_dir(self.player.position.xy(), self.facing) {
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
