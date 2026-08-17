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
                let xy = $xy;
                let kind = $kind;

                let colour_index: u16 = match kind.colour {
                    CardColour::Red => 2,
                };

                // Card Back
                commands.sspr_override(
                    specs.keycard_shuffle_cards.xy_from_tile_sprite(0u16),
                    specs.keycard_shuffle_cards.rect(xy),
                    PALETTE[usize::from(colour_index)],
                );
        
                // Card Stripe
                commands.sspr(
                    specs.keycard_shuffle_cards.xy_from_tile_sprite(1u16),
                    specs.keycard_shuffle_cards.rect(xy),
                );

                // Colour Label
                let label_base_xy = xy + unscaled::H::new(20);

                commands.sspr_override(
                    specs.keycard_shuffle_letters.xy_from_tile_sprite(colour_index),
                    specs.keycard_shuffle_letters.rect(label_base_xy),
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
                            ),
                            PALETTE[6]
                        );
                    },
                };
            })
        }

        let xy = unscaled::XY {
            x: unscaled::X(0) + unscaled::W::new(command::WIDTH as unscaled::Inner / 2),
            y: unscaled::Y(0) + unscaled::H::new(command::HEIGHT as unscaled::Inner / 3),
        };

        draw_card!(xy, CardKind { colour: CardColour::Red, symbol: CardSymbol::None });
        draw_card!(xy + card_wh.h + card_wh.h / 2, CardKind { colour: CardColour::Red, symbol: CardSymbol::OnePip });
    }
}