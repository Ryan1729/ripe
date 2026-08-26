use xs::Xs;
use models::{ShakeAmount, Speech};

pub mod to_tile;

use gfx_sizes::ARGB;
use pak_types::{sprite::{self, Renderable, BaseFont, BaseUI},};
use platform_types::{Command, PALETTE, unscaled, command, arrow_timer::{self, ArrowTimer}, PaletteIndex};
use text::byte_slice as text;

/// 64k fade frames ought to be enough for anybody!
type FadeTimer = u16;

#[derive(Clone)]
pub struct FadeMessage {
    pub message: Vec<u8>,
    pub fade_timer: FadeTimer,
    pub xy: unscaled::XY,
    // TODO: Should conceptually be an "XYD", as in an XY delta but that type doesn't exist yet.
    pub offset_per_frame: unscaled::WH,
}

// TODO? Put a hard limit on the amount of these, with I guess LIFO eviction?
pub type FadeMessages = Vec<FadeMessage>;

pub struct Commands {
    commands: Vec<Command>,
    font_spec: sprite::Spec<BaseFont>,
    ui_spec: sprite::Spec<BaseUI>,
    shake_xd: unscaled::XD,
    shake_yd: unscaled::YD,
    rng: Xs,
    fade_messages: FadeMessages,
}

// Okay, we need clipping of rendered sprites and that seems to best expressed via negative coords
// So we'll need to change unscaled::X and unscaled::Y to be signed, and adjusting clipping.
// It seems worth attempting to do this without invasive changes to the render crate, if possible.
//     So rect clipping, sprite XY adjustments, and command filtering would be done when creating the commands
// Proposed steps:
// Change the gfx API to take unscaled::Rect, but convert to command::Rect inside that crate.
// Break the connection between command::Rect and unscaled::*
//    Passing them in is fine, but changing the datatype in the next step should not affect what is in a Command.
// Change the unsigned inner datatype to signed.
// Implement proper clipping that allows sliding a sprite off the edge of the screen.


// TODO Probably worth backporting the new clipping into the template once we have it working well.


impl Commands {
    pub fn new(
        seed: xs::Seed,
        font_spec: sprite::Spec<BaseFont>,
        ui_spec: sprite::Spec<BaseUI>,
    ) -> Self {
        Self {
            commands: <_>::default(),
            font_spec,
            ui_spec,
            shake_xd: <_>::default(),
            shake_yd: <_>::default(),
            rng: xs::from_seed(seed),
            fade_messages: FadeMessages::with_capacity(4),
        }
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

    pub fn end_frame(&mut self) {
        //
        // Tick the fade messages
        //
        for i in (0..self.fade_messages.len()).rev() {
            let message = &mut self.fade_messages[i];

            message.fade_timer = message.fade_timer.saturating_sub(1);
            if message.fade_timer == 0 {
                self.fade_messages.remove(i);
                continue
            }

            // TODO? A timer or other method to be able to move less than one pixel per frame?
            //     At that point, do we want sub-pixel blending enough to implement it?
            message.xy += message.offset_per_frame;
        }

        //
        // Draw the fade messages on top of everything
        //
        // If always drawing these over everything becomes an issue, we can 
        // add a boolean onto self here, that can be set to prevent the 
        // drawing, and maybe the ticking too. Not sure whether it should 
        // be reset every frame or not.
        for message in &self.fade_messages {
            print::lines(
                &mut self.commands,
                &self.font_spec,
                self.shake_xd,
                self.shake_yd,
                message.xy,
                0,
                &message.message,
                6,
            );
        }
    }

    pub fn ui_edge_wh(&self) -> unscaled::WH {
        self.ui_spec.tile().halve()
    }
}

pub trait AddDrawCommands {
    fn sspr_override(
        &mut self,
        sprite_xy: sprite::XY<Renderable>,
        unscaled_rect: unscaled::Rect,
        colour_override: ARGB,
    );

    fn sspr(
        &mut self,
        sprite_xy: sprite::XY<Renderable>,
        unscaled_rect: unscaled::Rect,
    ) {
        self.sspr_override(
            sprite_xy,
            unscaled_rect,
            0,
        );
    }

    fn print_char(
        &mut self,
        character: u8,
        x: unscaled::X,
        y: unscaled::Y,
        colour: PaletteIndex
    );

    fn print_line(
        &mut self,
        bytes: &[u8],
        xy: unscaled::XY,
        colour: PaletteIndex,
    );

    fn print_lines(
        &mut self,
        base_xy: unscaled::XY,
        top_index_with_offset: usize,
        to_print: &[u8],
        colour: PaletteIndex,
    );

    fn nine_slice(
        &mut self,
        nine_slice_sprite: nine_slice::Sprite,
        outer_rect: unscaled::Rect
    );

    fn next_arrow_in_corner_of(
        &mut self,
        next_arrow_sprite: next_arrow::Sprite,
        timer: ArrowTimer,
        rect: unscaled::Rect
    );

    fn next_arrow(
        &mut self,
        next_arrow_sprite: next_arrow::Sprite,
        x: unscaled::X,
        y: unscaled::Y
    );

    fn speech(&mut self, speech: &Speech);

    fn push_fade_message(&mut self, message: Vec<u8>, xy: unscaled::XY);

    fn clipped<'commands>(&'commands mut self, rect: unscaled::Rect) -> ClippedCommands<'commands, Self> where Self: Sized {
        ClippedCommands {
            commands: self,
            rect,
        }
    }

    fn raw(&mut self) -> &mut Commands;
}

impl AddDrawCommands for Commands {
    fn sspr_override(
        &mut self,
        sprite_xy: sprite::XY<Renderable>,
        unscaled_rect: unscaled::Rect,
        colour_override: ARGB,
    ) {
        push_with_screenshake(
            &mut self.commands,
            self.shake_xd,
            self.shake_yd,
            sprite_xy,
            unscaled_rect,
            colour_override,
        );
    }

    // sspr uses the default impl that forwards to sspr_override, with a 0 override.

    fn print_char(
        &mut self,
        character: u8,
        x: unscaled::X,
        y: unscaled::Y,
        colour: PaletteIndex
    ) {
        print::char(
            &mut self.commands,
            &self.font_spec,
            self.shake_xd,
            self.shake_yd,
            character,
            x,
            y,
            colour,
        )
    }

    fn print_line(
        &mut self,
        bytes: &[u8],
        xy: unscaled::XY,
        colour: PaletteIndex,
    ) {
        print::line(
            &mut self.commands,
            &self.font_spec,
            self.shake_xd,
            self.shake_yd,
            bytes,
            xy,
            colour,
        )
    }

    fn print_lines(
        &mut self,
        base_xy: unscaled::XY,
        top_index_with_offset: usize,
        to_print: &[u8],
        colour: PaletteIndex,
    ) {
        print::lines(
            &mut self.commands,
            &self.font_spec,
            self.shake_xd,
            self.shake_yd,
            base_xy,
            top_index_with_offset,
            to_print,
            colour,
        )
    }

    fn nine_slice(&mut self, nine_slice_sprite: nine_slice::Sprite, outer_rect: unscaled::Rect) {
        nine_slice::render(self, nine_slice_sprite, outer_rect);
    }

    fn next_arrow_in_corner_of(&mut self, next_arrow_sprite: next_arrow::Sprite, timer: ArrowTimer, rect: unscaled::Rect) {
        next_arrow::next_arrow_in_corner_of(self, next_arrow_sprite, timer, rect);
    }

    fn next_arrow(&mut self, next_arrow_sprite: next_arrow::Sprite, x: unscaled::X, y: unscaled::Y) {
        next_arrow::render(self, next_arrow_sprite, x, y);
    }

    fn speech(&mut self, speech: &Speech) {
        speech::render(self, speech);
    }

    fn push_fade_message(&mut self, message: Vec<u8>, xy: unscaled::XY) {
        self.fade_messages.push(FadeMessage {
            message,
            // TODO? Scale this based on text length?
            fade_timer: 100,
            xy,
            // TODO? Scale this based on text length?
            offset_per_frame: unscaled::WH { w: unscaled::W::ZERO, h: unscaled::H::ONE },
        });
    }

    fn raw(&mut self) -> &mut Commands {
        self
    }
}

pub struct ClippedCommands<'commands, C: AddDrawCommands + Sized> {
    commands: &'commands mut C,
    rect: unscaled::Rect,
}

macro_rules! clip_new_commands {
    ($clipped: ident, $code: block) => ({        
        let old_count = {
            let clipped: &mut ClippedCommands<'commands, C> = $clipped;
            let raw: &mut Commands = clipped.raw();
            raw.commands.len()
        };

        {
            $code;
        }

        let clipped: &mut ClippedCommands<'commands, C> = $clipped;
        let rect = clipped.rect.clone();
        let raw: &mut Commands = clipped.raw();

        // Reverse so we have a chance to hit the fast path of removing
        // the last element of the vector, multiple times.
        // TODO? Benchmark with a version that uses Vec::retain_mut, and 
        // an index variable?
        for i in (old_count..raw.commands.len()).rev() {
            if let Some(new_cmd) = raw.commands[i].clipped_to(rect) {
                raw.commands[i] = new_cmd;
            } else {
                raw.commands.remove(i);
            }
        }

    })
}

impl <'commands, C: AddDrawCommands + Sized> AddDrawCommands for ClippedCommands<'commands, C> {
    fn sspr_override(
        &mut self,
        sprite_xy: sprite::XY<Renderable>,
        unscaled_rect: unscaled::Rect,
        colour_override: ARGB,
    ) {
        clip_new_commands! {
            self,
            {
                self.commands.sspr_override(
                    sprite_xy,
                    unscaled_rect,
                    colour_override,
                );
            }
        }
    }

    // sspr uses the default impl that forwards to sspr_override, with a 0 override.

    fn print_char(
        &mut self,
        character: u8,
        x: unscaled::X,
        y: unscaled::Y,
        colour: PaletteIndex
    ) {
        clip_new_commands! {
            self,
            {
                self.commands.print_char(
                    character,
                    x,
                    y,
                    colour,
                );
            }
        }
    }

    fn print_line(
        &mut self,
        bytes: &[u8],
        xy: unscaled::XY,
        colour: PaletteIndex,
    ) {
        clip_new_commands! {
            self,
            {
                self.commands.print_line(
                    bytes,
                    xy,
                    colour,
                );
            }
        }
    }

    fn print_lines(
        &mut self,
        base_xy: unscaled::XY,
        top_index_with_offset: usize,
        to_print: &[u8],
        colour: PaletteIndex,
    ) {
        clip_new_commands! {
            self,
            {
                self.commands.print_lines(
                    base_xy,
                    top_index_with_offset,
                    to_print,
                    colour,
                );
            }
        }
    }

    fn nine_slice(&mut self, nine_slice_sprite: nine_slice::Sprite, outer_rect: unscaled::Rect) {
        clip_new_commands! {
            self,
            { self.commands.nine_slice(nine_slice_sprite, outer_rect); }
        }
    }

    fn next_arrow_in_corner_of(&mut self, next_arrow_sprite: next_arrow::Sprite, timer: ArrowTimer, rect: unscaled::Rect) {
        clip_new_commands! {
            self,
            { self.commands.next_arrow_in_corner_of(next_arrow_sprite, timer, rect); }
        }
    }

    fn next_arrow(&mut self, next_arrow_sprite: next_arrow::Sprite, x: unscaled::X, y: unscaled::Y) {
        clip_new_commands! {
            self,
            { self.commands.next_arrow(next_arrow_sprite, x, y); }
        }
    }

    fn speech(&mut self, speech: &Speech) {
        clip_new_commands! {
            self,
            { self.commands.speech(speech); }
        }
    }

    fn push_fade_message(&mut self, message: Vec<u8>, xy: unscaled::XY) {
        clip_new_commands! {
            self,
            { self.commands.push_fade_message(message, xy); }
        }
    }

    fn raw(&mut self) -> &mut Commands {
        self.commands.raw()
    }
}

mod print {
    use super::*;

    pub fn char(
        command_vec: &mut Vec<Command>, 
        spec: &sprite::Spec<BaseFont>,
        shake_xd: unscaled::XD,
        shake_yd: unscaled::YD,
        character: u8, 
        x: unscaled::X,
        y: unscaled::Y,
        colour: PaletteIndex
    ) {
        fn get_char_xy(spec: &sprite::Spec<BaseFont>, sprite_number: u8) -> sprite::XY<BaseFont> {
            type Inner = sprite::Inner;
            let sprite_number = Inner::from(sprite_number);

            const SPRITES_PER_ROW: Inner = 16;
        
            sprite::XY::<BaseFont> {
                x: sprite::x::<BaseFont>(0) + sprite::W::new(
                    ((sprite_number % SPRITES_PER_ROW) * spec.tile().w.u16()).try_into().expect("bad char x")
                ),
                y: sprite::y::<BaseFont>(0) + sprite::H::new(
                    ((sprite_number / SPRITES_PER_ROW) * spec.tile().h.u16()).try_into().expect("bad char y")
                ),
            }
        }

        let sprite_xy = get_char_xy(spec, character).apply(spec);
        push_with_screenshake(
            command_vec,
            shake_xd,
            shake_yd,
            sprite_xy,
            unscaled::Rect {
                x,
                y,
                w: spec.tile().w,
                h: spec.tile().h,
            },
            PALETTE[colour as usize],
        );
    }
    
    pub fn line(
        command_vec: &mut Vec<Command>,
        spec: &sprite::Spec<BaseFont>,
        shake_xd: unscaled::XD,
        shake_yd: unscaled::YD,
        bytes: &[u8],
        mut xy : unscaled::XY,
        colour: PaletteIndex,
    ) {
        for &c in bytes.iter() {
            char(
                command_vec,
                spec,
                shake_xd,
                shake_yd,
                c,
                xy.x,
                xy.y,
                colour
            );
            xy.x += spec.tile().w;
        }
    }
    
    pub fn lines(
        command_vec: &mut Vec<Command>,
        spec: &sprite::Spec<BaseFont>,
        shake_xd: unscaled::XD,
        shake_yd: unscaled::YD,
        base_xy: unscaled::XY,
        top_index_with_offset: usize,
        to_print: &[u8],
        colour: PaletteIndex,
    ) {
        for (y, text_line) in text::lines(to_print)
            .skip((top_index_with_offset as unscaled::Inner / spec.tile().h.get()) as usize)
            .take(usize::from(command::HEIGHT * spec.tile().h.u16()))
            .enumerate()
        {
            let y = y as unscaled::Inner;
    
            let offset = top_index_with_offset as unscaled::Inner % spec.tile().h.get();
    
            line(
                command_vec,
                spec,
                shake_xd,
                shake_yd,
                text_line,
                base_xy
                // TODO investigate scrolling shimmering which seems to be
                // related to this part. Do we need to make the scrolling
                // speed up, then slow down or something? or is the offset
                // calculation just wrong? Maybe it won't look right unless
                // we add more in-between frames?
                + unscaled::H::new(
                    (y * spec.tile().h.get())
                    - offset
                ),
                colour
            );
        }
    }
}

fn push_with_screenshake(
    command_vec: &mut Vec<Command>, 
    shake_xd: unscaled::XD,
    shake_yd: unscaled::YD,
    sprite_xy: sprite::XY<sprite::Renderable>,
    mut rect: unscaled::Rect,
    colour_override: ARGB,
) {
    rect.x += shake_xd;
    rect.y += shake_yd;

    if let Some(command) = Command::new(sprite_xy, rect, colour_override) {
        command_vec.push(command);
    }
}

pub mod speech {
    use super::*;
    use crate::nine_slice;
    use platform_types::unscaled;

    pub const SPACING: unscaled::Inner = 20;

    pub const OUTER_RECT: unscaled::Rect = unscaled::Rect {
        x: unscaled::X(SPACING),
        y: unscaled::Y(platform_types::command::HEIGHT_SIGNED - 120),
        w: unscaled::W::new(platform_types::command::WIDTH_SIGNED - (SPACING * 2)),
        h: unscaled::H::new(120),
    };

    pub(crate) fn render(commands: &mut Commands, speech: &models::Speech) {
        let mut inner_rect = nine_slice::inner_rect(commands.ui_edge_wh(), OUTER_RECT);

        // TODO? Bother figuring out why these particular adjustments are needed to make it look right?
        const X_NUDGE: unscaled::W = unscaled::W::new(2);
        const Y_NUDGE: unscaled::H = unscaled::H::new(2);

        inner_rect.x = unscaled::x_const_add_w(inner_rect.x, X_NUDGE);
        inner_rect.w = unscaled::w_const_sub(inner_rect.w, unscaled::w_const_mul(X_NUDGE, 2));

        inner_rect.y = unscaled::y_const_add_h(inner_rect.y, Y_NUDGE);
        inner_rect.h = unscaled::h_const_sub(inner_rect.h, unscaled::h_const_mul(Y_NUDGE, 2));

        // This might get more complicated, with like text colouring or effects, etc.
        commands.print_lines(
            inner_rect.xy(),
            0,
            speech.text.as_bytes(),
            6,
        )
    }
}

pub mod next_arrow {
    use super::*;

    pub type Sprite = u8;
    pub const TALKING: Sprite = 0;
    pub const INVENTORY: Sprite = 1;

    const ARROW_W: unscaled::W = unscaled::W::new(8);
    const ARROW_H: unscaled::H = unscaled::H::new(4);

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
            1 => sprite::XY::<BaseUI> { x: sprite::x::<BaseUI>(0), y: sprite::y::<BaseUI>(4) },
            _ => sprite::XY::<BaseUI> { x: sprite::x::<BaseUI>(0), y: sprite::y::<BaseUI>(0) },
        };

        commands.sspr(
            sprite_xy.apply(&commands.ui_spec),
            unscaled::Rect {
                x,
                y,
                w: ARROW_W,
                h: ARROW_H,
            }
        );
    }
}

#[cfg(test)]
mod nine_slice_works {
    use super::*;

    #[test]
    fn on_this_uneven_example() {
        let specs = sprite::Specs::default();

        let mut commands = Commands::new(
            <_>::default(),
            specs.base_font,
            specs.base_ui,
        );

        commands.nine_slice(
            0,
            unscaled::Rect {
                x: unscaled::X(0),
                y: unscaled::Y(0),
                w: unscaled::W(32),
                h: unscaled::H(20),
            },
        );

        let actual = commands.commands.iter().map(|c| c.rect.clone()).collect::<Vec<_>>();
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
    pub const SELECTRUM: Sprite = 2;
    pub const CONTEXT_MENU: Sprite = 3;

    struct Slices {
        // Top left point on the rect that makes up the top left corner of the sprite.
        top_left: sprite::XY<BaseUI>,
        // Top left point on the rect that makes up the top right corner of the sprite.
        top_right: sprite::XY<BaseUI>,
        // Top left point on the rect that makes up the bottom left corner of the sprite.
        bottom_left: sprite::XY<BaseUI>,
        // Top left point on the rect that makes up the bottom right corner of the sprite.
        bottom_right: sprite::XY<BaseUI>,
        // Top left point on the rect that makes up the middle of the sprite.
        middle: sprite::XY<BaseUI>,
        // Top left point on the rect that makes up the top edge of the sprite.
        top: sprite::XY<BaseUI>,
        // Top left point on the rect that makes up the left edge of the sprite.
        left: sprite::XY<BaseUI>,
        // Top left point on the rect that makes up the right edge of the sprite.
        right: sprite::XY<BaseUI>,
        // Top left point on the rect that makes up the bottom edge of the sprite.
        bottom: sprite::XY<BaseUI>,
    }

    pub(crate) fn render(
        commands: &mut Commands,
        nine_slice_sprite: Sprite,
        unscaled::Rect{ x, y, w, h }: unscaled::Rect
    ) {
        let spec = &commands.ui_spec;

        let edge_wh = commands.ui_edge_wh();
        debug_assert_eq!(edge_wh.w, spec.tile().halve().w);
        debug_assert_eq!(edge_wh.h, spec.tile().halve().h);
        // 3 for 3 by 3 cells, minus the edges on both ends, (thus 2).
        // So `spec.tile() * 3 - (edge_wh * 2)`. Since `edge_wh = spec.tile() / 2`
        // this simplifes to spec.tile() + spec.tile().
        let center_wh = spec.tile() + spec.tile();

        // We need the height of one of those 3 by 3 cells too.
        // Given `center_wh = spec.tile() + spec.tile()`, that is:
        let supertile_wh = center_wh + spec.tile();

        // Could also use a pair of opposite corners, though that makes things slightly less clear
        macro_rules! slices_from_corners {
            (
                $top_left: expr,
                $top_right: expr,
                $bottom_left: expr,
                $bottom_right: expr,
            ) => ({
                let top_left: sprite::XY<BaseUI> = $top_left;
                let top_right: sprite::XY<BaseUI> = $top_right;
                let bottom_left: sprite::XY<BaseUI> = $bottom_left;
                let bottom_right: sprite::XY<BaseUI> = $bottom_right;

                let middle: sprite::XY<BaseUI> = sprite::XY::<BaseUI> {
                    x: top_left.x + edge_wh.w,
                    y: top_left.y + edge_wh.h,
                };
                let top: sprite::XY<BaseUI> = sprite::XY::<BaseUI> {
                    x: middle.x,
                    y: top_left.y,
                };
                let left: sprite::XY<BaseUI> = sprite::XY::<BaseUI> {
                    x: top_left.x,
                    y: middle.y,
                };
                let right: sprite::XY<BaseUI> = sprite::XY::<BaseUI> {
                    x: top_right.x,
                    y: middle.y,
                };
                let bottom: sprite::XY<BaseUI> = sprite::XY::<BaseUI> {
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
            })
        }

        // TODO? Is it worth caching this spec math across frames?
        let talking_slices = slices_from_corners!(
            sprite::XY::<BaseUI> {
                x: sprite::x::<BaseUI>(0),
                y: sprite::y::<BaseUI>(0) + spec.tile().h,
            },
            sprite::XY::<BaseUI> {
                x: sprite::x::<BaseUI>(0) + edge_wh.w + center_wh.w,
                y: sprite::y::<BaseUI>(0) + spec.tile().h,
            },
            sprite::XY::<BaseUI> {
                x: sprite::x::<BaseUI>(0),
                y: sprite::y::<BaseUI>(0) + spec.tile().h + edge_wh.h + center_wh.h,
            },
            sprite::XY::<BaseUI> {
                x: sprite::x::<BaseUI>(0) + edge_wh.w + center_wh.w,
                y: sprite::y::<BaseUI>(0) + spec.tile().h + edge_wh.h + center_wh.h,
            },
        );

        let slices = match nine_slice_sprite & 0b11 {
            INVENTORY => slices_from_corners!(
                talking_slices.top_left + supertile_wh.h,
                talking_slices.top_right + supertile_wh.h,
                talking_slices.bottom_left + supertile_wh.h,
                talking_slices.bottom_right + supertile_wh.h,
            ),
            SELECTRUM => slices_from_corners!(
                talking_slices.top_left + supertile_wh.h * 2,
                talking_slices.top_right + supertile_wh.h * 2,
                talking_slices.bottom_left + supertile_wh.h * 2,
                talking_slices.bottom_right + supertile_wh.h * 2,
            ),
            CONTEXT_MENU => slices_from_corners!(
                talking_slices.top_left + supertile_wh.h * 3,
                talking_slices.top_right + supertile_wh.h * 3,
                talking_slices.bottom_left + supertile_wh.h * 3,
                talking_slices.bottom_right + supertile_wh.h * 3,
            ),
            _ => talking_slices,
        };

        let after_left_corner = x.saturating_add_w(edge_wh.w);
        let before_right_corner = x.saturating_add_w(w).saturating_sub_w(edge_wh.w);

        let below_top_corner = y.saturating_add_h(edge_wh.h);
        let above_bottom_corner = y.saturating_add_h(h).saturating_sub_h(edge_wh.h);

        // ABBBC
        // DEEEF
        // DEEEF
        // GHHHI

        // Draw E
        for fill_y in (below_top_corner.get()..above_bottom_corner.get()).step_by(center_wh.h.get() as _).map(unscaled::Y) {
            for fill_x in (after_left_corner.get()..before_right_corner.get()).step_by(center_wh.w.get() as _).map(unscaled::X) {
                commands.sspr(
                    slices.middle.apply(&commands.ui_spec),
                    unscaled::Rect {
                        x: fill_x,
                        y: fill_y,
                        // Clamp these values so we don't draw past the edge.
                        w: core::cmp::min(center_wh.w, before_right_corner - fill_x),
                        h: core::cmp::min(center_wh.h, above_bottom_corner - fill_y),
                    }
                );
            }
        }

        // Draw B and H
        for fill_x in (after_left_corner.get()..before_right_corner.get()).step_by(center_wh.w.get() as _).map(unscaled::X) {
            commands.sspr(
                slices.top.apply(&commands.ui_spec),
                unscaled::Rect {
                    x: fill_x,
                    y,
                    // Clamp this value so we don't draw past the edge.
                    w: core::cmp::min(center_wh.w, before_right_corner - fill_x),
                    h: edge_wh.h,
                }
            );

            commands.sspr(
                slices.bottom.apply(&commands.ui_spec),
                unscaled::Rect {
                    x: fill_x,
                    y: above_bottom_corner,
                    // Clamp this value so we don't draw past the edge.
                    w: core::cmp::min(center_wh.w, before_right_corner - fill_x),
                    h: edge_wh.h,
                }
            );
        }

        // Draw D and F
        for fill_y in (below_top_corner.get()..above_bottom_corner.get()).step_by(center_wh.h.get() as _).map(unscaled::Y) {
            commands.sspr(
                slices.left.apply(&commands.ui_spec),
                unscaled::Rect {
                    x,
                    y: fill_y,
                    // Clamp this value so we don't draw past the edge.
                    w: edge_wh.w,
                    h: core::cmp::min(center_wh.h, above_bottom_corner - fill_y),
                }
            );

            commands.sspr(
                slices.right.apply(&commands.ui_spec),
                unscaled::Rect {
                    x: before_right_corner,
                    y: fill_y,
                    // Clamp this value so we don't draw past the edge.
                    w: edge_wh.w,
                    h: core::cmp::min(center_wh.h, above_bottom_corner - fill_y),
                }
            );
        }

        // Draw A
        commands.sspr(
            slices.top_left.apply(&commands.ui_spec),
            unscaled::Rect {
                x,
                y,
                w: edge_wh.w,
                h: edge_wh.h,
            }
        );

        // Draw C
        commands.sspr(
            slices.top_right.apply(&commands.ui_spec),
            unscaled::Rect {
                x: before_right_corner,
                y,
                w: edge_wh.w,
                h: edge_wh.h,
            }
        );

        // Draw G
        commands.sspr(
            slices.bottom_left.apply(&commands.ui_spec),
            unscaled::Rect {
                x,
                y: above_bottom_corner,
                w: edge_wh.w,
                h: edge_wh.h,
            }
        );

        // Draw I
        commands.sspr(
            slices.bottom_right.apply(&commands.ui_spec),
            unscaled::Rect {
                x: before_right_corner,
                y: above_bottom_corner,
                w: edge_wh.w,
                h: edge_wh.h,
            }
        );
    }

    pub const fn inner_rect(edge_wh: unscaled::WH, outer_rect: unscaled::Rect) -> unscaled::Rect {
        unscaled::Rect {
            x: unscaled::x_const_add_w(outer_rect.x, edge_wh.w),
            y: unscaled::y_const_add_h(outer_rect.y, edge_wh.h),
            w: unscaled::w_const_sub(outer_rect.w, unscaled::w_const_mul(edge_wh.w, 2)),
            h: unscaled::h_const_sub(outer_rect.h, unscaled::h_const_mul(edge_wh.h, 2)),
        }
    }
}

pub const TEN_CHAR: u8 = 27;

pub const CLUB_CHAR: u8 = 31;
pub const DIAMOND_CHAR: u8 = 29;
pub const HEART_CHAR: u8 = 30;
pub const SPADE_CHAR: u8 = 28;

