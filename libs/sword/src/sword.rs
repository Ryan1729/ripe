use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Dir, Input, Speaker};
use vec1::{Vec1, vec1};
use xs::Xs;

use std::collections::BTreeMap;

// Sketching this out, this is seeming a lot like the main game parts, which makes sense, 
// because the intended game has te same grid based movement etc.
// But I am reluctant to actually make them depend on too many of the same things, since
// changing how one of them works shouldn't affect the other.
// On the other hand, it doesn't seem liek a trivial amount of code to do a bunch of stuff
// that I think both of them will both do for the forseeable future. (again, with a risk of
// partial divergence down the line)
// Thus, I am thinking it makes sense to copy some code into here from the main game parts.
// But, before I do that, I think it's worthwhile to take some time try to simplify and 
// reduce the amount of unneeded lines of that code in-situ, so there's less that there are
// duplicate versions of. More generally, it's worth considering completing any TODOs in the
// to-be-duplicated code before the copy, as well.

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

const PLAYER_BASE: TileSprite = TILES_PER_ROW;
const STAIRS_TOP_LEFT_EDGE: TileSprite = TILES_PER_ROW * 2;
const STAIRS_TOP_EDGE: TileSprite = STAIRS_TOP_LEFT_EDGE + 1;
const STAIRS_TOP_RIGHT_EDGE: TileSprite = STAIRS_TOP_LEFT_EDGE + 2;

type Tile = TileSprite;

#[derive(Clone, Debug, Default)]
pub struct Entity {
    pub xy: XY,
    pub tile_sprite: TileSprite,
}

impl Entity {
    pub fn key(&self) -> Key {
        Key {
            xy: self.xy,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub xy: XY,
}

pub type Entities = BTreeMap<Key, Entity>;

fn xy_in_dir(xy: XY, dir: Dir) -> Option<XY> {
    use Dir::*;

    let x = xy.x;
    let y = xy.y;

    let (new_x, new_y) = match dir {
        Left => (x.dec(), y),
        Right => (x.inc(), y),
        Up => (x, y.dec()),
        Down => (x, y.inc()),
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
                    xy: key.xy,
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
        if let Some(dir) = input.dir_pressed_this_frame() {
            if let Some(XY { x: new_x, y: new_y }) = xy_in_dir(self.player.xy, dir) {
                // TODO? Worth making every update to any entities x/y update the offset?
                //self.player.offset_x = offset::X::from(self.player.xy.x) - offset::X::from(new_x);
                //self.player.offset_y = offset::Y::from(self.player.xy.y) - offset::Y::from(new_y);
    //
                self.player.xy.x = new_x;
                self.player.xy.y = new_y;
            }
        }

        let tile = spec.tile();
        let tile_w = tile.w;
        let tile_h = tile.h;

        for entity in self.all_entities() {
            commands.sspr(
                to_s_xy(spec, entity.tile_sprite),
                command::Rect::from_unscaled(unscaled::Rect {
                    x: unscaled::X(unscaled::Inner::from(entity.xy.x.0) * tile_w.get()),
                    y: unscaled::Y(unscaled::Inner::from(entity.xy.y.0) * tile_h.get()),
                    w: tile_w,
                    h: tile_h,
                }),
            );
        }
    }
}
