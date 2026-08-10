use gfx::{Commands};
use gfx_sizes::{ARGB};
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use qrs::{QRS, QRSD, Q, R, qr};
use vec1::{Grid1, Grid1Spec};
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
struct Goal {
    xy: face::XY,
    frame: GoalFrame,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
enum TileKind {
    #[default]
    Wall,
    Blank,
}

impl TileKind {
    const ALL: [Self; 2] = [
        Self::Wall,
        Self::Blank,
    ];
}

#[derive(Clone, Default, Debug)]
struct Tile {
    kind: TileKind,
}

mod face {
    use super::*;

    type Width = u8;

    pub type Index = unscaled::Inner;

    pub type X = unscaled::Inner;
    pub type Y = unscaled::Inner;

    #[derive(Clone, Copy, Debug, Default)]
    pub struct XY {
        pub x: X,
        pub y: Y,
    }

    impl XY {
        pub fn checked_push(self, dir: impl Into<Dir>) -> Option<XY> {
            Some(match dir.into() {
                Dir::Up => XY { x: self.x, y: self.y.checked_sub(1)? },
                Dir::Right => XY { x: self.x.checked_add(1)?, y: self.y },
                Dir::Down => XY { x: self.x, y: self.y.checked_add(1)? },
                Dir::Left => XY { x: self.x.checked_sub(1)?, y: self.y },
            })
        }
    }

    pub fn i_to_xy(width: Width, i: Index) -> XY {
        XY {
            x: i % (width as unscaled::Inner),
            y: i / (width as unscaled::Inner),
        }
    }

    #[derive(Debug)]
    pub enum XYToIError {
         XPastWidth
    }

    pub fn xy_to_i(width: Width, xy: XY) -> Result<Index, XYToIError> {    
        let width = width as unscaled::Inner;

        if xy.x >= width {
            return Err(XYToIError::XPastWidth);
        }
    
        let i = xy.y * width + xy.x;

        Ok(i)
    }

    #[derive(Clone, Debug, Default)]
    pub struct Face {
        pub player: XY,
        pub goal: Goal,
        pub tiles: Vec<Tile>,
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Kind {
        Top,
        Left,
        Right,
    }

    impl Kind {
        pub const ALL: [Kind; 3] = [
            Kind::Top,
            Kind::Left,
            Kind::Right,
        ];
    }

    #[derive(Clone, Default, Debug)]
    pub struct Faces {
        pub width: Width,
        pub top: Face,
        pub left: Face,
        pub right: Face,
    }

    fn to_tiles(maze_tiles: &Grid1<maze::Tile, std::num::NonZero<u16>>) -> Vec<Tile> {
        let mut output = Vec::with_capacity(maze_tiles.cells.len());

        for maze_tile in &maze_tiles.cells {
            output.push(Tile {
                kind: match maze_tile {
                    maze::Tile::Wall => TileKind::Wall,
                    maze::Tile::Floor => TileKind::Blank,
                },
            });
        }

        output
    }

    fn random_floor_xy(rng: &mut Xs, tiles: &[Tile], width: Width) -> Option<face::XY> {
        let len = tiles.len();

        let start = xs::range(rng, 0..len as u32) as usize;

        for offset in 0..tiles.len() {
            let i = ((start + offset) % len) as u16;
            if tiles[i as usize].kind == TileKind::Blank {
                return Some(face::i_to_xy(width, i as i16));
            }
        }

        None
    }

    impl Faces {
        pub fn new(rng: &mut Xs) -> Self {
            let width: Width = 7; // looks bad if this isn't an odd number, and it should be >= 7
            let length = width as usize * width as usize;

            let mut faces = [
                Face {
                    player: <_>::default(),
                    goal: <_>::default(),
                    tiles: Vec::with_capacity(length),
                },
                Face {
                    player: <_>::default(),
                    goal: <_>::default(),
                    tiles: Vec::with_capacity(length),
                },
                Face {
                    player: <_>::default(),
                    goal: <_>::default(),
                    tiles: Vec::with_capacity(length),
                },
            ];

            let dimension = width.into();

            let dimensions = (dimension, dimension); // square

            let maze_flags: maze::Flags = 0;

            // generate solvable maze for one random face, including placing the exit.
            let base_generated = maze::generate(
                rng,
                dimensions,
                maze_flags,
            );

            faces[0].goal.xy = i_to_xy(width, base_generated.exit_index.try_into().expect("exit index invalid"));
            faces[0].tiles = to_tiles(&base_generated.tiles);

            let spec = Grid1Spec {
                len: faces[0].tiles.len(),
                width: dimensions.0.into(),
            };

            // define a random non-optimal solution to the top face.
            let mut selected_path: Vec<usize> = Vec::with_capacity(64 /* not thought about too hard */);
            let mut temp_paths = Vec::with_capacity(16 /* not thought about too hard */);

            faces[0].player = random_floor_xy(rng, &faces[0].tiles, width).expect("No starting floor tile found in face 0 maze");
            let mut start_index = face::xy_to_i(width, faces[0].player).expect("Start xy is invalid") as usize;

            // Pick a few random points and pathfind paths across them
            for i in 0..3 {
                temp_paths.clear();
        
                let next_xy = random_floor_xy(rng, &faces[0].tiles, width).expect("No floor tile found in face 0 maze");

                let next_index = face::xy_to_i(width, next_xy).expect("Next xy is invalid") as usize;

                spec.find_all_paths(
                    start_index,
                    next_index,
                    |i| { matches!(faces[0].tiles.get(i).map(|t| t.kind), Some(TileKind::Blank)) },
                    &mut temp_paths,
                );

                let path: Vec<_> = temp_paths.pop().expect("No path in face 0 maze");

                assert_eq!(Some(&start_index), path.first());
                assert_eq!(Some(&next_index), path.last());

                // We want the last element of the output path to be the last next_index, but we don't want duplicates.
                if let Some(previous_end) = selected_path.pop() {
                    start_index = previous_end;
                }
                
                selected_path.extend(&path);
            }

            // for the other faces, generate mazes selecting start area, without selecting exit.
            faces[1].tiles = to_tiles(
                &maze::generate(
                    rng,
                    dimensions,
                    maze_flags,
                ).tiles
            );

            faces[2].tiles = to_tiles(
                &maze::generate(
                    rng,
                    dimensions,
                    maze_flags,
                ).tiles
            );

            faces[1].player = random_floor_xy(rng, &faces[1].tiles, width).expect("No floor tile found in face 1 maze");
            faces[1].goal.xy = faces[1].player;

            faces[2].player = random_floor_xy(rng, &faces[2].tiles, width).expect("No floor tile found in face 2 maze");
            faces[2].goal.xy = faces[2].player;

            let width_isize = width as isize;

            // play the non-optimal solution across the other two faces and mark wherever we end up as the exits
            for window in selected_path.windows(2) {
                let from = window[0] as isize;
                let to = window[1] as isize;

                let delta = from - to;

                let dir;

                if delta == -1 {
                    dir = Dir::Right;
                } else if delta == 1 {
                    dir = Dir::Left;
                } else if delta == width_isize {
                    dir = Dir::Up;
                } else if delta == -width_isize {
                    dir = Dir::Down;
                } else {
                    debug_assert!(false, "Disconnected path");
                    continue
                }

                if let Some(new_xy) = faces[1].goal.xy.checked_push(dir) {
                    if let Ok(new_i) = face::xy_to_i(width, new_xy) {
                        if let Some(TileKind::Blank) = faces[1].tiles.get(new_i as usize).map(|t| t.kind) {
                            faces[1].goal.xy = new_xy;
                        }
                    }
                }

                if let Some(new_xy) = faces[2].goal.xy.checked_push(dir) {
                    if let Ok(new_i) = face::xy_to_i(width, new_xy) {
                        if let Some(TileKind::Blank) = faces[2].tiles.get(new_i as usize).map(|t| t.kind) {
                            faces[2].goal.xy = new_xy;
                        }
                    }
                }
            }

            xs::shuffle(rng, &mut faces);

            let top = std::mem::replace(&mut faces[0], <_>::default());
            let left = std::mem::replace(&mut faces[1], <_>::default());
            let right = std::mem::replace(&mut faces[2], <_>::default());

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
    pub tick_count: u64,
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

        Self {
            seed,
            rng: rng_,
            faces,
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

    fn tick(&mut self) {
        if self.tick_count & 15 == 0 {
            for face_kind in face::Kind::ALL {
                let face = match face_kind {
                    face::Kind::Top => &mut self.faces.top,
                    face::Kind::Left => &mut self.faces.left,
                    face::Kind::Right => &mut self.faces.right,
                };
    
                face.goal.frame = match face.goal.frame {
                    GoalFrame::Zero => GoalFrame::One,
                    GoalFrame::One => GoalFrame::Two,
                    GoalFrame::Two => GoalFrame::Three,
                    GoalFrame::Three => GoalFrame::Four,
                    GoalFrame::Four => GoalFrame::Zero,
                };
            }
        }

        self.tick_count = self.tick_count.wrapping_add(1);
    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        specs: &sprite::Specs,
        input: Input,
        _speaker: &mut Speaker,
    ) {
        //
        // Update
        //

        self.tick();

        //
        // Render
        //

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

        macro_rules! face_xy_to_offset {
            ($xy: expr, $face_kind: expr) => ({
                // (xy: face::XY, kind: face::Kind) -> unscaled::XYD
                let xy: face::XY = $xy;

                match $face_kind {
                    face::Kind::Top => {
                        // "- width_inner" to shift up by the width amount,
                        // making the bottom row's location fixed as the
                        // width changes.
                        unscaled::XYD {
                            xd: unscaled::XD(
                                xy.x * top_x_face_x_factor + (xy.y - width_inner) * top_x_face_y_factor
                            ),
                            yd: unscaled::YD(
                                xy.x * top_y_face_x_factor + (xy.y - width_inner) * top_y_face_y_factor
                            ),
                        }
                    }
                    face::Kind::Left => {
                        // "- width_inner" to shift left by the width amount,
                        // making the bottom row's location fixed as the
                        // width changes.
                        unscaled::XYD {
                            xd: unscaled::XD(
                                (xy.x - width_inner) * left_x_face_x_factor + xy.y * left_x_face_y_factor
                            ),
                            yd: unscaled::YD(
                                (xy.x - width_inner) * left_y_face_x_factor + xy.y * left_y_face_y_factor
                            ),
                        }
                    }
                    face::Kind::Right => {
                        unscaled::XYD {
                            xd: unscaled::XD(
                                xy.x * right_x_face_x_factor + xy.y * right_x_face_y_factor
                            ),
                            yd: unscaled::YD(
                                xy.x * right_y_face_x_factor + xy.y * right_y_face_y_factor
                            ),
                        }
                    }
                }
            })
        }

        for face_kind in face::Kind::ALL {
            let (face, base_tile_sprite, base_offset, wall_colour) = match face_kind {
                face::Kind::Top => (&self.faces.top, top, top_base_offset, PALETTE[6]),
                face::Kind::Left => (&self.faces.left, left, left_base_offset, PALETTE[0]),
                face::Kind::Right => (&self.faces.right, right, right_base_offset, PALETTE[1]),
            };

            let base = cube_corner_xy + base_offset;

            for (i, tile) in face.tiles.iter().enumerate() {
                if i > unscaled::Inner::MAX as usize { break }
                let i = i as unscaled::Inner;

                draw_side!(
                    match tile.kind {
                        TileKind::Wall => base_tile_sprite,
                        TileKind::Blank => continue,
                    },
                    base + face_xy_to_offset!(face::i_to_xy(self.faces.width, i), face_kind),
                    wall_colour,
                );
            }

            match face.goal.frame {
                GoalFrame::Zero => {}
                frame => {
                    draw_side!(
                        base_tile_sprite + frame.index(),
                        base + face_xy_to_offset!(face.goal.xy, face_kind),
                        PALETTE[2],
                    );
                }
            }

            draw_side!(
                base_tile_sprite,
                base + face_xy_to_offset!(face.player, face_kind),
                PALETTE[2],
            );
        }
    }
}
