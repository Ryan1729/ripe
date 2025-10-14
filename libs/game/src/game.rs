use models::{Card, gen_card};
use platform_types::{command, unscaled};
use xs::{Xs, Seed};

type TileSprite = u8;

#[derive(Clone, Default)]
pub struct Tile {
    pub sprite: TileSprite,
}

pub type SegmentWidth = usize;

#[derive(Clone, Default)]
pub struct WorldSegment {
    pub width: SegmentWidth,
    pub tiles: Vec<Tile>,
}

#[derive(Clone, Default)]
pub struct State {
    pub rng: Xs,
    pub segment: WorldSegment,
}

impl State {
    pub fn new(seed: Seed) -> State {
        let mut rng = xs::from_seed(seed);

        let width = xs::range(&mut rng, 2..9) as SegmentWidth;

        let height = xs::range(&mut rng, 2..9) as usize;

        let len = width * height;
        let mut tiles = Vec::with_capacity(len);

        for _ in 0..len {
            tiles.push(Tile {
                sprite: xs::range(&mut rng, 0..2) as TileSprite,
            });
        }

        let segment = WorldSegment {
            width,
            tiles,
        };

        State {
            rng,
            segment,
            .. <_>::default()
        }
    }
}
