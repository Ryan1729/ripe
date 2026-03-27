use vec1::{vec1, Vec1};

type Index = usize;
type TileCount = usize;

/// Used for a version of https://en.wikipedia.org/wiki/A*_search_algorithm
#[derive(Clone, Copy, Debug)]
struct DijkstrasTileData {
    previous_index: Index,
    shortest_distance: TileCount,
}

impl Default for DijkstrasTileData {
    fn default() -> Self {
        Self {
            previous_index: Index::max_value(),
            shortest_distance: TileCount::max_value(),
        }
    }
}

trait XYTrait<Direction: Clone + Copy> : PartialEq + Sized + Clone + Copy {
    fn to_i(self) -> usize;

    fn apply_dir(self, dir: Direction) -> Option<Self>;

    /// The Chebyshev distance for regular (x,y) coords is
    /// max((x2 - x1).abs(), (y2 - y1).abs())
    /// Chebyshev distance works as an A* hueristic on 8 way movement
    /// and 4 way movement, where for example, Manhattan distance
    /// only works on 4 way, and messed things up for 8.
    fn chebyshev_distance_to(self, other: Self) -> usize;
}

pub enum Error {
    Unreachable,
    BadIndex,
    FromEqualsTo,
}

// Returns next xy to go to, to move along the shortest path from `from` to `to`.
pub fn next_xy_along_shortest_path<const TILES_LENGTH: usize, Tile, Direction, XY>(
    tiles: &[Tile],
    all_dirs: &[Direction],
    from: XY,
    to: XY,
    can_pass_through: &dyn Fn(&Tile) -> bool
) -> Result<XY, Error> 
    where XY: XYTrait<Direction>,
        Direction: Clone + Copy
{
    fn find_xy<Direction, XY>(
        came_from: &[XY],
        from: XY,
        mut current: XY,
    ) -> XY
        where XY: XYTrait<Direction>,
            Direction: Clone + Copy {
        let mut current_i = current.to_i();

        while current_i < came_from.len() {
            let xy = came_from[current_i];

            if xy == from {
                // Leave `current` as the one before `to`.
                break
            }

            current = xy;
            current_i = current.to_i();
        }

        current
    }

    match calculate_intermediates::<TILES_LENGTH, Tile, Direction, XY>(
        tiles,
        all_dirs,
        from,
        to,
        can_pass_through,
    ) {
        Ok(Intermediates { came_from, .. }) => {
            let xy: XY = find_xy::<Direction, XY>(&came_from, from, to);
            Ok(xy)
        },
        Err(Error::FromEqualsTo) => Ok(to),
        Err(other_err) => Err(other_err),
    }
}

// Returns path in order from `to` to `from`, likely reverse of what you'd expect.
pub fn shortest_path<const TILES_LENGTH: usize, Tile, Direction, XY>(
    tiles: &[Tile],
    all_dirs: &[Direction],
    from: XY,
    to: XY,
    can_pass_through: &dyn Fn(&Tile) -> bool
) -> Result<Vec1<XY>, Error> 
    where XY: XYTrait<Direction>,
        Direction: Clone + Copy
{
    fn reconstruct_path<const TILES_LENGTH: usize, Direction, XY>(
        came_from: &[XY],
        mut current: XY,
    ) -> Vec1<XY>
        where XY: XYTrait<Direction>,
        Direction: Clone + Copy {
        // A reasonable upper bound is diagonally from one corner of the tile grid to another.
        // If we assume the tile grid is square, that diagonal line is around sqrt(2) times the
        // width (AKA height) of the grid. That width would be around sqrt(TILES_LENGTH) in that
        // case. Don't want to acutally spend the time to calcaute that! If we further assume 
        // that the length is an even power of 2, then sqrt() is the same as shifting down by 
        // half the bits used. For example, 0b1_0000_0000 = 0b1_0000 * 0b1_0000.
        let capacity = TILES_LENGTH >> (TILES_LENGTH.trailing_zeros() / 2);

        let mut output = Vec1::singleton_with_capacity(current, capacity);

        let mut current_i = current.to_i();

        while current_i < came_from.len() {
            current = came_from[current_i];
            output.push(current);
            current_i = current.to_i();
        }

        output
    }

    match calculate_intermediates::<TILES_LENGTH, Tile, Direction, XY>(
        tiles,
        all_dirs,
        from,
        to,
        can_pass_through,
    ) {
        Ok(Intermediates { came_from, .. }) => Ok(reconstruct_path::<TILES_LENGTH, Direction, XY>(&came_from, to)),
        Err(Error::FromEqualsTo) => Ok(vec1![to]),
        Err(other_err) => Err(other_err),
    }
}

struct Intermediates<const TILES_LENGTH: usize, XY> {
    came_from: Vec<XY>,
    shortest_distance: [TileCount; TILES_LENGTH],
    estimated_cost: [TileCount; TILES_LENGTH],
}

fn calculate_intermediates<const TILES_LENGTH: usize, Tile, Direction, XY>(
    tiles: &[Tile],
    all_dirs: &[Direction],
    from: XY,
    to: XY,
    can_pass_through: &dyn Fn(&Tile) -> bool
) -> Result<Intermediates<TILES_LENGTH, XY>, Error> 
    where XY: XYTrait<Direction>,
        Direction: Clone + Copy
{
    use Error::*;

    if from == to {
        return Err(FromEqualsTo);
    }

    let from_i = from.to_i();

    macro_rules! set_result {
        ($arr: ident [$index: expr] = $value: expr) => {
            if let Some(element) = $arr.get_mut($index) {
                *element = $value;
                Ok(())
            } else {
                Err(BadIndex)
            }
        }
    }

    // Just stuffed in here, from another place where it made more cense, without thinking 
    // too hard about it.
    let capacity = TILES_LENGTH >> (TILES_LENGTH.trailing_zeros() / 2);

    let mut next_xys = std::collections::VecDeque::with_capacity(capacity);
    next_xys.push_back(to);

    // For an xy index i, came_from[i] is the xy immediately preceding it on 
    // the shortest path to i currently known.
    let mut came_from: Vec<XY> = Vec::with_capacity(16);

    let mut shortest_distance = [TileCount::max_value(); TILES_LENGTH];
    set_result!( shortest_distance[from_i] = 0 )?;

    // For xy, estimated_cost[xy.to_i()]
    //    = shortest_distance[xy.to_i()] + from.chebyshev_distance_to(xy.to_i());
    let mut estimated_cost = [TileCount::max_value(); TILES_LENGTH];
    set_result!( estimated_cost[from_i] = from.chebyshev_distance_to(from) )?;

    while let Some(current_xy) = next_xys.pop_front() {
        // current_xy has the lowest estimated_cost.
        if current_xy == to {
            return Ok(Intermediates { came_from, shortest_distance, estimated_cost });
        }

        let current_i = current_xy.to_i();

        for &dir in all_dirs.iter() {
            let xy_opt = current_xy.apply_dir(dir);
            let neighbor_xy = match xy_opt {
                Some(new_xy) => new_xy,
                None => continue,
            };

            let neighbor_i = neighbor_xy.to_i();

            if !can_pass_through(&tiles[neighbor_i]) {
                continue
            }

            let tentative_distance = shortest_distance.get(current_i).ok_or(BadIndex)? + 1;

            if tentative_distance < *shortest_distance.get(neighbor_i).ok_or(BadIndex)? {
                // A new shortest distance!
                set_result!{ came_from[neighbor_i] = current_xy }?;
                set_result!{ shortest_distance[neighbor_i] = tentative_distance }?;
                set_result!{ estimated_cost[neighbor_i] = tentative_distance + from.chebyshev_distance_to(neighbor_xy) }?;
                if !next_xys.contains(&neighbor_xy) {
                    next_xys.push_back(neighbor_xy);
                }
            }
        }
    }

    Err(Unreachable)
}