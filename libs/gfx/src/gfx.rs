use models::{};

use platform_types::{ARGB, Command, PALETTE, sprite, unscaled, command::{self, Rect}, PaletteIndex, FONT_BASE_Y, FONT_WIDTH};

#[derive(Default)]
pub struct Commands {
    commands: Vec<Command>,
}

impl Commands {
    pub fn slice(&self) -> &[Command] {
        &self.commands
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn sspr(
        &mut self,
        sprite_xy: sprite::XY,
        rect: command::Rect,
    ) {
        self.commands.push(
            Command {
                sprite_xy,
                rect,
                colour_override: 0,
            }
        );
    }

    pub fn print_char(
        &mut self,
        character: u8, 
        x: unscaled::X,
        y: unscaled::Y,
        colour: PaletteIndex
    ) {
        fn get_char_xy(sprite_number: u8) -> sprite::XY {
            type Inner = sprite::Inner;
            let sprite_number = Inner::from(sprite_number);
            const CH_SIZE: Inner = CHAR_SIZE as Inner;
            const SPRITES_PER_ROW: Inner = FONT_WIDTH as Inner / CH_SIZE;
        
            sprite::XY {
                x: sprite::X(
                    (sprite_number % SPRITES_PER_ROW) * CH_SIZE
                ),
                y: sprite::Y(
                    FONT_BASE_Y as Inner + 
                    (sprite_number / SPRITES_PER_ROW) * CH_SIZE
                ),
            }
        }

        let sprite_xy = get_char_xy(character);
        self.commands.push(
            Command {
                sprite_xy,
                rect: Rect::from_unscaled(unscaled::Rect {
                    x,
                    y,
                    w: CHAR_W,
                    h: CHAR_H,
                }),
                colour_override: PALETTE[colour as usize],
            }
        );
    }

    pub fn print_line(
        &mut self,
        bytes: &[u8],
        mut xy : unscaled::XY,
        colour: PaletteIndex,
    ) {
        for &c in bytes.iter() {
            self.print_char(c, xy.x, xy.y, colour);
            xy.x += CHAR_W;
        }
    }

    pub fn print_lines(
        &mut self,
        base_xy: unscaled::XY,
        top_index_with_offset: usize,
        to_print: &[u8],
        colour: PaletteIndex,
    ) {
        for (y, line) in Self::lines(to_print)
            .skip((top_index_with_offset as u16 / CHAR_H.get()) as usize)
            .take(usize::from(command::HEIGHT * CHAR_H))
            .enumerate()
        {
            let y = y as unscaled::Inner;

            let offset = top_index_with_offset as u16 % CHAR_H.get();

            self.print_line(
                line,
                base_xy
                // TODO investigate scrolling shimmering which seems to be
                // related to this part. Do we need to make the scrolling
                // speed up, then slow down or something? or is the offset
                // calculation just wrong?  Maybe it won't look right unless
                // we add more in-between frames?
                + unscaled::H(
                    ((y + 1) * CHAR_H.get())
                    - offset
                    - 1
                )
                + CHAR_H,
                colour
            );
        }
    }

    #[allow(unused)]
    pub fn reflow(bytes: &[u8], width: usize) -> Vec<u8> {
        if width == 0 || bytes.is_empty() {
            return Vec::new();
        }

        let mut output = Vec::with_capacity(bytes.len() + bytes.len() / width);

        let mut x = 0;
        for word in Self::split_whitespace(bytes) {
            x += word.len();

            if x == width && x == word.len() {
                output.extend(word.iter());
                continue;
            }

            if x >= width {
                output.push(b'\n');

                x = word.len();
            } else if x > word.len() {
                output.push(b' ');

                x += 1;
            }
            output.extend(word.iter());
        }

        output
    }

    pub fn split_whitespace(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
        bytes
            .split(|b| b.is_ascii_whitespace())
            .filter(|word| !word.is_empty())
    }

    pub fn lines(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
        bytes.split(|&b| b == b'\n')
    }

    pub fn nine_slice(&mut self, outer_rect: unscaled::Rect) {
        nine_slice::render(self, outer_rect);
    }
}

#[cfg(test)]
mod nine_slice_works {
    use super::*;

    #[test]
    fn on_this_uneven_example() {
        let mut commands = Commands::default();

        commands.nine_slice(
            unscaled::X(0),
            unscaled::Y(0),
            unscaled::W(32),
            unscaled::H(20),
        );

        let mut actual = commands.commands.iter().map(|c| c.rect.clone()).collect::<Vec<_>>();
        // This was mainly written as a quick way to just look at the results. Might be useful to keep around, so
        // put in an assert that is unlikely to break later, and if it does, it should be clear why
        assert_eq!(actual.len(), 12);
    }
}

mod nine_slice {
    use super::*;

    // Top left point on the rect that makes up the top left corner of the sprite.
    const TOP_LEFT: sprite::XY = sprite::XY {
        x: sprite::X(0),
        y: sprite::Y(32),
    };

    // Top left point on the rect that makes up the top right corner of the sprite.
    const TOP_RIGHT: sprite::XY = sprite::XY {
        x: sprite::X(20),
        y: sprite::Y(32),
    };

    // Top left point on the rect that makes up the bottom left corner of the sprite.
    const BOTTOM_LEFT: sprite::XY = sprite::XY {
        x: sprite::X(0),
        y: sprite::Y(52),
    };

    // Top left point on the rect that makes up the bottom right corner of the sprite.
    const BOTTOM_RIGHT: sprite::XY = sprite::XY {
        x: sprite::X(20),
        y: sprite::Y(52),
    };

    // Top left point on the rect that makes up the middle of the sprite.
    const MIDDLE: sprite::XY = sprite::XY {
        x: sprite::x_const_add_w(TOP_LEFT.x, EDGE_W),
        y: sprite::y_const_add_h(TOP_LEFT.y, EDGE_H),
    };

    // Top left point on the rect that makes up the top edge of the sprite.
    const TOP: sprite::XY = sprite::XY {
        x: MIDDLE.x,
        y: TOP_LEFT.y,
    };

    // Top left point on the rect that makes up the left edge of the sprite.
    const LEFT: sprite::XY = sprite::XY {
        x: TOP_LEFT.x,
        y: MIDDLE.y,
    };

    // Top left point on the rect that makes up the right edge of the sprite.
    const RIGHT: sprite::XY = sprite::XY {
        x: TOP_RIGHT.x,
        y: MIDDLE.y,
    };

    // Top left point on the rect that makes up the bottom edge of the sprite.
    const BOTTOM: sprite::XY = sprite::XY {
        x: TOP.x,
        y: BOTTOM_LEFT.y,
    };

    const CENTER_W: unscaled::W = unscaled::W(16);
    const CENTER_H: unscaled::H = unscaled::H(16);

    const EDGE_W: unscaled::W = unscaled::W(4);
    const EDGE_H: unscaled::H = unscaled::H(4);

    pub fn render(commands: &mut Commands, unscaled::Rect{ x, y, w, h }: unscaled::Rect) {
        let after_left_corner = x.saturating_add(EDGE_W);
        let before_right_corner = x.saturating_add(w).saturating_sub(EDGE_W);

        let below_top_corner = y.saturating_add(EDGE_H);
        let above_bottom_corner = y.saturating_add(h).saturating_sub(EDGE_H);

        // ABBBC
        // DEEEF
        // DEEEF
        // GHHHI

        // Draw E
        for fill_y in (below_top_corner.get()..above_bottom_corner.get()).step_by(CENTER_H.get() as _).map(unscaled::Y) {
            for fill_x in (after_left_corner.get()..before_right_corner.get()).step_by(CENTER_W.get() as _).map(unscaled::X) {
                commands.sspr(
                    MIDDLE,
                    Rect::from_unscaled(unscaled::Rect {
                        x: fill_x,
                        y: fill_y,
                        // Clamp these values so we don't draw past the edge.
                        w: core::cmp::min(CENTER_W, before_right_corner - fill_x),
                        h: core::cmp::min(CENTER_H, above_bottom_corner - fill_y),
                    })
                );
            }
        }

        // Draw B and H
        for fill_x in (after_left_corner.get()..before_right_corner.get()).step_by(CENTER_W.get() as _).map(unscaled::X) {
            commands.sspr(
                TOP,
                Rect::from_unscaled(unscaled::Rect {
                    x: fill_x,
                    y,
                    // Clamp this value so we don't draw past the edge.
                    w: core::cmp::min(CENTER_W, before_right_corner - fill_x),
                    h: EDGE_H,
                })
            );

            commands.sspr(
                BOTTOM,
                Rect::from_unscaled(unscaled::Rect {
                    x: fill_x,
                    y: above_bottom_corner,
                    // Clamp this value so we don't draw past the edge.
                    w: core::cmp::min(CENTER_W, before_right_corner - fill_x),
                    h: EDGE_H,
                })
            );
        }

        // Draw D and F
        for fill_y in (below_top_corner.get()..above_bottom_corner.get()).step_by(CENTER_H.get() as _).map(unscaled::Y) {
            commands.sspr(
                LEFT,
                Rect::from_unscaled(unscaled::Rect {
                    x,
                    y: fill_y,
                    // Clamp this value so we don't draw past the edge.
                    w: EDGE_W,
                    h: core::cmp::min(CENTER_H, above_bottom_corner - fill_y),
                })
            );

            commands.sspr(
                RIGHT,
                Rect::from_unscaled(unscaled::Rect {
                    x: before_right_corner,
                    y: fill_y,
                    // Clamp this value so we don't draw past the edge.
                    w: EDGE_W,
                    h: core::cmp::min(CENTER_H, above_bottom_corner - fill_y),
                })
            );
        }

        // Draw A
        commands.sspr(
            TOP_LEFT,
            Rect::from_unscaled(unscaled::Rect {
                x,
                y,
                w: EDGE_W,
                h: EDGE_H,
            })
        );

        // Draw C
        commands.sspr(
            TOP_RIGHT,
            Rect::from_unscaled(unscaled::Rect {
                x: before_right_corner,
                y,
                w: EDGE_W,
                h: EDGE_H,
            })
        );

        // Draw G
        commands.sspr(
            BOTTOM_LEFT,
            Rect::from_unscaled(unscaled::Rect {
                x,
                y: above_bottom_corner,
                w: EDGE_W,
                h: EDGE_H,
            })
        );

        // Draw I
        commands.sspr(
            BOTTOM_RIGHT,
            Rect::from_unscaled(unscaled::Rect {
                x: before_right_corner,
                y: above_bottom_corner,
                w: EDGE_W,
                h: EDGE_H,
            })
        );
    }

    pub fn inner_rect(outer_rect: unscaled::Rect) -> unscaled::Rect {
        unscaled::Rect {
            x: outer_rect.x + EDGE_W,
            y: outer_rect.y + EDGE_H,
            w: outer_rect.w - (EDGE_W * 2),
            h: outer_rect.h - (EDGE_H * 2),
        }
    }
}
pub use nine_slice::{inner_rect as nine_slice_inner_rect};

pub const TEN_CHAR: u8 = 27;

pub const CLUB_CHAR: u8 = 31;
pub const DIAMOND_CHAR: u8 = 29;
pub const HEART_CHAR: u8 = 30;
pub const SPADE_CHAR: u8 = 28;

pub const CHAR_SIZE: u8 = 8;
pub const CHAR_W: unscaled::W = unscaled::W(CHAR_SIZE as _);
pub const CHAR_H: unscaled::H = unscaled::H(CHAR_SIZE as _);

pub const FONT_FLIP: u8 = 128;

