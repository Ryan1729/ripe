use xs::Xs;
use models::{Speech, ShakeAmount};

use platform_types::{ARGB, Command, PALETTE, sprite, unscaled, command::{self, Rect}, arrow_timer::{self, ArrowTimer}, PaletteIndex, FONT_BASE_Y, FONT_WIDTH};

pub struct Commands {
    commands: Vec<Command>,
    shake_xd: unscaled::XD,
    shake_yd: unscaled::YD,
    rng: Xs,
}

impl Commands {
    pub fn new(seed: xs::Seed) -> Self {
        Self {
            commands: <_>::default(),
            shake_xd: <_>::default(),
            shake_yd: <_>::default(),
            rng: xs::from_seed(seed),
        }
    }

    fn push_with_screenshake(&mut self, mut command: Command) {
        command.rect.x_min += self.shake_xd;
        command.rect.y_min += self.shake_yd;
        command.rect.x_max += self.shake_xd;
        command.rect.y_max += self.shake_yd;

        self.commands.push(
            command
        );
    }

    pub fn slice(&self) -> &[Command] {
        &self.commands
    }

    pub fn begin_frame(&mut self, shake_amount: &mut ShakeAmount) {
        self.commands.clear();

        //
        // tick screenshake
        //
        if *shake_amount > 0 {
            *shake_amount -= 1;
        }

        if *shake_amount > 0 {
            let shake_angle: f32 = xs::zero_to_one(&mut self.rng) * core::f32::consts::TAU;

            let shake_amount = f32::from(*shake_amount);

            self.shake_xd = unscaled::XD::from(shake_angle.cos() * shake_amount);
            self.shake_yd = unscaled::YD::from(shake_angle.sin() * shake_amount);
        } else {
            self.shake_xd = unscaled::XD::ZERO;
            self.shake_yd = unscaled::YD::ZERO;
        }
    }

    pub fn sspr(
        &mut self,
        sprite_xy: sprite::XY,
        rect: command::Rect,
    ) {
        self.push_with_screenshake(
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
        self.push_with_screenshake(
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

    pub fn nine_slice(&mut self, nine_slice_sprite: nine_slice::Sprite, outer_rect: unscaled::Rect) {
        nine_slice::render(self, nine_slice_sprite, outer_rect);
    }

    pub fn next_arrow_in_corner_of(&mut self, next_arrow_sprite: next_arrow::Sprite, timer: ArrowTimer, rect: unscaled::Rect) {
        next_arrow::next_arrow_in_corner_of(self, next_arrow_sprite, timer, rect);
    }

    pub fn next_arrow(&mut self, next_arrow_sprite: next_arrow::Sprite, x: unscaled::X, y: unscaled::Y) {
        next_arrow::render(self, next_arrow_sprite, x, y);
    }

    pub fn speech(&mut self, speech: &Speech) {
        speech::render(self, speech);
    }
}

pub mod next_arrow {
    use super::*;

    pub type Sprite = u8;
    pub const TALKING: Sprite = 0;
    pub const INVENTORY: Sprite = 1;

    const ARROW_W: unscaled::W = unscaled::W(8);
    const ARROW_H: unscaled::H = unscaled::H(4);

    pub(crate) fn next_arrow_in_corner_of(
        commands: &mut Commands,
        next_arrow_sprite: Sprite,
        arrow_timer: ArrowTimer,
        rect: unscaled::Rect,
    ) {
        let unscaled::XY{ x, y } = rect.max_xy();

        let wh = arrow_timer::offset(arrow_timer);

        render(
            commands,
            next_arrow_sprite,
            x - ARROW_W - arrow_timer::MAX_W + wh.w,
            y - ARROW_H - arrow_timer::MAX_H + wh.h
        )
    }

    pub(crate) fn render(
        commands: &mut Commands,
        next_arrow_sprite: Sprite,
        x: unscaled::X,
        y: unscaled::Y,
    ) {
        let sprite_xy = match next_arrow_sprite & 1 {
            1 => sprite::XY { x: sprite::X(0), y: sprite::Y(0) },
            _ => sprite::XY { x: sprite::X(0), y: sprite::Y(4) },
        };

        commands.sspr(
            sprite_xy,
            Rect::from_unscaled(unscaled::Rect {
                x,
                y,
                w: ARROW_W,
                h: ARROW_H,
            })
        );
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

pub mod nine_slice {
    use super::*;

    pub type Sprite = u8;
    pub const TALKING: Sprite = 0;
    pub const INVENTORY: Sprite = 1;

    struct Slices {
        // Top left point on the rect that makes up the top left corner of the sprite.
        top_left: sprite::XY,
        // Top left point on the rect that makes up the top right corner of the sprite.
        top_right: sprite::XY,
        // Top left point on the rect that makes up the bottom left corner of the sprite.
        bottom_left: sprite::XY,
        // Top left point on the rect that makes up the bottom right corner of the sprite.
        bottom_right: sprite::XY,
        // Top left point on the rect that makes up the middle of the sprite.
        middle: sprite::XY,
        // Top left point on the rect that makes up the top edge of the sprite.
        top: sprite::XY,
        // Top left point on the rect that makes up the left edge of the sprite.
        left: sprite::XY,
        // Top left point on the rect that makes up the right edge of the sprite.
        right: sprite::XY,
        // Top left point on the rect that makes up the bottom edge of the sprite.
        bottom: sprite::XY,
    }

    const TALKING_SLICES: Slices = {
        let top_left: sprite::XY = sprite::XY {
            x: sprite::X(0),
            y: sprite::Y(8),
        };
        let top_right: sprite::XY = sprite::XY {
            x: sprite::X(20),
            y: sprite::Y(8),
        };
        let bottom_left: sprite::XY = sprite::XY {
            x: sprite::X(0),
            y: sprite::Y(28),
        };
        let bottom_right: sprite::XY = sprite::XY {
            x: sprite::X(20),
            y: sprite::Y(28),
        };
        let middle: sprite::XY = sprite::XY {
            x: sprite::x_const_add_w(top_left.x, EDGE_W),
            y: sprite::y_const_add_h(top_left.y, EDGE_H),
        };
        let top: sprite::XY = sprite::XY {
            x: middle.x,
            y: top_left.y,
        };
        let left: sprite::XY = sprite::XY {
            x: top_left.x,
            y: middle.y,
        };
        let right: sprite::XY = sprite::XY {
            x: top_right.x,
            y: middle.y,
        };
        let bottom: sprite::XY = sprite::XY {
            x: top.x,
            y: bottom_left.y,
        };

        Slices {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
            middle,
            top,
            left,
            right,
            bottom,
        }
    };

    const INVENTORY_SLICES: Slices = {
        let top_left: sprite::XY = sprite::XY {
            x: sprite::X(0),
            y: sprite::Y(32),
        };
        let top_right: sprite::XY = sprite::XY {
            x: sprite::X(20),
            y: sprite::Y(32),
        };
        let bottom_left: sprite::XY = sprite::XY {
            x: sprite::X(0),
            y: sprite::Y(52),
        };
        let bottom_right: sprite::XY = sprite::XY {
            x: sprite::X(20),
            y: sprite::Y(52),
        };
        let middle: sprite::XY = sprite::XY {
            x: sprite::x_const_add_w(top_left.x, EDGE_W),
            y: sprite::y_const_add_h(top_left.y, EDGE_H),
        };
        let top: sprite::XY = sprite::XY {
            x: middle.x,
            y: top_left.y,
        };
        let left: sprite::XY = sprite::XY {
            x: top_left.x,
            y: middle.y,
        };
        let right: sprite::XY = sprite::XY {
            x: top_right.x,
            y: middle.y,
        };
        let bottom: sprite::XY = sprite::XY {
            x: top.x,
            y: bottom_left.y,
        };

        Slices {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
            middle,
            top,
            left,
            right,
            bottom,
        }
    };

    const CENTER_W: unscaled::W = unscaled::W(16);
    const CENTER_H: unscaled::H = unscaled::H(16);

    const EDGE_W: unscaled::W = unscaled::W(4);
    const EDGE_H: unscaled::H = unscaled::H(4);

    pub(crate) fn render(commands: &mut Commands, nine_slice_sprite: Sprite, unscaled::Rect{ x, y, w, h }: unscaled::Rect) {
        let slices = match nine_slice_sprite & 1 {
            1 => INVENTORY_SLICES,
            _ => TALKING_SLICES,
        };

        let after_left_corner = x.saturating_add_w(EDGE_W);
        let before_right_corner = x.saturating_add_w(w).saturating_sub_w(EDGE_W);

        let below_top_corner = y.saturating_add_h(EDGE_H);
        let above_bottom_corner = y.saturating_add_h(h).saturating_sub_h(EDGE_H);

        // ABBBC
        // DEEEF
        // DEEEF
        // GHHHI

        // Draw E
        for fill_y in (below_top_corner.get()..above_bottom_corner.get()).step_by(CENTER_H.get() as _).map(unscaled::Y) {
            for fill_x in (after_left_corner.get()..before_right_corner.get()).step_by(CENTER_W.get() as _).map(unscaled::X) {
                commands.sspr(
                    slices.middle,
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
                slices.top,
                Rect::from_unscaled(unscaled::Rect {
                    x: fill_x,
                    y,
                    // Clamp this value so we don't draw past the edge.
                    w: core::cmp::min(CENTER_W, before_right_corner - fill_x),
                    h: EDGE_H,
                })
            );

            commands.sspr(
                slices.bottom,
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
                slices.left,
                Rect::from_unscaled(unscaled::Rect {
                    x,
                    y: fill_y,
                    // Clamp this value so we don't draw past the edge.
                    w: EDGE_W,
                    h: core::cmp::min(CENTER_H, above_bottom_corner - fill_y),
                })
            );

            commands.sspr(
                slices.right,
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
            slices.top_left,
            Rect::from_unscaled(unscaled::Rect {
                x,
                y,
                w: EDGE_W,
                h: EDGE_H,
            })
        );

        // Draw C
        commands.sspr(
            slices.top_right,
            Rect::from_unscaled(unscaled::Rect {
                x: before_right_corner,
                y,
                w: EDGE_W,
                h: EDGE_H,
            })
        );

        // Draw G
        commands.sspr(
            slices.bottom_left,
            Rect::from_unscaled(unscaled::Rect {
                x,
                y: above_bottom_corner,
                w: EDGE_W,
                h: EDGE_H,
            })
        );

        // Draw I
        commands.sspr(
            slices.bottom_right,
            Rect::from_unscaled(unscaled::Rect {
                x: before_right_corner,
                y: above_bottom_corner,
                w: EDGE_W,
                h: EDGE_H,
            })
        );
    }

    pub const fn inner_rect(outer_rect: unscaled::Rect) -> unscaled::Rect {
        unscaled::Rect {
            x: unscaled::x_const_add_w(outer_rect.x, EDGE_W),
            y: unscaled::y_const_add_h(outer_rect.y, EDGE_H),
            w: unscaled::w_const_sub(outer_rect.w, unscaled::w_const_mul(EDGE_W, 2)),
            h: unscaled::h_const_sub(outer_rect.h, unscaled::h_const_mul(EDGE_H, 2)),
        }
    }
}

pub mod speech {
    use super::*;
    use crate::nine_slice;
    use platform_types::unscaled;

    pub const SPACING: unscaled::Inner = 20;

    pub const OUTER_RECT: unscaled::Rect = unscaled::Rect {
        x: unscaled::X(SPACING),
        y: unscaled::Y(platform_types::command::HEIGHT - 120),
        w: unscaled::W(platform_types::command::WIDTH - (SPACING * 2)),
        h: unscaled::H(120),
    };

    pub const INNER_RECT: unscaled::Rect = nine_slice::inner_rect(OUTER_RECT);

    pub(crate) fn render(commands: &mut Commands, speech: &models::Speech) {
        // This might get more complicated, with like text colouring or effects, etc.
        commands.print_lines(
            INNER_RECT.xy(),
            0,
            speech.text.as_bytes(),
            6,
        )
    }
}

pub const TEN_CHAR: u8 = 27;

pub const CLUB_CHAR: u8 = 31;
pub const DIAMOND_CHAR: u8 = 29;
pub const HEART_CHAR: u8 = 30;
pub const SPADE_CHAR: u8 = 28;

pub const CHAR_SIZE: u8 = 8;
pub const CHAR_W: unscaled::W = unscaled::W(CHAR_SIZE as _);
pub const CHAR_H: unscaled::H = unscaled::H(CHAR_SIZE as _);

pub const FONT_FLIP: u8 = 128;

