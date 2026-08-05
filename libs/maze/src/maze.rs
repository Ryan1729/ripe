use dir::{Dir};
use xs::Xs;

type Index = usize;

pub trait AtTrait<IndexContext, Direction: Clone + Copy> : PartialEq + Sized + Clone + Copy {
    /// The index context might be something like the width of the tile grid, which is useful to calculate the
    /// index given a regular (x,y) coord pair.
    fn to_i(self, context: IndexContext) -> Option<Index>;

    fn apply_dir(self, dir: Direction) -> Option<Self>;
}

pub type ProtoTileFlags = u8;

/// A flag that is outside the range of the Dir flags, which is meant to indicate that the given cell
/// should not be filled at all.
pub const SKIP: ProtoTileFlags = 1 << (Dir::ALL.len());

pub fn via_backtracking<At, IndexContex>(
    proto_tiles: &mut [ProtoTileFlags],
    rng: &mut Xs,
    index_contex: IndexContex,
    current_at: At,
) where
    At: AtTrait<IndexContex, Dir>,
    IndexContex: Copy,
{
    let mut dirs = Dir::ALL;
    xs::shuffle(rng, &mut dirs);

    for dir in dirs {
        let option: Option<At> = current_at.apply_dir(dir);
        if let Some(new_at) = option {
            let pair: (Option<Index>, Option<Index>) = (current_at.to_i(index_contex), new_at.to_i(index_contex));

            if let (Some(current_index), Some(new_index)) = pair
            {
                if let Ok([flags, adjacent_flags])
                    = proto_tiles.get_disjoint_mut([current_index, new_index])
                {
                    // Don't revisit previously visited spots
                    if *adjacent_flags != 0 { continue }

                    *flags |= dir.flag();
                    *adjacent_flags |= dir.opposite().flag();
                    via_backtracking(proto_tiles, rng, index_contex, new_at);
                }
            }
        }
    }
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
            let xy = XY { x: xy::x(x), y: xy::y(y) };

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

    type ProtoTilesWidth = usize;

    type X = u16;
    type Y = u16;

    /// An example XY for these tests
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
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

    #[derive(Debug)]
    pub enum XYToIError {
        XPastWidth
    }

    pub fn xy_to_i(width: impl Into<usize>, xy: XY) -> Result<usize, XYToIError> {
        let width_usize = width.into();
    
        let x_usize = usize::from(xy.x);
        if x_usize >= width_usize {
            return Err(XYToIError::XPastWidth);
        }
    
        Ok(usize::from(xy.y) * width_usize + x_usize)
    }
    
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

        via_backtracking(&mut tiles, &mut rng, width, <_>::default());

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

        via_backtracking(&mut tiles, &mut rng, width, <_>::default());

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
