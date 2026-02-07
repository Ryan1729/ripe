use platform_types::{sprite, sprite::{Renderable, BaseTiles}, unscaled, TILES_PER_ROW};
use models::{Entity, xy::{XY}, TileSprite, offset};
/// Take a models::XY to the unscaled::XY representing the corner of the tile, with the mininum x/y values.
/// Suitable for drawing the tile at that point
pub fn min_corner(spec: &sprite::Spec<BaseTiles>, xy: XY) -> unscaled::XY {
    let tile = spec.tile();

    let x = unscaled::X(0) + unscaled::W(xy.x.get() * tile.w.get());
    let y = unscaled::Y(0) + unscaled::H(xy.y.get() * tile.h.get());

    unscaled::XY { x, y }
    // This is a BaseTiles specific adjustment to render the tiles at a different spot on the screen
    + tile
}

/// Take a models::XY to the unscaled::XY representing the center of the tile.
pub fn center(spec: &sprite::Spec<BaseTiles>, xy: XY) -> unscaled::XY {
    min_corner(spec, xy) + spec.tile_center_offset()
}

pub fn sprite_xy(spec: &sprite::Spec<BaseTiles>, tile_sprite: TileSprite) -> sprite::XY<Renderable> {
    let tile = spec.tile();
    sprite::XY::<BaseTiles> {
        x: sprite::x(0) + sprite::W(tile_sprite as sprite::Inner % sprite::Inner::from(TILES_PER_ROW)) * tile.w.get(),
        y: sprite::y(0) + sprite::H(tile_sprite as sprite::Inner / sprite::Inner::from(TILES_PER_ROW)) * tile.h.get(),
    }.apply(spec)
}

pub fn entity_rect(spec: &sprite::Spec<BaseTiles>, entity: &Entity) -> unscaled::Rect {
    spec.offset_rect(entity.offset, min_corner(spec, entity.xy))
}