#![deny(unused)]

use platform_types::{sprite, sprite::{BaseTiles}, unscaled};
use models::{Entity, xy::{XY}};
/// Take a models::XY to the unscaled::XY representing the corner of the tile, with the mininum x/y values.
/// Suitable for drawing the tile at that point
pub fn min_corner(spec: &sprite::Spec<BaseTiles>, xy: XY) -> unscaled::XY {
    let tile = spec.tile();

    let x = unscaled::X(0) + unscaled::W::new(i16::try_from(xy.x.get()).expect("model::X too large") * tile.w.get());
    let y = unscaled::Y(0) + unscaled::H::new(i16::try_from(xy.y.get()).expect("model::Y too large") * tile.h.get());

    unscaled::XY { x, y }
    // This is a BaseTiles specific adjustment to render the tiles at a different spot on the screen
    + tile
}

/// Take a models::XY to the unscaled::XY representing the center of the tile.
pub fn center(spec: &sprite::Spec<BaseTiles>, xy: XY) -> unscaled::XY {
    min_corner(spec, xy) + spec.tile_center_offset()
}

pub fn entity_rect(spec: &sprite::Spec<BaseTiles>, entity: &Entity) -> unscaled::Rect {
    spec.offset_rect(entity.offset, min_corner(spec, entity.xy))
}