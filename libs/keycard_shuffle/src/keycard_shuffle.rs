use gfx::{Commands};
use gfx_sizes::{ARGB};
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
//use vec1::{Grid1, Grid1Spec};
use xs::{Seed, Xs};

#[derive(Clone, Debug, Default)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
}

impl State {
    pub fn new(rng: &mut Xs, specs: &sprite::Specs) -> Self {
        let seed = xs::new_seed(rng);

        Self::init(seed, specs)
    }

    fn init(seed: Seed, _specs: &sprite::Specs) -> Self {
        let mut rng_ = xs::from_seed(seed);
        let rng = &mut rng_;

        Self {
            seed,
            rng: rng_,
            .. <_>::default()
        }
    }

    #[allow(unused)]
    fn restart(&mut self, specs: &sprite::Specs) {
        *self = Self::init(self.seed, specs);
    }

    fn all_offsets_settled(&self) -> bool {
        true
    }

    pub fn is_complete(&self) -> bool {
        // If the animations are not settled, delay completion
        if !self.all_offsets_settled() {
            return false
        }

        // TODO actual checking
        false
    }

    fn tick(&mut self) {

    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        specs: &sprite::Specs,
        input: Input,
        _speaker: &mut Speaker,
    ) {
        //
        // Update
        //

        if input.pressed_this_frame(Button::START) {
            self.restart(specs);
        }

        self.tick();

        //
        // Render
        //

        const PALETTE: [ARGB; 8] = [
            0xFF3352E1, // Blue
            0xFF30B06E, // Green
            0xFFDE4949, // Red
            0xFFFFB937, // Yellow
            0xFF533354, // Purple
            0xFF5A7D8B, // Cyan/Grey
            0xFFEEEEEE, // White
            0xFF222222, // Black
        ];

        enum CardColour {
            Red,
        }

        enum CardSymbol {
            None,
            OnePip,
        }

        struct CardKind {
            colour: CardColour,
            symbol: CardSymbol,
        }

        let card_wh = specs.keycard_shuffle_cards.tile();
        let letters_wh = specs.keycard_shuffle_letters.tile();
        let lights_wh = specs.keycard_shuffle_lights.tile();

        macro_rules! draw_card {
            ($xy: expr, $kind: expr) => ({
                draw_card!($xy, $kind, unscaled::X(command::WIDTH_SIGNED))
            });
            ($xy: expr, $kind: expr, $cutoff_x: expr) => ({
                let xy = $xy;
                let kind = $kind;
                let clip_rect = unscaled::Rect {
                    x: unscaled::X(0),
                    y: unscaled::Y(0),
                    w: $cutoff_x - unscaled::X(0),
                    h: unscaled::H::new(command::HEIGHT_SIGNED),
                };

                let colour_index: u16 = match kind.colour {
                    CardColour::Red => 2,
                };

                // Card Back
                commands.sspr_override(
                    specs.keycard_shuffle_cards.xy_from_tile_sprite(0u16),
                    specs.keycard_shuffle_cards.rect(xy).clip(clip_rect),
                    PALETTE[usize::from(colour_index)],
                );

                // Card Stripe
                commands.sspr(
                    specs.keycard_shuffle_cards.xy_from_tile_sprite(1u16),
                    specs.keycard_shuffle_cards.rect(xy).clip(clip_rect),
                );

                // Colour Label
                let label_base_xy = xy + unscaled::H::new(20);

                commands.sspr_override(
                    specs.keycard_shuffle_letters.xy_from_tile_sprite(colour_index),
                    specs.keycard_shuffle_letters.rect(label_base_xy).clip(clip_rect),
                    PALETTE[6]
                );

                // Symbol (if any)
                match kind.symbol {
                    CardSymbol::None => {},
                    CardSymbol::OnePip => {
                        commands.sspr_override(
                            specs.keycard_shuffle_lights.xy_from_tile_sprite(2u16),
                            specs.keycard_shuffle_lights.rect(
                                label_base_xy
                                + letters_wh.w + lights_wh.w.halve()
                                + letters_wh.h.halve() - lights_wh.h.halve()
                            ).clip(clip_rect),
                            PALETTE[6]
                        );
                    },
                };
            })
        }

        let xy = unscaled::XY {
            x: unscaled::X(0) + unscaled::W::new((command::WIDTH_SIGNED / 2) + 50),
            y: unscaled::Y(0) + unscaled::H::new(command::HEIGHT_SIGNED / 3),
        };

        // Render card slot back
        let slot_xy = unscaled::XY {
            x: unscaled::X(0) + unscaled::W::new((command::WIDTH_SIGNED / 8) * 5),
            y: unscaled::Y(102),
        };

        let slot_sprite_xy = specs.keycard_shuffle_slot.xy_from_tile_sprite(0u16);

        let slot_rect = specs.keycard_shuffle_slot.rect(slot_xy);

        commands.sspr(slot_sprite_xy, slot_rect);

        let slot_overlay_x_shift = slot_rect.w.halve().inc();

        let mut slot_overlay_sprite_xy = slot_sprite_xy;
        slot_overlay_sprite_xy.x += slot_overlay_x_shift;

        let mut slot_overlay_rect = slot_rect;
        slot_overlay_rect.x += slot_overlay_x_shift;
        slot_overlay_rect.w -= slot_overlay_x_shift;

        draw_card!(xy, CardKind { colour: CardColour::Red, symbol: CardSymbol::OnePip }, slot_overlay_rect.x);
        draw_card!(xy + card_wh.h + card_wh.h / 2, CardKind { colour: CardColour::Red, symbol: CardSymbol::OnePip });


        // Render card slot overlay
        commands.sspr(slot_overlay_sprite_xy, slot_overlay_rect);
    }
}