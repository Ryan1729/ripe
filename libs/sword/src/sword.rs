///! S.W.O.R.D.: Staff Whacking Ordeal Required, Duh

use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Dir, Input, Speaker};
use vec1::{Vec1, vec1};
use xs::Xs;

use std::collections::BTreeMap;

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
    Floor(NeighborMask),
}

impl Default for TileIndex {
    fn default() -> Self {
        Self::Wall(<_>::default())
    }
}

impl TileIndex {
    pub fn neighbor_mask(self) -> NeighborMask {
        match self {
            TileIndex::Wall(mask)
            | TileIndex::Floor(mask) => mask,
        }
    }

    pub fn neighbor_mask_mut(&mut self) -> &mut NeighborMask {
        match self {
            TileIndex::Wall(mask)
            | TileIndex::Floor(mask) => mask,
        }
    }
}

pub type NeighborFlag = std::num::NonZeroU8;

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

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Y(pub Inner);

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
                }
            )+
        }
    }

    def!{ X, Y }

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
}
use xy::{X, Y, XY};

type TileSprite = u16;

fn to_s_xy(spec: &sprite::Spec<sprite::SWORD>, tile_sprite: TileSprite) -> sprite::XY<sprite::Renderable> {
    let tile = spec.tile();
    let tiles_per_row = spec.tiles_per_row();
    sprite::XY::<sprite::SWORD> {
        x: sprite::x(0) + sprite::W(tile_sprite as sprite::Inner % sprite::Inner::from(tiles_per_row)) * tile.w.get(),
        y: sprite::y(0) + sprite::H(tile_sprite as sprite::Inner / sprite::Inner::from(tiles_per_row)) * tile.h.get(),
    }.apply(spec)
}

const PLAYER_BASE: TileSprite = 0;
const STAFF_BASE: TileSprite = 1;
const STAIRS_TOP_LEFT_EDGE: TileSprite = 2;
const STAIRS_TOP_EDGE: TileSprite = STAIRS_TOP_LEFT_EDGE + 1;
const STAIRS_TOP_RIGHT_EDGE: TileSprite = STAIRS_TOP_LEFT_EDGE + 2;

type Tile = TileSprite;


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
enum Dir8 {
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

#[derive(Clone, Debug)]
pub struct State {
    pub rng: Xs,
    pub player: Entity,
    pub facing: Dir8,
    pub mobs: Entities,
    pub tiles: Vec1<Tile>,
}

impl State {
    pub fn new(rng: &mut Xs) -> Self {
        let seed = xs::new_seed(rng);

        let mut rng = xs::from_seed(seed);

        let mut player = Entity::default();
        player.tile_sprite = PLAYER_BASE;

        let mut mobs = Entities::default();

        let y = xy::Y::default();

        let mut offset = 0;
        for key in [
            Key {
                xy: XY { x: xy::X(3), y },
            },
            Key {
                xy: XY { x: xy::X(4), y },
            },
            Key {
                xy: XY { x: xy::X(5), y },
            },
        ] {
            mobs.insert(
                key,
                Entity {
                    position: Position::from(key.xy),
                    tile_sprite: STAIRS_TOP_LEFT_EDGE + offset,
                }
            );
            offset += 1;
        }


        let mut tiles = vec1![
            // Placeholder for once we have the various corner and edge tiles set up
            5
        ];

        Self {
            rng,
            player,
            facing: <_>::default(),
            mobs,
            tiles,
        }
    }

    pub fn is_complete(&self) -> bool {
        if let Some(mob) = self.mobs.get(&self.player.key()) {
            return mob.tile_sprite >= STAIRS_TOP_LEFT_EDGE && mob.tile_sprite <= STAIRS_TOP_RIGHT_EDGE;
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
        spec: &sprite::Spec::<sprite::SWORD>,
        input: Input,
        speaker: &mut Speaker,
    ) {
        //
        // Update
        //

        self.tick();

        if let Some(dir) = input.dir_pressed_this_frame() {
            let (new_xy, _) = xy_in_dir(self.player.position.xy(), dir.into());
            self.player.position.set_xy(new_xy);
        } else if input.pressed_this_frame(Button::A) {
            self.facing = self.facing.counter_clockwise();
        } else if input.pressed_this_frame(Button::B) {
            self.facing = self.facing.clockwise();
        }

        //
        // Render
        //

        let tiles_per_row = spec.tiles_per_row();

        let tile = spec.tile();
        let tile_w = tile.w;
        let tile_h = tile.h;

        let mut draw_at_position_pieces = |xy: XY, offset_xy, tile_sprite| {
            let base_xy = unscaled::XY {
                x: unscaled::X(unscaled::Inner::from(xy.x.0) * tile_w.get()),
                y: unscaled::Y(unscaled::Inner::from(xy.y.0) * tile_h.get())
            };

            commands.sspr(
                to_s_xy(spec, tile_sprite),
                command::Rect::from_unscaled(spec.offset_rect(offset_xy, base_xy)),
            );
        };

        let mut draw_at_position = |position: Position, tile_sprite| {
            draw_at_position_pieces(position.xy(), position.offset(), tile_sprite)
        };

        for entity in self.mobs.values() {
            draw_at_position(entity.position, entity.tile_sprite);
        }

        let facing_index = self.facing.index();

        draw_at_position(self.player.position, dbg!(self.player.tile_sprite + tiles_per_row as TileSprite * facing_index as TileSprite));

        if let (staff_xy, EdgeHitKind::Neither) = xy_in_dir(self.player.position.xy(), self.facing) {
            draw_at_position_pieces(staff_xy, self.player.position.offset(), STAFF_BASE + tiles_per_row as TileSprite * facing_index as TileSprite);
        }
    }
}
