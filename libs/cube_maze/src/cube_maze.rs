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
            let width: Width = 11;
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

        // These constants were found for the original art by measurement/trial and error.
        // Then we attemtped to convert them to be based on the tile dimensions, without
        // bothering to check if changing them behaved at all reasonably. Probably the way
        // to address this at such time that we really need to support different graphics
        // sizes is to allow configuring these values as well.

        let top_x_face_x_factor: i16 = tile_wh.h.get() - 2;
        let top_x_face_y_factor: i16 = top_x_face_x_factor;

        let top_y_face_x_factor: i16 = -top_x_face_x_factor/2;
        let top_y_face_y_factor: i16 = top_x_face_x_factor/2;

        let left_x_face_x_factor: i16 = top_x_face_x_factor;
        let left_x_face_y_factor: i16 = 0;

        let left_y_face_x_factor: i16 = top_x_face_x_factor/2;
        let left_y_face_y_factor: i16 = left_y_face_x_factor + (left_y_face_x_factor/2);

        let right_x_face_x_factor: i16 = top_x_face_x_factor;
        let right_x_face_y_factor: i16 = 0;

        let right_y_face_x_factor: i16 = -top_x_face_x_factor/2;
        let right_y_face_y_factor: i16 = left_y_face_x_factor + (left_y_face_x_factor/2);

        // This part makes evident that either some ergonomic additions, or the removal of
        // the W, H, and WH types in favour of the XD, YD, and XYD types, would be an
        // improvement on te status quo. Doesn't currently seem to come up that often though.
        let top_base_offset: unscaled::XYD = unscaled::XYD::default();
        let left_base_offset: unscaled::XYD = unscaled::XYD::default() - unscaled::XYD::from(unscaled::WH::default() + (tile_wh.w / 4) - (tile_wh.h / 2) - unscaled::H::new(1));
        let right_base_offset: unscaled::XYD = unscaled::XYD::default() - unscaled::XYD::from(unscaled::WH::default() + (tile_wh.w / 4) - unscaled::H::new(2));

        let width_inner = self.faces.width as unscaled::Inner;

        let top_base = cube_corner_xy + top_base_offset;

        for (i, tile) in self.faces.top.iter().enumerate() {
            if i > unscaled::Inner::MAX as usize { break }
            let i = i as unscaled::Inner;

            let face_x = (i % width_inner);
            // "- width_inner" to shift up by the width amount,
            // making the bottom row's location fixed as the
            // width changes.
            let face_y = i / width_inner - width_inner;

            let offset = unscaled::XYD {
                xd: unscaled::XD(
                    face_x * top_x_face_x_factor + face_y * top_x_face_y_factor
                ),
                yd: unscaled::YD(
                    face_x * top_y_face_x_factor + face_y * top_y_face_y_factor
                ),
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

        let left_base = cube_corner_xy + left_base_offset;

        for (i, tile) in self.faces.left.iter().enumerate() {
            if i > unscaled::Inner::MAX as usize { break }
            let i = i as unscaled::Inner;

            // "- width_inner" to shift left by the width amount,
            // making the bottom row's location fixed as the
            // width changes.
            let face_x = (i % width_inner) - width_inner;
            let face_y = i / width_inner;

            let offset = unscaled::XYD {
                xd: unscaled::XD(
                    face_x * left_x_face_x_factor + face_y * left_x_face_y_factor
                ),
                yd: unscaled::YD(
                    face_x * left_y_face_x_factor + face_y * left_y_face_y_factor
                ),
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

        let right_base = cube_corner_xy + right_base_offset;

        for (i, tile) in self.faces.right.iter().enumerate() {
            if i > unscaled::Inner::MAX as usize { break }
            let i = i as unscaled::Inner;

            let face_x = i % (self.faces.width as unscaled::Inner);
            let face_y = i / (self.faces.width as unscaled::Inner);

            let offset = unscaled::XYD {
                xd: unscaled::XD(
                    face_x * right_x_face_x_factor + face_y * right_x_face_y_factor
                ),
                yd: unscaled::YD(
                    face_x * right_y_face_x_factor + face_y * right_y_face_y_factor
                ),
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
