use gfx::{Commands};
use gfx_sizes::{ARGB};
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use qrs::{QRS, QRSD, Q, R, qr};
use xs::{Seed, Xs};

type TileSprite = u8;

#[derive(Clone, Copy, Default, Debug)]
enum GoalFrame {
    #[default]
    Zero, // Blank
    One,
    Two,
    Three,
    Four,
}

impl GoalFrame {
    fn index(self) -> u8 {
        match self {
            Self::Zero => 0,
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
        }
    }

    const DEFAULT: Self = Self::Zero;
}

#[derive(Clone, Copy, Default, Debug)]
enum TileKind {
    #[default]
    Wall,
    Blank,
    Goal(GoalFrame),
}

impl TileKind {
    const ALL: [Self; 3] = [
        Self::Wall,
        Self::Blank,
        Self::Goal(GoalFrame::DEFAULT),
    ];
}

#[derive(Clone, Default, Debug)]
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

mod face {
    use super::*;

    type Width = u8;
    type Face = Vec<Tile>;

    #[derive(Clone, Default, Debug)]
    pub struct Faces {
        pub width: Width,
        pub top: Face,
        pub left: Face,
        pub right: Face,
    }

    impl Faces {
        pub fn new(rng: &mut Xs) -> Self {
            let width: Width = 3;
            let length = width as usize * width as usize;

            let mut top = Vec::with_capacity(length);
            let mut left = Vec::with_capacity(length);
            let mut right = Vec::with_capacity(length);

            // TODO ensure solvabilty

            for i in 0..length {
                let kind = TileKind::ALL[xs::index(rng, 0..TileKind::ALL.len())];
                top.push(Tile { kind });
                left.push(Tile { kind });
                right.push(Tile { kind });
            }

            Self {
                width,
                top,
                left,
                right,
            }
        }
    }
}
use face::Faces;

#[derive(Clone, Debug, Default)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
    pub faces: Faces,
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

        let faces = Faces::new(rng);
        let mobs = Mobs::default();

        Self {
            seed,
            rng: rng_,
            faces,
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

        let top_base = cube_corner_xy - tile_wh.h;

        for (i, tile) in self.faces.top.iter().enumerate() {
            if i > unscaled::Inner::MAX as usize { break }
            let i = i as unscaled::Inner;

            let offset = unscaled::XYD {
                // TODO scale these
                xd: unscaled::XD(i % (self.faces.width as unscaled::Inner)),
                yd: unscaled::YD(i / (self.faces.width as unscaled::Inner)),
            };

            draw_side!(
                match tile.kind {
                    TileKind::Wall => top,
                    TileKind::Blank => continue,
                    TileKind::Goal(GoalFrame::Zero) => continue,
                    TileKind::Goal(frame) => top + frame.index(),
                },
                top_base + offset,
                PALETTE[6],
            );
        }

        let left_base = cube_corner_xy + tile_wh.h;

        for (i, tile) in self.faces.left.iter().enumerate() {
            if i > unscaled::Inner::MAX as usize { break }
            let i = i as unscaled::Inner;

            let offset = unscaled::XYD {
                // TODO scale these
                xd: unscaled::XD(i % (self.faces.width as unscaled::Inner)),
                yd: unscaled::YD(i / (self.faces.width as unscaled::Inner)),
            };

            draw_side!(
                match tile.kind {
                    TileKind::Wall => left,
                    TileKind::Blank => continue,
                    TileKind::Goal(GoalFrame::Zero) => continue,
                    TileKind::Goal(frame) => left + frame.index(),
                },
                left_base + offset,
                PALETTE[0],
            );
        }

        let right_base = cube_corner_xy;

        for (i, tile) in self.faces.right.iter().enumerate() {
            if i > unscaled::Inner::MAX as usize { break }
            let i = i as unscaled::Inner;

            let offset = unscaled::XYD {
                // TODO scale these
                xd: unscaled::XD(i % (self.faces.width as unscaled::Inner)),
                yd: unscaled::YD(i / (self.faces.width as unscaled::Inner)),
            };

            draw_side!(
                match tile.kind {
                    TileKind::Wall => right,
                    TileKind::Blank => continue,
                    TileKind::Goal(GoalFrame::Zero) => continue,
                    TileKind::Goal(frame) => right + frame.index(),
                },
                right_base + offset,
                PALETTE[1],
            );
        }
    }
}
