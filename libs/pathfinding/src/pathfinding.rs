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

trait XYTrait<Direction: Clone + Copy> : PartialEq {
    fn to_i() -> usize;

    fn apply_dir(&self, dir: Dir) -> Option<Self>;

    /// The Chebyshev distance for regular (x,y) coords is
    /// max((x2 - x1).abs(), (y2 - y1).abs())
    /// Chebyshev distance works as an A* hueristic on 8 way movement
    /// and 4 way movement, where for example, Manhattan distance
    /// only works on 4 way, and messed things up for 8.
    fn chebyshev_distance_to(other: Self) -> usize;
}

pub enum Error {
    Unreachable,
    BadIndex
}

// Returns path in order from `to` to `from`, likely reverse of what you'd expect.
fn shortest_path<const TILES_LENGTH: usize, Tile, Direction, XY>(
    tiles: &[Tile],
    all_dirs: &[Direction],
    from: XY,
    to: XY,
    can_pass_through: &dyn Fn(Tile) -> bool
) -> Result<Vec1<XY>, Error> 
    where XY: XYTrait
{
    use Error::*;

    if from == to {
        return Ok(vec1![to]);
    }

    fn reconstruct_path(
        came_from: &[XY],
        current: XY,
    ) -> Vec1<XY> {
        // A reasonable upper bound is diagonally from one corner of the tile grid to another.
        // If we assume the tile grid is square, that diagonal line is around sqrt(2) times the
        // width (AKA height) of the grid. That width would be around sqrt(TILES_LENGTH) in that
        // case. Don't want to acutally spend the time to calcaute that! If we further assume 
        // that the length is an even power of 2, then sqrt() is the same as shifting down by 
        // half the bits used. For example, 0b1_0000_0000 = 0b1_0000 * 0b1_0000.
        let capacity = TILES_LENGTH >> (TILES_LENGTH.trailing_zeroes() / 2);

        let output = Vec1::singleton_with_capacity(current, capacity);

        let mut current_i = current.to_i();

        while current_i < came_from.len() {
            current = came_from[current_i];
            output.push(current);
            current_i = current.to_i();
        }

        output
    }

    let from_i = from.to_i():

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

    let mut next_xys = std::collections::VecDeque::with_capacity(max_x * max_y);
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
            return Ok(reconstruct_path(&came_from, current_xy));
        }

        let current_i = current_xy.to_i();

        for &dir in all_dirs.iter() {
            let xy_opt = current_xy.apply_dir(dir);
            let neighbor_xy = match xy_opt {
                Some(new_xy) => {
                    if new_xy.x > max_xy.x
                    || new_xy.y > max_xy.y
                    || new_xy.x < min_xy.x
                    || new_xy.y < min_xy.y {
                        continue;
                    }
                    new_xy
                },
                None => {
                    continue;
                }
            };

            let tentative_distance = shortest_distance.get(current_i).ok_or(BadIndex)? + 1;

            let neighbor_i = neighbor_xy.to_i();

            if tentative_distance < shortest_distance.get(neighbor_i).ok_or(BadIndex)? {
                // A new shortest distance!
                set_result!{ came_from[neighbor_i] = current_xy }?;
                set_result!{ shortest_distance[neighbor_i] = tentative_distance }?;
                set_result!{ estimated_cost[neighbor_i] = tentative_distance + from.chebyshev_distance_to(neighbor_xy) }?;
                if !next_xys.contains(neighbor_xy) {
                    next_xys.push_back(neighbor_xy);
                }
            }
        }
    }

    Err(Unreachable)
}