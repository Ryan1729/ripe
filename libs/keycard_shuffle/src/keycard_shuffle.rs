use gfx::{Commands, AddDrawCommands};
use gfx_sizes::{ARGB};
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
//use vec1::{Grid1, Grid1Spec};
use xs::{Seed, Xs};

#[derive(Clone, Copy, Debug)]
enum CardColour {
    Blue,
    Green,
    Red,
    Yellow,
    Purple,
    Cyan,
}

impl CardColour {
    const ALL: [Self; 6] = [
        Self::Blue,
        Self::Green,
        Self::Red,
        Self::Yellow,
        Self::Purple,
        Self::Cyan,
    ];
}

#[derive(Clone, Copy, Debug)]
enum CardSymbol {
    None,
    OnePip,
}

impl CardSymbol {
    const ALL: [Self; 2] = [
        Self::None,
        Self::OnePip,
    ];
}

#[derive(Clone, Copy, Debug)]
struct CardKind {
    colour: CardColour,
    symbol: CardSymbol,
}

type Inventory = Vec<CardKind>;

#[derive(Clone, Debug, Default)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
    pub inventory: Inventory,
}

impl State {
    pub fn new(rng: &mut Xs, specs: &sprite::Specs) -> Self {
        let seed = xs::new_seed(rng);

        Self::init(seed, specs)
    }

    fn init(seed: Seed, _specs: &sprite::Specs) -> Self {
        let mut rng_ = xs::from_seed(seed);
        let rng = &mut rng_;

        let mut inventory = Vec::with_capacity(CardColour::ALL.len() * CardSymbol::ALL.len());

        for colour in CardColour::ALL {
            for symbol in CardSymbol::ALL {
                inventory.push(
                    CardKind { colour, symbol },
                );
            }
        }

        Self {
            seed,
            rng: rng_,
            inventory,
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
        mut commands: &mut Commands,
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

        use gfx::nine_slice;

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

        let card_wh = specs.keycard_shuffle_cards.tile();
        let letters_wh = specs.keycard_shuffle_letters.tile();
        let lights_wh = specs.keycard_shuffle_lights.tile();

        macro_rules! draw_card {
            ($xy: expr, $kind: expr) => ({
                draw_card!($xy, $kind, unscaled::X(command::WIDTH_SIGNED))
            });
            ($xy: expr, $kind: expr, $cutoff_x: expr) => ({
                draw_card!(@commands: &mut commands, $xy, $kind, $cutoff_x)
            });
            (@commands: $commands: expr, $xy: expr, $kind: expr) => ({
                draw_card!(@commands: $commands, $xy, $kind, unscaled::X(command::WIDTH_SIGNED))
            });
            (@commands: $commands: expr, $xy: expr, $kind: expr, $cutoff_x: expr) => ({
                let xy = $xy;
                let kind = $kind;
                let clip_rect = unscaled::Rect {
                    x: unscaled::X(0),
                    y: unscaled::Y(0),
                    w: $cutoff_x - unscaled::X(0),
                    h: unscaled::H::new(command::HEIGHT_SIGNED),
                };

                let colour_index: u16 = match kind.colour {
                    CardColour::Blue => 0,
                    CardColour::Green => 1,
                    CardColour::Red => 2,
                    CardColour::Yellow => 3,
                    CardColour::Purple => 4,
                    CardColour::Cyan => 5,
                };

                let mut cmds = $commands.clipped(clip_rect);

                // Card Back
                cmds.sspr_override(
                    specs.keycard_shuffle_cards.xy_from_tile_sprite(0u16),
                    specs.keycard_shuffle_cards.rect(xy),
                    PALETTE[usize::from(colour_index)],
                );

                // Card Stripe
                cmds.sspr(
                    specs.keycard_shuffle_cards.xy_from_tile_sprite(1u16),
                    specs.keycard_shuffle_cards.rect(xy),
                );

                // Colour Label
                let label_base_xy = xy + unscaled::H::new(20);

                cmds.sspr_override(
                    specs.keycard_shuffle_letters.xy_from_tile_sprite(colour_index),
                    specs.keycard_shuffle_letters.rect(label_base_xy),
                    PALETTE[6]
                );

                // Symbol (if any)
                match kind.symbol {
                    CardSymbol::None => {},
                    CardSymbol::OnePip => {
                        cmds.sspr_override(
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
            });
        }

        let xy = unscaled::XY {
            x: unscaled::X(0) + unscaled::W::new((command::WIDTH_SIGNED / 2) + 50),
            y: unscaled::Y(0) + unscaled::H::new(command::HEIGHT_SIGNED / 6),
        };

        // Render card slot back
        let slot_xy = unscaled::XY {
            x: unscaled::X(0) + unscaled::W::new((command::WIDTH_SIGNED / 8) * 5),
            y: xy.y - unscaled::H::new(4),
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

        // Render card slot overlay
        commands.sspr(slot_overlay_sprite_xy, slot_overlay_rect);

        // Render inventory

        let inventory_outer_rect = unscaled::Rect {
            x: unscaled::X(0),
            y: unscaled::Y(command::HEIGHT_SIGNED / 2),
            w: unscaled::W::new(command::WIDTH_SIGNED),
            h: unscaled::H::new(command::HEIGHT_SIGNED / 2),
        };

        commands.nine_slice(nine_slice::INVENTORY, inventory_outer_rect);

        let edge_wh = commands.ui_edge_wh();

        let inventory_inner_rect = nine_slice::inner_rect(edge_wh, inventory_outer_rect);

        let cell_wh = edge_wh + specs.keycard_shuffle_cards.tile() + edge_wh;

        let inventory_x_max = inventory_inner_rect.x + inventory_inner_rect.w;
        let inventory_y_max = inventory_inner_rect.y + inventory_inner_rect.h;

        // TODO store on the state
        let current_index = 0;

        let mut inventory_index = 0;

        let mut at = inventory_inner_rect.xy();

        let mut clipped_commands = commands.clipped(inventory_inner_rect);

        // TODO clip the inventory to the inner rect => add clipping feature to gfx::Commands
        //    I think maybe return a new thing with the same interface that clips to the given rect
        //    Use it for the card as well
        // TODO implement scrolling for the inventory
        while at.x < inventory_x_max && at.y < inventory_y_max {
            // draw selectrum
            if inventory_index == current_index {
                clipped_commands.nine_slice(
                    nine_slice::SELECTRUM,
                    unscaled::Rect {
                        x: at.x,
                        y: at.y,
                        w: cell_wh.w,
                        h: cell_wh.h,
                    },
                );
            }

            if let Some(card_kind) = self.inventory.get(inventory_index) {
                draw_card!(@commands: &mut clipped_commands, at + edge_wh, card_kind);
            };

            at.x += cell_wh.w;
            if at.x + cell_wh.w >= inventory_x_max {
                at.y += cell_wh.h;
                at.x = inventory_inner_rect.x;
            }
            inventory_index += 1;
        }

    }
}