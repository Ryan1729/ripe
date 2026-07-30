use gfx::{Commands};
use gfx_sizes::{ARGB};
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use qrs::{QRS, QRSD, Q, R, qr};
use xs::{Seed, Xs};

type TileSprite = u8;

enum GoalFrame {
    Zero, // Blank
    One,
    Two,
    Three,
    Four,
}

enum TileKind {
    Wall,
    Empty,
    Goal(GoalFrame),
}

struct Tile {
    kind: TileKind,
}

#[derive(Clone, Debug, Default)]
struct Tiles; // placeholder

#[derive(Clone, Debug, Default)]
struct Mobs; // placeholder

/*
const X_Q_FACTOR: i16 = 2;
const X_R_FACTOR: i16 = 0;

const Y_Q_FACTOR: i16 = 1;
const Y_R_FACTOR: i16 = 2;

const HEX_X_SCALE: i16 = 22;
const HEX_Y_SCALE: i16 = 25;

const HEX_X_OFFSET: i16 = 160;
const HEX_Y_OFFSET: i16 = 140;

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
*/

#[derive(Clone, Debug, Default)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
    pub tiles: Tiles,
    pub mobs: Mobs,
}

impl State {
    pub fn new(rng: &mut Xs, specs: &sprite::Specs) -> Self {
        let seed = xs::new_seed(rng);

        Self::init(seed, specs)
    }

    fn init(seed: Seed, _specs: &sprite::Specs) -> Self {
        let mut rng_ = xs::from_seed(seed);
        let rng = &mut rng_;

        let tiles = Tiles::default();
        let mobs = Mobs::default();

        Self {
            seed,
            rng: rng_,
            tiles,
            mobs,
            .. <_>::default()
        }
    }

    #[allow(unused)]
    fn restart(&mut self, specs: &sprite::Specs) {
        *self = Self::init(self.seed, specs);
    }

    pub fn all_offsets_settled(&self) -> bool {
        false
    }

    pub fn is_complete(&self) -> bool {
        // If the animations are not settled, delay completion
        if !self.all_offsets_settled() {
            return false
        }

        false
    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        specs: &sprite::Specs,
        input: Input,
        speaker: &mut Speaker,
    ) {
        let tile_wh = specs.cube_maze_sides.tile();
        let tiles_per_row = specs.cube_maze_sides.tiles_per_row();

        const PALETTE: [ARGB; 8] = [
            0xFF3352E1, // Blue
            0xFF30B06E, // Green
            0xFFDE4949, // Red
            0xFFFFB937, // Yellow
            0xFF533354, // Purple
            0xFF5A7D8B, // Cyan/Grey
            0xFFEEEEEE, // White
            0xFF222222, // Black
        ];

        let top: TileSprite = 0;
        let right: TileSprite = tiles_per_row;
        let left: TileSprite = tiles_per_row * 2;
        
        let cube_corner_xy = unscaled::XY {
            x: unscaled::X(command::WIDTH_SIGNED / 3),
            y: unscaled::Y(command::HEIGHT_SIGNED / 2),
        };

        macro_rules! draw_side {
            ($sprite: expr, $xy: expr, $colour: expr $(,)?) => {
                commands.sspr_override(
                    specs.cube_maze_sides.xy_from_tile_sprite($sprite),
                    specs.cube_maze_sides.rect($xy),
                    $colour,
                );
            }
        }

        // TODO define a face::XY instead of using QRS,
        // and convert to the given face in a special
        // way in each of three loops
        /*
        for (qrs, tile) in [
            (QRS{ q: qrs::Q(0), r: qrs::R(0) }, Tile { kind: TileKind::Wall }),
            (QRS{ q: qrs::Q(1), r: qrs::R(0) }, Tile { kind: TileKind::Blank }),
            (QRS{ q: qrs::Q(0), r: qrs::R(1) }, Tile { kind: TileKind::Goal(GoalFrame::Three) }),
            (QRS{ q: qrs::Q(1), r: qrs::R(1) }, Tile { kind: TileKind::Wall }),
        ] {
            let xy = qrs_to_unscaled(qrs);

            draw_side!(
                match tile.kind {
                    TileKind::Wall => top,
                    TileKind::Blank => continue,
                    TileKind::Goal(GoalFrame::Zero) => continue,
                    TileKind::Goal(frame) => top + frame.index(),
                },
                cube_corner_xy - tile_wh.h,
                PALETTE[6],
            );
        }
        */

        draw_side!(
            top,
            cube_corner_xy - tile_wh.h,
            PALETTE[6],
        );
        draw_side!(
            left,
            cube_corner_xy - tile_wh.w,
            PALETTE[0],
        );
        draw_side!(
            right,
            cube_corner_xy,
            PALETTE[1],
        );
    }
}
