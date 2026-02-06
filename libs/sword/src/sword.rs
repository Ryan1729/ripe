use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Dir, Input, Speaker};
use vec1::{Vec1, vec1};
use xs::Xs;

use std::collections::BTreeMap;

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

type TileSprite = u8;

const TILES_PER_ROW: TileSprite = 8;

fn to_s_xy(spec: &sprite::Spec<sprite::SWORD>, tile_sprite: TileSprite) -> sprite::XY<sprite::Renderable> {
    let tile = spec.tile();
    sprite::XY::<sprite::SWORD> {
        x: sprite::x(0) + sprite::W(tile_sprite as sprite::Inner % sprite::Inner::from(TILES_PER_ROW)) * tile.w.get(),
        y: sprite::y(0) + sprite::H(tile_sprite as sprite::Inner / sprite::Inner::from(TILES_PER_ROW)) * tile.h.get(),
    }.apply(spec)
}

const STAFF_BASE: TileSprite = 0;
const PLAYER_BASE: TileSprite = TILES_PER_ROW;
const STAIRS_TOP_LEFT_EDGE: TileSprite = TILES_PER_ROW * 2;
const STAIRS_TOP_EDGE: TileSprite = STAIRS_TOP_LEFT_EDGE + 1;
const STAIRS_TOP_RIGHT_EDGE: TileSprite = STAIRS_TOP_LEFT_EDGE + 2;

type Tile = TileSprite;


mod position {
    use super::XY;

    #[derive(Clone, Debug, Default)]
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
                offset::Point::from(self.xy),
            );
            self.xy = xy;
        }

        pub fn offset(&self) -> offset::XY {
            self.offset
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

fn xy_in_dir(xy: XY, dir: Dir8) -> Option<XY> {
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

    // This can happen due to saturation
    if new_x == x
    && new_y == y {
        return None
    }

    Some(XY { x: new_x, y: new_y })
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
            TILES_PER_ROW * 3
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

        if let Some(dir) = input.dir_pressed_this_frame() {
            if let Some(new_xy) = xy_in_dir(self.player.position.xy(), dir.into()) {
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

        let tile = spec.tile();
        let tile_w = tile.w;
        let tile_h = tile.h;

        let mut draw_tile_sprite = |xy: XY, tile_sprite| {
            commands.sspr(
                to_s_xy(spec, tile_sprite),
                command::Rect::from_unscaled(unscaled::Rect {
                    x: unscaled::X(unscaled::Inner::from(xy.x.0) * tile_w.get()),
                    y: unscaled::Y(unscaled::Inner::from(xy.y.0) * tile_h.get()),
                    w: tile_w,
                    h: tile_h,
                }),
            );
        };

        for entity in self.mobs.values() {
            draw_tile_sprite(entity.position.xy(), entity.tile_sprite);
        }

        let facing_index = self.facing.index();

        draw_tile_sprite(self.player.position.xy(), self.player.tile_sprite + facing_index);

        if let Some(staff_xy) = xy_in_dir(self.player.position.xy(), self.facing) {
            draw_tile_sprite(staff_xy, STAFF_BASE + facing_index);
        }
    }
}
