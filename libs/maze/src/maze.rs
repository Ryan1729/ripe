use dir::{Dir, DirFlag};
use vec1::{Grid1, Grid1Spec, vec1};
use xs::Xs;

// TODO cleanup/merge these types

pub type Index = usize;
pub type TileIndex = usize;
pub type ProtoIndex = usize;

pub type TilesWidthInner = u16;
pub type TilesWidth = std::num::NonZeroU16;

#[derive(Clone, Copy)]
pub struct ProtoTilesIndex(Index);

#[derive(Clone, Copy, Debug)]
pub struct ProtoTilesWidth(TilesWidth);

impl ProtoTilesWidth {
    #[cfg(test)]
    #[allow(unused)]
    fn new(inner: TilesWidthInner) -> Option<Self> {
        TilesWidth::new(inner).map(Self)
    }

    fn get(&self) -> TilesWidthInner {
        self.0.get()
    }
}

impl From<ProtoTilesWidth> for TilesWidth {
    fn from(ProtoTilesWidth(width): ProtoTilesWidth) -> Self {
        width
    }
}

impl From<ProtoTilesWidth> for usize {
    fn from(ProtoTilesWidth(width): ProtoTilesWidth) -> Self {
        width.get().into()
    }
}

pub type ProtoTileFlags = u8;

pub const ALL_DIRS: ProtoTileFlags = {
    let mut flags: ProtoTileFlags = 0;

    let mut i = 0;
    while i < Dir::ALL.len() {
        flags |= Dir::ALL[i].flag();
        i += 1;
    }

    flags
};

/// A flag that is outside the range of the Dir flags, which is meant to indicate that the given cell
/// should not be filled at all.
pub const SKIP: ProtoTileFlags = 1 << (Dir::ALL.len());

fn via_backtracking(
    rng: &mut Xs,
    proto_tiles: &mut [ProtoTileFlags],
    width: ProtoTilesWidth,
) {
    via_backtracking_helper(rng, proto_tiles, width, <_>::default())
}

fn via_backtracking_helper(
    rng: &mut Xs,
    proto_tiles: &mut [ProtoTileFlags],
    width: ProtoTilesWidth,
    current_xy: XY,
) {
    let mut dirs = Dir::ALL;
    xs::shuffle(rng, &mut dirs);

    for dir in dirs {
        let option: Option<XY> = current_xy.checked_push(dir);
        if let Some(new_xy) = option {
            let pair: (Option<Index>, Option<Index>) = (
                xy_to_i(width, current_xy).ok(),
                xy_to_i(width, new_xy).ok(),
            );

            if let (Some(current_index), Some(new_index)) = pair
            {
                if let Ok([flags, adjacent_flags])
                    = proto_tiles.get_disjoint_mut([current_index, new_index])
                {
                    // Don't revisit previously visited spots
                    if *adjacent_flags != 0 { continue }

                    *flags |= dir.flag();
                    *adjacent_flags |= dir.opposite().flag();
                    via_backtracking_helper(rng, proto_tiles, width, new_xy);
                }
            }
        }
    }
}


type XYInner = u16;
type X = XYInner;
type Y = XYInner;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct XY {
    x: X,
    y: Y,
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

#[derive(Clone, Copy, Debug)]
pub enum XYToIError {
    XPastWidth
}

fn xy_to_i(width: impl Into<usize>, xy: XY) -> Result<usize, XYToIError> {
    let width_usize = width.into();

    let x_usize = usize::from(xy.x);
    if x_usize >= width_usize {
        return Err(XYToIError::XPastWidth);
    }

    Ok(usize::from(xy.y) * width_usize + x_usize)
}

#[cfg(false)]
#[allow(unused)]
fn print_proto_tiles(
    tiles: &[ProtoTileFlags],
    ProtoTilesWidth(width): ProtoTilesWidth,
) {
    let mut output = String::with_capacity(tiles.len());

    output.push(' ');
    for _ in 0..(width.get() * 2 - 1) {
        output.push('_');
    }
    output.push('\n');

    let height = calc_height(width, tiles);

    for y in 0..height {
        output.push('|');
        for x in 0..width.get() {
            let xy = XY { x: (x), y: (y) };

            let Ok(i) = xy_to_i(width, xy) else { continue };

            let tile = tiles[i];

            output.push(if tile & Dir::Down.flag() != 0 { ' ' } else { '_' });

            if tile & Dir::Right.flag() != 0 {
                output.push(
                    if (tile | tiles.get(i + 1).cloned().unwrap_or(0)) & Dir::Down.flag() != 0 {
                        ' '
                    } else {
                        '_'
                    }
                );
            } else {
                output.push('|');
            }
        }

        output.push('\n');
    }

    eprintln!("{output}");
}

// TODO put back once the external things that use this are pulled into this crate
//#[cfg(test)]
pub mod via_backtracking_connects_all_cells_on {
    use dir::Dir;
    use super::*;
    
    pub(crate) fn are_all_cells_connected_options(
        proto_tiles: &[ProtoTileFlags],
        width: ProtoTilesWidth,
        skip_mask: ProtoTileFlags,
    ) -> bool {
        use std::collections::HashSet;
        let mut seen = HashSet::with_capacity(proto_tiles.len());

        let mut to_see = vec![XY::default()];

        while let Some(xy) = to_see.pop() {
            if let Ok(i) = xy_to_i(width, xy) {
                let tile = proto_tiles[i];

                if tile & skip_mask != 0 { continue }

                // Don't even look at ones that should be skipped.
                seen.insert(i);

                for dir in Dir::ALL {
                    if tile & dir.flag() != 0
                    && let Some(new_xy) = xy.checked_push(dir)
                    && let Ok(new_i) = xy_to_i(width, new_xy)
                    && new_i < proto_tiles.len()
                    && !seen.contains(&new_i) {
                        to_see.push(new_xy);
                    }
                }
            }
        }

        let mut skip_count = 0;

        for i in 0..proto_tiles.len() {
            let tile = proto_tiles[i];

            if tile & skip_mask != 0 { skip_count += 1 }
        }

        seen.len() == (proto_tiles.len() - skip_count)
    }

    pub fn are_all_cells_connected(
        proto_tiles: &[ProtoTileFlags],
        width: impl Into<ProtoTilesWidth>,
    ) -> bool {
        are_all_cells_connected_options(proto_tiles, width.into(), 0)
    }

    // Test predicate test
    #[test]
    fn are_all_cells_connected_returns_false_sometimes() {
        use Dir::*;

        let width = ProtoTilesWidth::new(4).unwrap();

        let rd = Right.flag() | Down.flag();
        let ru = Right.flag() | Up.flag();
        let rl = Right.flag() | Left.flag();
        let ld =  Left.flag() | Down.flag();
        let lu =  Left.flag() | Up.flag();

        // All walls
        let mut tiles = vec1![0; 16usize];

        assert!(!are_all_cells_connected(&mut tiles, width));

        // Top half
        let mut tiles = vec1![
            rd, rl, rl, ld,
            ru, rl, rl, lu,
             0,  0,  0,  0,
             0,  0,  0,  0,
        ];

        assert!(!are_all_cells_connected(&mut tiles, width));

        // Disjoint top and bottom
        let mut tiles = vec1![
            rd, rl, rl, ld,
            ru, rl, rl, lu,

            rd, rl, rl, ld,
            ru, rl, rl, lu,
        ];

        assert!(!are_all_cells_connected(&mut tiles, width));
    }

    #[test]
    fn are_all_cells_connected_options_respects_the_skip_flag() {
        use Dir::*;

        let f = Up.flag() | Down.flag() | Right.flag() | Left.flag();

        let width = ProtoTilesWidth::new(4).unwrap();

        // All floor
        let mut tiles = vec1![f; 16usize];

        assert!(are_all_cells_connected_options(&mut tiles, width, SKIP));

        // Top half
        let mut tiles = vec1![
             f,  f,  f,  f,
             f,  f,  f,  f,
             SKIP,  SKIP,  SKIP,  SKIP,
             SKIP,  SKIP,  SKIP,  SKIP,
        ];

        assert!(are_all_cells_connected_options(&mut tiles, width, SKIP));

        // Disjoint top and bottom
        let mut tiles = vec1![
            f,  f,  f,  f,

            SKIP,  SKIP,  SKIP,  SKIP,
            SKIP,  SKIP,  SKIP,  SKIP,

            f,  f,  f,  f,
        ];

        assert!(!are_all_cells_connected_options(&mut tiles, width, SKIP));
    }

    #[test]
    fn this_small_example() {
        let width = ProtoTilesWidth::new(10).unwrap();
        let mut tiles = vec1![0; 100usize];
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        assert!(!are_all_cells_connected(&mut tiles, width));

        via_backtracking(&mut rng, &mut tiles, width);

        assert!(are_all_cells_connected(&mut tiles, width));
    }
}

#[cfg(test)]
mod via_backtracking_allows_blocking_out_areas_on {
    use super::*;
    use via_backtracking_connects_all_cells_on::{are_all_cells_connected, are_all_cells_connected_options};

    #[test]
    fn this_small_example() {
        let width = ProtoTilesWidth::new(10).unwrap();
        let mut tiles = vec1![0; 100usize];

        for i in 0..tiles.len() {
            if i % usize::from(width.get()) > 5 {
                tiles[i] |= SKIP;
            }
        }

        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        assert!(!are_all_cells_connected(&mut tiles, width));
        assert!(!are_all_cells_connected_options(&mut tiles, width, SKIP));

        via_backtracking(&mut rng, &mut tiles, width);

        assert!(!are_all_cells_connected(&mut tiles, width));
        assert!(are_all_cells_connected_options(&mut tiles, width, SKIP));

        for i in 0..tiles.len() {
            if i % usize::from(width.get()) > 5 {
                // The dir flags should all be 0, still
                assert_eq!(tiles[i], SKIP);
            }
        }
    }
}

//#[cfg(test)]
#[cfg(false)] // We inlined this function
mod generate_with_exit_at_index_generates_reachable_rooms_on {
    use super::*;
    use std::collections::HashSet;

    // Short for assert. We can be this terse because the scope here is limited.
    macro_rules! a {
        ($proto_tiles: expr, $width: expr, $exit_index: expr $(,)?) => ({
            let proto_tiles = $proto_tiles;
            let width = $width;

            fn is_open(flags: ProtoTileFlags) -> bool {
                // 0b1111 are the dir flags.
                // TODO? Add constant for that?
                flags & 0b1111 != 0
            }

            let mut open_tiles_count = 0;
            for &tile in &proto_tiles {
                if is_open(tile) {
                    open_tiles_count += 1;
                }
            }

            fn get_reachable_from(
                proto_tiles: &[ProtoTileFlags],
                width: ProtoTilesWidth,
                start_index: Index,
            ) -> HashSet<Index> {
                use std::collections::HashSet;
                let mut seen = HashSet::with_capacity(proto_tiles.len() / 2 /* was not thought about too hard */);

                let mut to_see = vec![i_to_xy(width, start_index)];

                while let Some(xy) = to_see.pop() {
                    if let Ok(i) = xy_to_i(width, xy)
                    && let Some(&proto_tile) = proto_tiles.get(i) {
                        if !is_open(proto_tile) { continue }

                        seen.insert(i);

                        for dir in Dir::ALL {
                            if let Some(new_xy) = xy.checked_push(dir)
                            && let Ok(new_i) = xy_to_i(width, new_xy)
                            && !seen.contains(&new_i) {
                                to_see.push(new_xy);
                            }
                        }
                    }
                }

                seen
            }

            let seen = get_reachable_from(
                &proto_tiles,
                width,
                $exit_index
            );

            let reachable_from_exit_count = seen.len();

            assert_eq!(reachable_from_exit_count, open_tiles_count);
        })
    }

    #[test]
    fn these_random_examples_in_the_top_of_a_small_vertical_maze() {
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        let proto_width = ProtoTilesWidth(TilesWidth::new(3).unwrap());
        let exit_index = 4; // The center of the top 3 x 3
        // A 3 x 4 room
        let mut proto_tiles;

        for _ in 0..16 {
            proto_tiles = vec1![0; 12usize];

            generate_with_exit_at_index(&mut rng, &mut proto_tiles, proto_width, exit_index);

            a!(
                proto_tiles,
                proto_width,
                exit_index
            );
        }
    }

    #[test]
    fn these_random_examples_in_the_bottom_of_a_small_vertical_maze() {
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        let proto_width = ProtoTilesWidth(TilesWidth::new(3).unwrap());
        let exit_index = 7; // The center of the bottom 3 x 3
        // A 3 x 4 room
        let mut proto_tiles;

        for _ in 0..16 {
            proto_tiles = vec1![0; 12usize];

            generate_with_exit_at_index(&mut rng, &mut proto_tiles, proto_width, exit_index);

            a!(
                proto_tiles,
                proto_width,
                exit_index
            );
        }
    }

    #[test]
    fn these_random_examples_in_the_left_of_a_small_horizontal_maze() {
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        let proto_width = ProtoTilesWidth(TilesWidth::new(4).unwrap());
        let exit_index = 5; // The center of the left 3 x 3
        // A 4 x 3 room
        let mut proto_tiles;

        for _ in 0..16 {
            proto_tiles = vec1![0; 12usize];

            generate_with_exit_at_index(&mut rng, &mut proto_tiles, proto_width, exit_index);

            a!(
                proto_tiles,
                proto_width,
                exit_index
            );
        }
    }

    #[test]
    fn these_random_examples_in_the_right_of_a_small_horizontal_maze() {
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        let proto_width = ProtoTilesWidth(TilesWidth::new(4).unwrap());
        let exit_index = 6; // The center of the right 3 x 3
        // A 4 x 3 room
        let mut proto_tiles;

        for _ in 0..16 {
            proto_tiles = vec1![0; 12usize];

            generate_with_exit_at_index(&mut rng, &mut proto_tiles, proto_width, exit_index);

            a!(
                proto_tiles,
                proto_width,
                exit_index
            );
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tile {
    #[default]
    Wall,
    Floor
}

pub type Tiles = Grid1<Tile, TilesWidth>;

pub struct Generated {
    pub tiles: Tiles,
    pub exit_index: usize,
    pub exit_facing: Dir,
}

pub type Flags = u8;

pub const EXIT_STAIRS: Flags = 1;

// TODO? tighten these types to those that allow always correct generation, or at least good fallbacks.
type MazeWidth = u16;
type MazeHeight = u16;

pub fn generate_fallback(
    (w, h): (MazeWidth, MazeHeight),
) -> Generated {    
    let sizes = Sizes::new(w, h);

    let proto_tiles = vec1![ALL_DIRS; sizes.proto_length];

    let tiles = to_one_thick(&proto_tiles, &sizes);

    Generated {
        tiles,
        // Default to the first non-edge tile
        exit_index: w as usize + 2,
        exit_facing: <_>::default(),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GenerationError {
    NonEdge(NonEdgeError),
    NoExitFacing,
    NoExitIndex,
}

impl From<NonEdgeError> for GenerationError {
    fn from(e: NonEdgeError) -> Self {
        Self::NonEdge(e)
    }
}

pub fn generate(
    rng: &mut Xs,
    (w, h): (MazeWidth, MazeHeight),
    flags: Flags,
) -> Result<Generated, GenerationError> {
    // TODO? assert/return error for (w, h) where there are no non-edge proto tiles?
    let sizes = Sizes::new(w, h);

    let mut proto_tiles = vec1![0; sizes.proto_length];

    let proto_width = sizes.proto_width;

    //
    // Place the exit first
    //

    // Multiple things in the generation function rely on the starting exit_index being an non-edge tile!
    let exit_index = random::non_edge_index(
        Grid1Spec { width: sizes.proto_width.0, len: proto_tiles.len() },
        rng
    )?;

    if !random::is_non_edge_index(
        Grid1Spec::<TilesWidth> { width: proto_width.into(), len: proto_tiles.len() },
        exit_index
    ) {
        return Err(GenerationError::NonEdge(NonEdgeError::BedGeneration))
    }

    let exit_xy = i_to_xy(proto_width, exit_index);

    let exit_facing;

    if flags & EXIT_STAIRS != 0 {
        exit_facing = 'exit_facing: {
            let height = calc_height(proto_width.into(), proto_tiles.slice_mut());

            let mut available_dirs = [
                if exit_xy.y >= (2) { Some(Dir::Up) } else { None },
                if exit_xy.y < (height.saturating_sub(2).into()) { Some(Dir::Down) } else { None },
                if exit_xy.x >= (2) { Some(Dir::Left) } else { None },
                if exit_xy.x < (proto_width.get().saturating_sub(2).into()) { Some(Dir::Right) } else { None },
            ];
    
            xs::shuffle(rng, &mut available_dirs);
    
            for dir_opt in available_dirs {
                if let Some(dir) = dir_opt {
                    break 'exit_facing dir;
                }
            }
    
            return Err(GenerationError::NoExitFacing)
        };


        let (exit_hallway_index, fix_flags) = set_flags_for_exit_stairs(
            proto_tiles.slice_mut(),
            proto_width,
            exit_index,
            exit_facing
        );
    
        //
        // Generate the maze in the area we didn't block out
        //
    
        via_backtracking(rng, proto_tiles.slice_mut(), proto_width);
    
        //
        // Hook up the maze to the blocked out exit
        //
    
        proto_tiles[exit_hallway_index] |= fix_flags;
    } else {
        exit_facing = Dir::ALL[0];

        set_flags_for_simple_exit(
            proto_tiles.slice_mut(),
            proto_width,
            exit_index,
            exit_facing
        );

        via_backtracking(rng, proto_tiles.slice_mut(), proto_width);
    }

    let exit_index = proto_i_to_tile_i(&sizes, ProtoTilesIndex(exit_index))
        .ok_or_else(|| GenerationError::NoExitIndex)?;

    let tiles = to_one_thick(&proto_tiles, &sizes);

    Ok(Generated {
        tiles,
        exit_index,
        exit_facing,
    })
}

#[cfg(test)]
mod generate_places_the_edges_properly_on {
    use super::*;

    #[test]
    fn this_example() {
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        let w = 11;

        let generated = generate(
            &mut rng,
            (w, w),
            0,
        );

        print_tiles(&generated.tiles.cells, w.try_into().unwrap());

        for x in 1..(w - 1) {
            let mut floor_in_column = false;
            
            for y in 1..(w - 1) {
                let tile_xy = XY { x, y };

                let i = xy_to_i(w, tile_xy).unwrap();

                floor_in_column |= generated.tiles.cells[i] == Tile::Floor;
            }

            assert!(floor_in_column, "col {x} has no floor");
        }
    }
}

fn i_to_xy(width: impl Into<TilesWidth>, index: usize) -> XY {
    let width = width.into();
    XY {
        x: ((index % usize::from(width.get())) as _),
        y: ((index / usize::from(width.get())) as _),
    }
}

fn proto_i_to_tile_i(sizes: &Sizes, proto_index: ProtoTilesIndex) -> Option<TileIndex> {
    let proto_xy = i_to_xy(sizes.proto_width.0, proto_index.0);
    
    let tile_xy = XY { x: proto_xy.x * 2 + 1, y: proto_xy.y * 2 + 1 };

    xy_to_i(sizes.tiles_width.get(), tile_xy).ok()
}

fn set_flags_for_simple_exit(
    proto_tiles: &mut [ProtoTileFlags],
    _proto_width: ProtoTilesWidth,
    exit_index: Index,
    exit_facing: Dir
) {
    let u = Dir::Up.flag();
    let d = Dir::Down.flag();
    let l = Dir::Left.flag();
    let r = Dir::Right.flag();

    proto_tiles[exit_index] = SKIP;

    let flag = exit_facing.flag();

    match exit_facing {
        Dir::Up
        | Dir::Down => {
            proto_tiles[exit_index] |= r | l | flag;
        },
        Dir::Left
        | Dir::Right => {
            proto_tiles[exit_index] |= u | d | flag;
        },
    }
}

/// Relies on the exit_index being an non-edge tile!
fn set_flags_for_exit_stairs(
    proto_tiles: &mut [ProtoTileFlags],
    proto_width: ProtoTilesWidth,
    exit_index: Index,
    exit_facing: Dir
) -> (Index, DirFlag) {
    let ProtoTilesWidth(width) = proto_width;
    let width_usize = usize::from(width.get());

    let u = Dir::Up.flag();
    let d = Dir::Down.flag();
    let l = Dir::Left.flag();
    let r = Dir::Right.flag();

    proto_tiles[exit_index - width_usize - 1] = SKIP;
    proto_tiles[exit_index - width_usize] = SKIP;
    proto_tiles[exit_index - width_usize + 1] = SKIP;
    proto_tiles[exit_index - 1] = SKIP;
    proto_tiles[exit_index] = SKIP;
    proto_tiles[exit_index + 1] = SKIP;
    proto_tiles[exit_index + width_usize - 1] = SKIP;
    proto_tiles[exit_index + width_usize] = SKIP;
    proto_tiles[exit_index + width_usize + 1] = SKIP;

    let flag = exit_facing.flag();
    let opposite_flag = exit_facing.opposite().flag();

    let (exit_hallway_index, fix_flags) = match exit_facing {
        Dir::Up
        | Dir::Down => {
            proto_tiles[exit_index - 1] |= r | flag;
            proto_tiles[exit_index] |= r | l | flag;
            proto_tiles[exit_index + 1] |= l | flag;

            let exit_hallway_index = if exit_facing == Dir::Up {
                exit_index - width_usize
            } else {
                exit_index + width_usize
            };

            proto_tiles[exit_hallway_index - 1] |= r | opposite_flag;
            // Clear flags so the maze reaches here
            proto_tiles[exit_hallway_index] = 0;
            proto_tiles[exit_hallway_index + 1] |= l | opposite_flag;

            (exit_hallway_index, r | l | opposite_flag)
        },
        Dir::Left
        | Dir::Right => {
            proto_tiles[exit_index - width_usize] |= d | flag;
            proto_tiles[exit_index] |= u | d | flag;
            proto_tiles[exit_index + width_usize] |= u | flag;

            let exit_hallway_index = if exit_facing == Dir::Left {
                exit_index - 1
            } else {
                exit_index + 1
            };

            proto_tiles[exit_hallway_index - width_usize] |= d | opposite_flag;
            // Clear flags so the maze reaches here
            proto_tiles[exit_hallway_index] = 0;
            proto_tiles[exit_hallway_index + width_usize] |= u | opposite_flag;

            (exit_hallway_index, u | d | opposite_flag)
        },
    };

    (exit_hallway_index, fix_flags)
}

#[cfg(test)]
mod set_flags_for_exit_stairs_produces_the_exact_result_on {
    use super::*;

    const U: DirFlag = Dir::Up.flag();
    const D: DirFlag = Dir::Down.flag();
    const L: DirFlag = Dir::Left.flag();
    const R: DirFlag = Dir::Right.flag();
    const S: DirFlag = SKIP;

    // Short for assert. We can be this terse because the scope here is limited.
    macro_rules! a {
        ($actual: expr, $expected: expr, $width: expr $(,)?) => {
            let actual = $actual;
            let expected = $expected;

            if actual != expected {
                let width = $width;
                let width_usize = usize::from(width.get());
                println!("actual:");

                for i in 0..actual.len() {
                    print!(" {:#04X}", actual[i]);
                    if i % width_usize == width_usize - 1 { println!(); }
                }
                println!();

                println!("expected:");

                for i in 0..expected.len() {
                    print!(" {:#04X}", expected[i]);
                    if i % width_usize == width_usize - 1 { println!(); }
                }
                println!();
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn the_minimal_up_case() {
        let proto_width = ProtoTilesWidth(TilesWidth::new(3).unwrap());
        let mut proto_tiles = vec1![0; 9usize];
        let exit_index = 4; // The center of the 3 x 3

        set_flags_for_exit_stairs(&mut proto_tiles, proto_width, exit_index, Dir::Up);

        a!(
            proto_tiles,
            // One cell intentionally leaves out the flags
            // to give a place to hook the maze onto.
            vec1![
                S | R | D,             0, S | L | D,
                S | R | U, S | L | R | U, S | L | U,
                        S,             S,         S,
            ],
            proto_width,
        );
    }

    #[test]
    fn the_minimal_down_case() {
        let proto_width = ProtoTilesWidth(TilesWidth::new(3).unwrap());
        let mut proto_tiles = vec1![0; 9usize];
        let exit_index = 4; // The center of the 3 x 3

        set_flags_for_exit_stairs(&mut proto_tiles, proto_width, exit_index, Dir::Down);

        a!(
            proto_tiles,
            // One cell intentionally leaves out the S
            // to give a place to hook the maze onto.
            vec1![
                        S,             S,         S,
                S | R | D, S | L | R | D, S | L | D,
                S | R | U,             0, S | L | U,
            ],
            proto_width,
        );
    }

    #[test]
    fn the_minimal_left_case() {
        let proto_width = ProtoTilesWidth(TilesWidth::new(3).unwrap());
        let mut proto_tiles = vec1![0; 9usize];
        let exit_index = 4; // The center of the 3 x 3

        set_flags_for_exit_stairs(&mut proto_tiles, proto_width, exit_index, Dir::Left);

        a!(
            proto_tiles,
            // One cell intentionally leaves out the S
            // to give a place to hook the maze onto.
            vec1![
                    S | R | D,     S | L | D, S,
                            0, S | L | U | D, S,
                    S | R | U,     S | L | U, S,
            ],
            proto_width,
        );
    }

    #[test]
    fn the_minimal_right_case() {
        let proto_width = ProtoTilesWidth(TilesWidth::new(3).unwrap());
        let mut proto_tiles = vec1![0; 9usize];
        let exit_index = 4; // The center of the 3 x 3

        set_flags_for_exit_stairs(&mut proto_tiles, proto_width, exit_index, Dir::Right);

        a!(
            proto_tiles,
            // One cell intentionally leaves out the S
            // to give a place to hook the maze onto.
            vec1![
                        S,     S | R | D, S | L | D,
                        S, S | R | U | D,         0,
                        S,     S | R | U, S | L | U,
            ],
            proto_width,
        );
    }
}

/// Convert the tiles to 1-thick walls
fn to_one_thick(
    proto_tiles: &[ProtoTileFlags],
    sizes: &Sizes
) -> Grid1<Tile, TilesWidth> {
    use Tile::*;

    const F: Tile = Floor;

    let mut tiles = vec1![Tile::default(); sizes.tiles_length];

    for proto_i in 0..proto_tiles.len() {
        let proto_tile_flags = proto_tiles[proto_i];

        if proto_tile_flags != 0 {
            // The cell is open on at least one side.

            if let Some(tile_i) = proto_i_to_tile_i(sizes, ProtoTilesIndex(proto_i)) {
                tiles[tile_i] = F;

                if proto_tile_flags & Dir::Right.flag() != 0 {
                    tiles[tile_i + 1] = F;
                }
    
                if proto_tile_flags & Dir::Down.flag() != 0 {
                    tiles[tile_i + usize::from(sizes.tiles_width.get())] = F;
                }
            }
        }
    }

    Grid1{
        width: sizes.tiles_width,
        cells: tiles,
    }
}

pub type TilesLength = usize;

#[derive(Clone, Copy, Debug)]
pub struct Sizes {
    pub tiles_width: TilesWidth,
    pub tiles_length: TilesLength,
    pub proto_width: ProtoTilesWidth,
    pub proto_length: TilesLength,
}

impl Sizes {
    pub fn new(w: u16, h: u16) -> Self {
        let tiles_length = (w * h).into();

        let proto_width = ProtoTilesWidth(TilesWidth::new((w - 1) / 2).unwrap_or(TilesWidth::MIN));
        let proto_height = TilesWidth::new((h - 1) / 2).unwrap_or(TilesWidth::MIN);
        let proto_length = usize::from(proto_width.get()) * usize::from(proto_height.get());

        let tiles_width = TilesWidth::new(w).unwrap_or(TilesWidth::MIN);

        Sizes {
            tiles_width,
            tiles_length,
            proto_width,
            proto_length,
        }
    }
}

#[cfg(test)]
mod sizes_new_works_on {
    use super::*;

    #[test]
    fn these_examples() {
        macro_rules! a {
            ($tile_size: expr => $expected_proto_size: expr) => ({
                let expected_proto_size = $expected_proto_size;

                let sizes = Sizes::new($tile_size, $tile_size);
                assert_eq!(expected_proto_size as i128, sizes.proto_width.0.get() as i128, "{} => {}", $tile_size, $expected_proto_size);
                assert_eq!((expected_proto_size * expected_proto_size) as i128, sizes.proto_length as i128, "{} => {}", $tile_size, $expected_proto_size);
            })
        }
        // #: always wall
        // .: proto-tile cell
        // _: possible connecting hallway
        
        // # # #
        // # . #
        // # # #
        a!(3 => 1);
        // # # # # #
        // # . _ . #
        // # _ # _ #
        // # . _ . #
        // # # # # #
        a!(5 => 2);
        // # # # # # # #
        // # . _ . _ . #
        // ... omitting for brevity
        // # # # # # # #
        a!(7 => 3);
        // # # # # # # # # #
        // # . _ . _ . _ . #
        // ... omitting for brevity
        // # # # # # # # # #
        a!(9 => 4);
        // # # # # # # # # # # #
        // # . _ . _ . _ . _ . #
        // ... omitting for brevity
        // # # # # # # # # # # #
        a!(11 => 5);
    }
}

mod random {
    use super::*;
    use std::num::TryFromIntError;

    #[derive(Clone, Copy, Debug)]
    pub enum NonEdgeError {
        WidthTooSmall,
        TilesTooShort,
        XYToI(XYToIError),
        TryFromInt(TryFromIntError),
        BedGeneration,
    }

    impl From<XYToIError> for NonEdgeError {
        fn from(e: XYToIError) -> Self {
            NonEdgeError::XYToI(e)
        }
    }

    impl From<TryFromIntError> for NonEdgeError {
        fn from(e: TryFromIntError) -> Self {
            NonEdgeError::TryFromInt(e)
        }
    }

    #[derive(Clone, Copy)]
    pub struct XYXY {
        pub min: XY,
        pub one_past_max: XY,
    }

    impl XYXY {
        pub fn contains(self, xy: XY) -> bool {
            xy.x >= self.min.x
            && xy.y >= self.min.y
            && xy.x < self.one_past_max.x
            && xy.y < self.one_past_max.y
        }
    }

    pub fn non_edge_rect(Grid1Spec { width, len }: Grid1Spec<TilesWidth>) -> Result<XYXY, NonEdgeError> {
        if width.get() < 3 {
            return Err(NonEdgeError::WidthTooSmall);
        }

        // The min/max non-edge corners; The corners of the rectangle of non-edge pieces.
        let min_corner_xy = XY { x: 1, y: 1 };
        let height = Y::try_from(len)? / width.get();
        if height < 3 {
            return Err(NonEdgeError::TilesTooShort);
        }

        let max_corner_xy = XY { x: (width.get() - 1), y: (height - 1) };

        Ok(XYXY{min: min_corner_xy, one_past_max: max_corner_xy})
    }

    // Written for an assert
    #[allow(unused)]
    pub fn is_non_edge_index(spec: Grid1Spec<TilesWidth>, index_to_check: Index) -> bool {
        let xy = i_to_xy(spec.width, index_to_check);

        non_edge_rect(spec)
            .map(|xyxy| xyxy.contains(xy))
            .unwrap_or(false)
    }

    pub fn non_edge_index(spec: Grid1Spec<TilesWidth>, rng: &mut Xs) -> Result<Index, NonEdgeError> {
        let width = spec.width;
        let XYXY { min, one_past_max } = non_edge_rect(spec)?;

        let selected_xy = XY {
            x: (xs::range(rng, u32::from(min.x)..u32::from(one_past_max.x)) as XYInner),
            y: (xs::range(rng, u32::from(min.y)..u32::from(one_past_max.y)) as XYInner)
        };

        {
            let min_corner_index = xy_to_i(width.get(), min)?;
            let max_corner_index = xy_to_i(width.get(), one_past_max)?;
    
            if max_corner_index < min_corner_index {
                return Err(NonEdgeError::TilesTooShort);
            }
        }

        Ok(xy_to_i(width.get(), selected_xy)?)
    }
}
use random::NonEdgeError;

fn calc_height<A>(
    width: TilesWidth,
    tiles: &[A],
) -> XYInner {
    calc_height_len(Grid1Spec { width, len: tiles.len() })
}

fn calc_height_len(
    Grid1Spec { width, len }: Grid1Spec<TilesWidth>
) -> XYInner {
    XYInner::try_from(len).map(|len| len / width.get()).unwrap_or(XYInner::MAX)
}

#[allow(unused)]
fn print_tiles(
    tiles: &[Tile],
    width: TilesWidth,
) {
    let mut output = String::with_capacity(tiles.len());

    let height = calc_height(width, tiles);

    let space_count = 3;

    for y in 0..height {
        for x in 0..width.get() {
            let xy = XY { x, y };

            let Ok(i) = xy_to_i(width.get(), xy) else { continue };

            let tile = tiles[i];

            if let Tile::Wall = tile {
                // default (space_count = n)
                let ch = '#';
                for _ in 0..space_count {
                    output.push(ch);
                }

                // decimal digits (space_count = 3)
                //let hundreds = index as u32/100;
                //let tens = (index as u32 - hundreds * 100)/10;
                //let ones = (index as u32 - hundreds * 100 - tens * 10);
                //output.push(char::from_digit(hundreds, 10).unwrap_or('?'));
                //output.push(char::from_digit(tens, 10).unwrap_or('?'));
                //output.push(char::from_digit(ones, 10).unwrap_or('?'));

                // Braille (space_count = 1)
                //output.push(char::from_u32(0x2800 + index as u32).unwrap_or('?'));
            } else {
                let ch = ' ';

                for _ in 0..space_count {
                    output.push(ch);
                }
            }
        }

        output.push('\n');
    }

    eprintln!("{output}");
}

#[cfg(test)]
mod to_one_thick_connects_all_cells_on {
    use super::*;
    use via_backtracking_connects_all_cells_on::are_all_cells_connected as are_all_proto_cells_connected;

    fn are_all_one_floor_tiles_connected(
        tiles: &[Tile],
        width: TilesWidth
    ) -> bool {
        print_tiles(tiles, width);

        let mut expected = 0;

        let mut start_floor_i = None;

        for i in 0..tiles.len() {
            if tiles[i] == Tile::Floor {
                expected += 1;

                if start_floor_i.is_none() {
                    start_floor_i = Some(i);
                }
            }
        }

        if expected == 0 {
            return true
        }

        let start_floor_i = start_floor_i.unwrap();

        use std::collections::HashSet;
        let mut seen = HashSet::with_capacity(tiles.len() / 2 /* was not thought about too hard */);

        let mut to_see = vec![i_to_xy(width, start_floor_i)];

        while let Some(xy) = to_see.pop() {
            if let Ok(i) = xy_to_i(width.get(), xy) {
                let tile = tiles[i];

                if tile != Tile::Floor { continue }

                seen.insert(i);

                for dir in Dir::ALL {
                    if let Some(new_xy) = xy.checked_push(dir)
                    && let Ok(new_i) = xy_to_i(width.get(), new_xy)
                    && !seen.contains(&new_i) {
                        to_see.push(new_xy);
                    }
                }
            }
        }

        seen.len() == expected
    }

    #[test]
    fn this_generated_example() {
        let sizes = Sizes::new(8, 8);

        let mut proto_tiles = vec1![0; sizes.proto_length];
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        assert!(!are_all_proto_cells_connected(proto_tiles.slice(), sizes.proto_width));

        via_backtracking(&mut rng, &mut proto_tiles, sizes.proto_width);

        assert!(are_all_proto_cells_connected(proto_tiles.slice(), sizes.proto_width));

        let tiles = to_one_thick(
            &proto_tiles,
            &sizes,
        );

        let slice = tiles.slice();

        assert!(are_all_one_floor_tiles_connected(slice.0, slice.1));
    }

    #[test]
    fn this_larger_non_square_example() {
        let sizes = Sizes::new(30, 20);

        let mut proto_tiles = vec1![0; sizes.proto_length];
        let mut rng = xs::from_seed([
            0x0, 0x1, 0x2, 0x3,
            0x4, 0x5, 0x6, 0x7,
            0x8, 0x9, 0xA, 0xB,
            0xC, 0xD, 0xE, 0xF,
        ]);

        
        assert!(!are_all_proto_cells_connected(proto_tiles.slice(), sizes.proto_width));

        via_backtracking(&mut rng, &mut proto_tiles, sizes.proto_width);

        assert!(are_all_proto_cells_connected(proto_tiles.slice(), sizes.proto_width));

        let tiles = to_one_thick(
            &proto_tiles,
            &sizes,
        );

        let slice = tiles.slice();

        assert!(are_all_one_floor_tiles_connected(slice.0, slice.1));
    }
}