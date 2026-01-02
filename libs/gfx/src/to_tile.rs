use platform_types::{sprite, unscaled, TILES_PER_ROW};
use models::{Entity, xy::{XY}, TileSprite, offset};

// TODO These should be changeable at runtime. Having it be here in this module as a static is inconvenient.
//      I think the place for it is on the gfx::Commands. So move this module into there, and start passing
//      them into these functions, adding high level methods to gfx::Commands as needed {
const TILE_W: unscaled::W = unscaled::W(16);
const TILE_H: unscaled::H = unscaled::H(16);

/// Where the tiles start on the spreadsheet.
const TILES_Y: sprite::Y = sprite::Y(0);
// }

const CENTER_OFFSET: unscaled::WH = unscaled::WH{
    w: TILE_W.halve(),
    h: TILE_H.halve(),
};

/// Take a models::XY to the unscaled::XY representing the corner of the tile, with the mininum x/y values.
/// Suitable for drawing the tile at that point
pub fn min_corner(xy: XY) -> unscaled::XY {
    // Could see this needing to be passed in later
    // And we might even want an intermediate type 
    // here that can go negative or fractional
    // that we'd ulimately clip to unscaled types
    let base_offset = unscaled::WH { w: TILE_W, h: TILE_H };

    let x = unscaled::X(xy.x.get() * TILE_W.get());
    let y = unscaled::Y(xy.y.get() * TILE_H.get());

    unscaled::XY { x, y } + base_offset
}

/// Take a models::XY to the unscaled::XY representing the center of the tile.
pub fn center(xy: XY) -> unscaled::XY {
    min_corner(xy) + CENTER_OFFSET
}

/// Take an unscaled::XY representing the center of a tile, and return the min corner of the tile.
pub fn center_to_min_corner(xy: unscaled::XY) -> unscaled::XY {
    xy - CENTER_OFFSET
}

pub fn sprite_xy(tile_sprite: TileSprite) -> sprite::XY {
    sprite::XY {
        x: sprite::X(0) + sprite::W(tile_sprite as sprite::Inner % sprite::Inner::from(TILES_PER_ROW)) * TILE_W.get(),
        y: TILES_Y + sprite::H(tile_sprite as sprite::Inner / sprite::Inner::from(TILES_PER_ROW)) * TILE_H.get(),
    }
}

pub fn rect(unscaled::XY{ x, y }: unscaled::XY) -> unscaled::Rect {
    unscaled::Rect {
        x: x,
        y: y,
        w: TILE_W,
        h: TILE_H,
    }
}

pub fn entity_rect(entity: &Entity) -> unscaled::Rect {
    let mut output = rect(min_corner(entity.xy()));

    if entity.offset_x > 0.0 {
        output.x += unscaled::W::from(entity.offset_x * offset::X::from(TILE_W));
    } else if entity.offset_x < 0.0 {
        output.x -= unscaled::W::from(entity.offset_x.abs() * offset::X::from(TILE_W));
    } else {
        // do nothing for zeroes or other weird values.
    }

    if entity.offset_y > 0.0 {
        output.y += unscaled::H::from(entity.offset_y * offset::Y::from(TILE_H));
    } else if entity.offset_y < 0.0 {
        output.y -= unscaled::H::from(entity.offset_y.abs() * offset::Y::from(TILE_H));
    } else {
        // do nothing for zeroes or other weird values.
    }

    output
}