use gfx::{Commands};
//use gfx_sizes::ARGB;
#[allow(unused)]
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
//use vec1::{Grid1, Grid1Spec, vec1, Vec1};
use xs::{Seed, Xs};

use std::collections::{BTreeMap};

#[derive(Clone, Copy, Debug, Default)]
pub enum TileKind {
    #[default]
    Symbol,
    Warp,
}

impl TileKind {
    const ALL: [TileKind; 2] = [
        Self::Symbol,
        Self::Warp,
    ];
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Tile {
    pub kind: TileKind,
}

pub type Key = ();//QRS;

pub type Tiles = BTreeMap<Key, Tile>;

#[derive(Clone, Debug)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
    pub tiles: Tiles,
}

impl State {
    pub fn new(rng: &mut Xs, specs: &sprite::Specs) -> Self {
        let seed = xs::new_seed(rng);

        Self::init(seed, specs)
    }

    fn init(seed: Seed, _specs: &sprite::Specs) -> Self {
        let mut rng_ = xs::from_seed(seed);
        let rng = &mut rng_;

        let mut tiles = Tiles::new();

        tiles.insert(
            (),
            Tile {
                kind: TileKind::ALL[xs::range(rng, 0..TileKind::ALL.len() as u32) as usize],
            }
        );

        Self {
            seed,
            rng: rng_,
            tiles,
            //mobs
        }
    }

    #[allow(unused)]
    fn restart(&mut self, specs: &sprite::Specs) {
        *self = Self::init(self.seed, specs);
    }

    pub fn is_complete(&self) -> bool {
        false
    }

    fn tick(&mut self) {

    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        specs: &sprite::Specs,
        input: Input,
        _speaker: &mut Speaker,
    ) {
        //
        //
        // Update Section
        //
        //

        if input.pressed_this_frame(Button::START) {
            self.restart(specs);
        }

        self.tick();

        //
        //
        // Render Section
        //
        //

        for tile in self.tiles.values() {
            commands.sspr_override(
                specs.hex_twiddle_tiles.xy_from_tile_sprite(0u16),
                command::Rect::from_unscaled(specs.hex_twiddle_tiles.rect(<_>::default())),
                match tile.kind {
                    TileKind::Symbol => 0xFF3352E1,
                    TileKind::Warp => 0xFF30B06E,
                }
            );
        }
    }
}
