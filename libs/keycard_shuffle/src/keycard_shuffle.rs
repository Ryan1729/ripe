use gfx::{Commands, AddDrawCommands};
use gfx_sizes::{ARGB};
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
//use vec1::{Grid1, Grid1Spec};
use xs::{Seed, Xs};

type Index = usize;

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
    TwoPips,
}

impl CardSymbol {
    const ALL: [Self; 3] = [
        Self::None,
        Self::OnePip,
        Self::TwoPips,
    ];
}

#[derive(Clone, Copy, Debug)]
struct CardKind {
    colour: CardColour,
    symbol: CardSymbol,
}

#[derive(Clone, Debug, Default)]
struct Inventory {
    cells: Vec<CardKind>,
    index: Index,
}

#[derive(Clone, Debug, Default)]
struct Lock {
    // TODO state for lights and whether is unlocked
}

#[derive(Clone, Debug, Default)]
struct Locks {
    locks: Vec<Lock>,
    index: Index,
}

type FrameCount = u16;

const MAX_INSERT_FRAME: FrameCount = 60;
const MAX_INSIDE_FRAME: FrameCount = 20;
const MAX_REMOVE_FRAME: FrameCount = 60;

#[derive(Clone, Debug)]
enum LockAnimationState {
    Insert(FrameCount),
    Inside(FrameCount),
    Remove(FrameCount),
}

impl Default for LockAnimationState {
    fn default() -> Self { LockAnimationState::Insert(0) }
}

#[derive(Clone, Debug, Default)]
struct LockAnimation {
    state: LockAnimationState,
    inventory_index: Index,
    lock_index: Index,
}

#[derive(Clone, Debug, Default)]
struct Animations {
    lock: Option<LockAnimation>,
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
    pub inventory: Inventory,
    pub inventory_scroll: unscaled::XYD,
    pub locks: Locks,
    pub animations: Animations,
}

impl State {
    pub fn new(rng: &mut Xs, specs: &sprite::Specs) -> Self {
        let seed = xs::new_seed(rng);

        Self::init(seed, specs)
    }

    fn init(seed: Seed, _specs: &sprite::Specs) -> Self {
        let mut rng_ = xs::from_seed(seed);
        let rng = &mut rng_;

        let mut inventory = Inventory {
            cells: Vec::with_capacity(CardColour::ALL.len() * CardSymbol::ALL.len()),
            index: 0,
        };

        for colour in CardColour::ALL {
            for symbol in CardSymbol::ALL {
                inventory.cells.push(
                    CardKind { colour, symbol },
                );
            }
        }

        let mut locks = Locks::default();
        locks.locks.push(Lock {});

        Self {
            seed,
            rng: rng_,
            inventory,
            locks,
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
        // Advance animations
        // We can make an iterator if we actually need at least 3 distinct animations that are handled the same.
        if let Some(animation) = &mut self.animations.lock {
            match &mut animation.state {
                LockAnimationState::Insert(at_frame) => {
                    *at_frame += 1;
                    if *at_frame > MAX_INSERT_FRAME {
                        // TODO Good place for click sound effect
                        animation.state = LockAnimationState::Inside(0);
                    }
                },
                LockAnimationState::Inside(at_frame) => {
                    *at_frame += 1;
                    if *at_frame > MAX_INSIDE_FRAME {
                        // TODO Good place for click sound effect
                        animation.state = LockAnimationState::Remove(0);
                    }
                },
                LockAnimationState::Remove(at_frame) => {
                    *at_frame += 1;
                    if *at_frame > MAX_REMOVE_FRAME {
                        self.animations.lock = None;
                    }
                },
            };
        }
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

        let inventory_outer_rect = unscaled::Rect {
            x: unscaled::X(0),
            y: unscaled::Y(command::HEIGHT_SIGNED / 2),
            w: unscaled::W::new(command::WIDTH_SIGNED),
            h: unscaled::H::new(command::HEIGHT_SIGNED / 2),
        };

        let edge_wh = commands.ui_edge_wh();

        let inventory_cell_wh = edge_wh + specs.keycard_shuffle_cards.tile() + edge_wh;

        let inventory_inner_rect = nine_slice::inner_rect(edge_wh, inventory_outer_rect);

        let inventory_x_max = inventory_inner_rect.x + inventory_inner_rect.w;
        let inventory_y_max = inventory_inner_rect.y + inventory_inner_rect.h;

        // debug scrolling
        if cfg!(debug_asserttions) && input.gamepad.contains(Button::B) {
            if let Some(dir) = input.dir_pressed_this_frame() {
                match dir {
                    Dir::Up => {
                        self.inventory_scroll.yd -= unscaled::YD(1);
                    }
                    Dir::Down => {
                        self.inventory_scroll.yd += unscaled::YD(1);
                    }
                    Dir::Left => {
                        
                    }
                    Dir::Right => {
                        
                    }
                }
            }
        } else if let Some(dir) = input.dir_pressed_this_frame() {
            let inventory_cells_wide_count = usize::from(inventory_inner_rect.w / inventory_cell_wh.w.get());

            match dir {
                Dir::Up => {
                    if self.inventory.index >= inventory_cells_wide_count {
                        self.inventory.index -= inventory_cells_wide_count;
                    }
                }
                Dir::Down => {
                    if self.inventory.index + inventory_cells_wide_count < self.inventory.cells.len() {
                        self.inventory.index += inventory_cells_wide_count;
                    }
                }
                Dir::Left => {
                    if self.inventory.index > 0 {
                        self.inventory.index -= 1;
                    }
                }
                Dir::Right => {
                    if self.inventory.index < self.inventory.cells.len() - 1 {
                        self.inventory.index += 1;
                    }
                }
            }

            // This is a version of the render loop, done here to find
            // If we ever have a third place we do this, then combine them

            let mut inventory_render_index = 0;

            let mut at = inventory_inner_rect.xy();

            while inventory_render_index < self.inventory.cells.len() {
                // draw selectrum
                if inventory_render_index == self.inventory.index {
                    let selected_at = unscaled::Rect {
                        x: at.x,
                        y: at.y,
                        w: inventory_cell_wh.w,
                        h: inventory_cell_wh.h,
                    } + self.inventory_scroll;

                    // If the top of the card is above the clip rect, adjust scroll so that it is in view, at the top
                    if selected_at.y < inventory_inner_rect.y {
                        self.inventory_scroll.yd += inventory_cell_wh.h.into();
                    }

                    // If the bottom of the card is below the clip rect, adjust scroll so that it is in view, at the bottom
                    if selected_at.y + selected_at.h > inventory_y_max {
                        self.inventory_scroll.yd -= inventory_cell_wh.h.into();
                    }

                    break
                }

                at.x += inventory_cell_wh.w;
                if at.x + inventory_cell_wh.w >= inventory_x_max {
                    at.y += inventory_cell_wh.h;
                    at.x = inventory_inner_rect.x;
                }
                inventory_render_index += 1;
            }
        } else if input.pressed_this_frame(Button::A) {
            if self.animations.lock.is_none() {
                if let (Some(_), Some(_)) = (
                    self.inventory.cells.get(self.inventory.index),
                    self.locks.locks.get(self.locks.index),
                ) {
                    self.animations.lock = Some(
                        LockAnimation{
                            state: <_>::default(),
                            inventory_index: self.inventory.index,
                            lock_index: self.locks.index,
                        }
                    );
                }
            }
        }

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

        let slot_sprite_xy = specs.keycard_shuffle_slot.xy_from_tile_sprite(0u16);

        let card_y = unscaled::Y(0) + unscaled::H::new(command::HEIGHT_SIGNED / 6);

        let slot_xy = unscaled::XY {
            x: unscaled::X(0) + unscaled::W::new((command::WIDTH_SIGNED / 8) * 7),
            y: card_y - unscaled::H::new(4),
        };

        let slot_rect = specs.keycard_shuffle_slot.rect(slot_xy);

        let card_x_min = unscaled::X(0) + unscaled::W::new(((command::WIDTH_SIGNED / 4) * 3) - 40);
        let card_x_max = slot_rect.x + unscaled::W::new(2);

        macro_rules! draw_pip_at {
            ($xy: expr) => {
                draw_pip_at!(@commands: &mut commands, $xy);
            };
            (@commands: $commands: expr, $xy: expr) => {
                $commands.sspr_override(
                    specs.keycard_shuffle_lights.xy_from_tile_sprite(2u16),
                    specs.keycard_shuffle_lights.rect($xy),
                    PALETTE[6]
                );
            }
        }

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
                        draw_pip_at!(
                            @commands: cmds,
                            label_base_xy
                                + letters_wh.w + lights_wh.w.halve()
                                + letters_wh.h.halve() - lights_wh.h.halve()
                        );
                    },
                    CardSymbol::TwoPips => {
                        let one_pip_xy = label_base_xy
                            + letters_wh.w + lights_wh.w.halve()
                            + letters_wh.h.halve() - lights_wh.h.halve();

                        draw_pip_at!(@commands: cmds, one_pip_xy);

                        draw_pip_at!(
                            @commands: cmds,
                            one_pip_xy + lights_wh.w + lights_wh.w.halve()
                        );
                    },
                };
            });
        }

        // Render lock scene

        const SPACING: unscaled::Inner = 4;

        let lock_scene_rect = unscaled::Rect {
            x: unscaled::X(0),
            y: unscaled::Y(0),
            w: (card_x_min - unscaled::X(0)) - unscaled::W::new(SPACING),
            h: (inventory_outer_rect.y - unscaled::Y(0)) - unscaled::H::new(SPACING),
        };

        commands.nine_slice(nine_slice::CONTEXT_MENU, lock_scene_rect);

        // Render card slot back

        // TODO Render lock lights

        commands.sspr(slot_sprite_xy, slot_rect);

        let slot_overlay_x_shift = slot_rect.w.halve().inc();

        let mut slot_overlay_sprite_xy = slot_sprite_xy;
        slot_overlay_sprite_xy.x += slot_overlay_x_shift;

        let mut slot_overlay_rect = slot_rect;
        slot_overlay_rect.x += slot_overlay_x_shift;
        slot_overlay_rect.w -= slot_overlay_x_shift;

        match self.animations.lock {
            Some(LockAnimation{ ref state, inventory_index, .. }) => {
                if let Some(item) = self.inventory.cells.get(inventory_index) {
                    let x = match state {
                        LockAnimationState::Insert(frame_count) => {
                            let fraction = (*frame_count) as f32 / MAX_INSERT_FRAME as f32;
                            unscaled::X(unscaled::lerp(card_x_min.0, fraction, card_x_max.0))
                        },
                        LockAnimationState::Inside(_) => card_x_max,
                        LockAnimationState::Remove(frame_count) => {
                            let fraction = (MAX_REMOVE_FRAME - (*frame_count)) as f32 / MAX_REMOVE_FRAME as f32;
                            unscaled::X(unscaled::lerp(card_x_min.0, fraction, card_x_max.0))
                        },
                    };

                    let xy = unscaled::XY {
                        x,
                        y: card_y,
                    };
    
                    draw_card!(xy, item, slot_overlay_rect.x);
                }
            }
            None => {}
        }

        // Render card slot overlay
        commands.sspr(slot_overlay_sprite_xy, slot_overlay_rect);

        // Render inventory

        commands.nine_slice(nine_slice::INVENTORY, inventory_outer_rect);

        let mut inventory_render_index = 0;

        let mut at = inventory_inner_rect.xy();

        let mut clipped_commands = commands.clipped(inventory_inner_rect);

        while inventory_render_index < self.inventory.cells.len() {
            // draw selectrum
            if inventory_render_index == self.inventory.index {
                clipped_commands.nine_slice(
                    nine_slice::SELECTRUM,
                    unscaled::Rect {
                        x: at.x,
                        y: at.y,
                        w: inventory_cell_wh.w,
                        h: inventory_cell_wh.h,
                    } + self.inventory_scroll,
                );
            }

            if let Some(card_kind) = self.inventory.cells.get(inventory_render_index) {
                draw_card!(@commands: &mut clipped_commands, at + edge_wh + self.inventory_scroll, card_kind);
            };

            at.x += inventory_cell_wh.w;
            if at.x + inventory_cell_wh.w >= inventory_x_max {
                at.y += inventory_cell_wh.h;
                at.x = inventory_inner_rect.x;
            }
            inventory_render_index += 1;
        }
    }
}