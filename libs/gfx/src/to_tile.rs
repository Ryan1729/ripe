use platform_types::{sprite, sprite::{Renderable, BaseTiles}, unscaled, TILES_PER_ROW};
use models::{Entity, xy::{XY}, TileSprite, offset};
/// Take a models::XY to the unscaled::XY representing the corner of the tile, with the mininum x/y values.
/// Suitable for drawing the tile at that point
pub fn min_corner(spec: &sprite::Spec<BaseTiles>, xy: XY) -> unscaled::XY {
    let base_offset = spec.tile();

    let x = unscaled::X(xy.x.get() * base_offset.w.get());
    let y = unscaled::Y(xy.y.get() * base_offset.h.get());

    unscaled::XY { x, y } + base_offset
}

/// Take a models::XY to the unscaled::XY representing the center of the tile.
pub fn center(spec: &sprite::Spec<BaseTiles>, xy: XY) -> unscaled::XY {
    min_corner(spec, xy) + spec.tile_center_offset()
}

/// Take an unscaled::XY representing the center of a tile, and return the min corner of the tile.
pub fn center_to_min_corner(spec: &sprite::Spec<BaseTiles>, xy: unscaled::XY) -> unscaled::XY {
    xy - spec.tile_center_offset()
}

pub fn sprite_xy(spec: &sprite::Spec<BaseTiles>, tile_sprite: TileSprite) -> sprite::XY<Renderable> {
    let tile = spec.tile();
    sprite::XY::<BaseTiles> {
        x: sprite::x(0) + sprite::W(tile_sprite as sprite::Inner % sprite::Inner::from(TILES_PER_ROW)) * tile.w.get(),
        y: sprite::y(0) + sprite::H(tile_sprite as sprite::Inner / sprite::Inner::from(TILES_PER_ROW)) * tile.h.get(),
    }.apply(spec)
}

pub fn rect(spec: &sprite::Spec<BaseTiles>, unscaled::XY{ x, y }: unscaled::XY) -> unscaled::Rect {
    let tile = spec.tile();
    unscaled::Rect {
        x: x,
        y: y,
        w: tile.w,
        h: tile.h,
    }
}

pub fn entity_rect(spec: &sprite::Spec<BaseTiles>, entity: &Entity) -> unscaled::Rect {
    let tile = spec.tile();
    let mut output = rect(spec, min_corner(spec, entity.xy()));

    if entity.offset_x > 0.0 {
        output.x += unscaled::W::from(entity.offset_x * offset::X::from(tile.w));
    } else if entity.offset_x < 0.0 {
        output.x -= unscaled::W::from(entity.offset_x.abs() * offset::X::from(tile.w));
    } else {
        // do nothing for zeroes or other weird values.
    }

    if entity.offset_y > 0.0 {
        output.y += unscaled::H::from(entity.offset_y * offset::Y::from(tile.h));
    } else if entity.offset_y < 0.0 {
        output.y -= unscaled::H::from(entity.offset_y.abs() * offset::Y::from(tile.h));
    } else {
        // do nothing for zeroes or other weird values.
    }

    output
}