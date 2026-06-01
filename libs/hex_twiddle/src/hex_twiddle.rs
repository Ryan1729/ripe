use gfx::{Commands};
//use gfx_sizes::ARGB;
#[allow(unused)]
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use qrs::{QRS, QRSD, Q, R};
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

pub type Key = QRS;

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

        macro_rules! qr {
            ($q_inner: literal $(,)? $r_inner: literal) => {
                QRS {
                    q: Q($q_inner),
                    r: R($r_inner),
                }
            }
        }

        tiles.insert(
            qr!(0, 0),
            Tile {
                kind: TileKind::ALL[xs::range(rng, 0..TileKind::ALL.len() as u32) as usize],
            }
        );

        tiles.insert(
            qr!(0, 1),
            Tile {
                kind: TileKind::ALL[xs::range(rng, 0..TileKind::ALL.len() as u32) as usize],
            }
        );

        tiles.insert(
            qr!(0, 2),
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

        const X_Q_FACTOR: i16 = 2;
        const X_R_FACTOR: i16 = 0;
        
        const Y_Q_FACTOR: i16 = 1;
        const Y_R_FACTOR: i16 = 2;

        const HEX_X_SCALE: i16 = 13;
        const HEX_Y_SCALE: i16 = 8;
        
        const HEX_X_OFFSET: i16 = 160;
        const HEX_Y_OFFSET: i16 = 110;

        fn qrs_to_unscaled(qrs: QRS) -> unscaled::XY {
            let q = qrs.q.0;
            let r = qrs.r.0;

            let x = (X_Q_FACTOR * q + X_R_FACTOR * r) * HEX_X_SCALE + HEX_X_OFFSET;
            let y = (Y_Q_FACTOR * q + Y_R_FACTOR * r) * HEX_Y_SCALE + HEX_Y_OFFSET;

            unscaled::XY {
                x: unscaled::X(x.try_into().unwrap_or(0)),
                y: unscaled::Y(y.try_into().unwrap_or(0)),
            }
        }

        fn tile_xy(qrs: QRS, Tile { .. }: &Tile) -> unscaled::XY {
            qrs_to_unscaled(qrs)
        }

        for (at, tile) in self.tiles.iter() {
            let xy = tile_xy(*at, &tile);

            commands.sspr_override(
                specs.hex_twiddle_tiles.xy_from_tile_sprite(0u16),
                command::Rect::from_unscaled(specs.hex_twiddle_tiles.rect(xy)),
                match tile.kind {
                    TileKind::Symbol => 0xFF3352E1,
                    TileKind::Warp => 0xFF30B06E,
                }
            );
        }
    }
}
