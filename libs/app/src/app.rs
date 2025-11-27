use gfx::{Commands, nine_slice, next_arrow, speech};
use platform_types::{command, sprite, unscaled, Button, Input, Speaker, SFX};
pub use platform_types::StateParams;
use game::{Dir, Mode, TalkingState, to_tile};
use models::{Entity, Speeches, XY, i_to_xy, TileSprite, Speech};

#[derive(Debug)]
pub enum Error {
    Config(config::Error),
    Game(game::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Error::*;
        match self {
            Config(error) => write!(f, "Config Error: {error:#?}"),
            Game(error) => write!(f, "Game Error: {error:#?}"),
        }
    }
}

pub struct ErrorState {
    pub error: Error,
    pub show_are_you_sure: bool,
}

impl From<Error> for ErrorState {
    fn from(error: Error) -> Self {
        Self {
            error,
            show_are_you_sure: <_>::default(),
        }
    }
}

type GameState = Result<game::State, ErrorState>;

pub struct State {
    pub game_state: GameState,
    pub commands: Commands,
    pub input: Input,
    pub speaker: Speaker,
    // Retained for resarting in error scenarios
    pub params: StateParams,
}

impl State {
    pub fn new(params: StateParams) -> Self {
        unsafe {
            features::GLOBAL_LOGGER = params.logger;
            features::GLOBAL_ERROR_LOGGER = params.error_logger;
        }
        let seed = params.seed;

        // We always want to log the seed, if there is a logger available, so use the function,
        // not the macro.
        features::log(&format!("{:?}", seed));

        const HARDCODED_CONFIG: &str = r#"
        import "tile_flags" as TF;
        const W = TF::WALL;
        const F = TF::FLOOR | TF::PLAYER_START;
        const B = TF::FLOOR; // Bare Floor
        const I = TF::FLOOR | TF::ITEM_START | TF::NPC_START;
        
        import "entity_flags" as EF;
        
        const MOB = 0;
        const ITEM = EF::COLLECTABLE;
        
        import "default_spritesheet" as DS;
        
        #{
            segments: [
                #{
                    width: 7,
                    tiles: [
                        F, F, F, F, F, F, F,
                        F, W, W, F, W, W, F,
                        F, W, B, I, B, W, F,
                        F, F, I, W, I, F, F,
                        F, W, B, I, B, W, F,
                        F, W, W, I, W, W, F,
                        F, F, F, F, F, F, F,
                    ],
                },
            ],
            entities: [
                #{
                    flags: MOB,
                    speeches: [ "hey can you get me something that's at least probably cool?" ],
                    tile_sprite: DS::mob(0),
                },
                #{
                    flags: ITEM,
                    inventory_description: [ "a chest, probably with something cool in it.", "can't seem to open it, so it'll stay at least probably cool forever." ],
                    tile_sprite: DS::item(0),
                },
                #{
                    flags: MOB,
                    speeches: [ "I lost my bayer-dollars! Can you help me find them?", "I don't know where I lost them. I'm looking over here because the light is better." ],
                    tile_sprite: DS::mob(1),
                },
                #{
                    flags: ITEM,
                    inventory_description: [ "some bayer-dollars. you can tell because of the pattern in the middle." ],
                    tile_sprite: DS::item(1),
                },
            ],
        }
        "#;

        let override_config = params.config_loader.and_then(|f| f());

        let game_state = 'game_state: {            
            let config = match config::parse(override_config.as_ref().map_or(HARDCODED_CONFIG, |s| s)) {
                Ok(c) => c,
                Err(err) => {
                    break 'game_state Err(Error::Config(err))
                }
            };
    
            game::State::new(seed, config).map_err(Error::Game)
        }.map_err(ErrorState::from);

        Self {
            game_state,
            // This doesn't have to use the same seed, but there's currently no reason not to.
            commands: Commands::new(seed),
            input: Input::default(),
            speaker: Speaker::default(),
            params,
        }
    }
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub fn frame(state: &mut State) -> (&[platform_types::Command], &[SFX]) {
    let mut shake_amount_fallback = 0;
    let shake_amount = match &mut state.game_state {
        Ok(s) => &mut s.shake_amount,
        Err(_) => &mut shake_amount_fallback
    };

    state.commands.begin_frame(shake_amount);
    state.speaker.clear();
    let effect = update_and_render(
        &mut state.commands,
        &mut state.game_state,
        state.input,
        &mut state.speaker,
    );

    match effect {
        Effect::NoOp => {},
        Effect::Reload => {
            *state = State::new(state.params);
        },
    }

    state.input.previous_gamepad = state.input.gamepad;

    (state.commands.slice(), state.speaker.slice())
}

pub fn press(state: &mut State, button: Button) {
    if state.input.previous_gamepad.contains(button) {
        //This is meant to pass along the key repeat, if any.
        //Not sure if rewriting history is the best way to do this.
        state.input.previous_gamepad.remove(button);
    }

    state.input.gamepad.insert(button);
}

pub fn release(state: &mut State, button: Button) {
    state.input.gamepad.remove(button);
}

const INVENTORY_WIDTH_CELLS: usize = 18;
const INVENTORY_HEIGHT_CELLS: usize = 8;
const INVENTORY_MAX_INDEX: usize = (INVENTORY_WIDTH_CELLS * INVENTORY_HEIGHT_CELLS) - 1;

fn game_update(state: &mut game::State, input: Input, _speaker: &mut Speaker) {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum TalkingUpdateState {
        StillTalking,
        Finished,
    }
    use TalkingUpdateState::*;

    fn talking_update(
        talking: &mut TalkingState,
        speeches: &Speeches,
        input: Input,
    ) -> TalkingUpdateState {
        let mut output = StillTalking;

        if game::get_speech(speeches, talking.key, talking.speech_index).is_none() {
            output = Finished;
        }

        if input.pressed_this_frame(Button::A)
        || input.pressed_this_frame(Button::B) {
            talking.speech_index += 1;
            return output
        }

        platform_types::arrow_timer::tick(&mut talking.arrow_timer);

        output
    }

    state.tick();

    match &mut state.mode {
        Mode::Walking => {
            if input.pressed_this_frame(Button::START) {
                state.mode = Mode::Inventory { 
                    current_index: <_>::default(),
                    last_dir: <_>::default(),
                    dir_count: <_>::default(),
                    description_talking: <_>::default(),
                };
                return
            }
        
            if input.pressed_this_frame(Button::UP) {
                state.walk(Dir::Up);
            } else if input.pressed_this_frame(Button::DOWN) {
                state.walk(Dir::Down);
            } else if input.pressed_this_frame(Button::LEFT) {
                state.walk(Dir::Left);
            } else if input.pressed_this_frame(Button::RIGHT) {
                state.walk(Dir::Right);
            } else {
                // Nothing to do
            };
        
            if input.pressed_this_frame(Button::A) {
                if input.gamepad.contains(Button::UP) {
                    state.interact(Dir::Up)
                } else if input.gamepad.contains(Button::DOWN) {
                    state.interact(Dir::Down)
                } else if input.gamepad.contains(Button::LEFT) {
                    state.interact(Dir::Left)
                } else if input.gamepad.contains(Button::RIGHT) {
                    state.interact(Dir::Right)
                }
            }
        },
        Mode::Inventory {
            current_index,
            last_dir,
            dir_count,
            description_talking,
        } => {
            if input.pressed_this_frame(Button::START) {
                state.mode = Mode::Walking;
                return
            }

            if let Some(talking) = description_talking {
                if talking_update(talking, &state.inventory_descriptions, input) == Finished {
                    *description_talking = None;
                }

                return
            } 

            if input.pressed_this_frame(Button::A) {
                if let Some(item) = state.player_inventory.get(*current_index) {
                    *description_talking = Some(TalkingState::new(item.speeches_key()));
                }
            } else if input.gamepad.contains(Button::UP) {
                if *last_dir == Some(Dir::Up) {
                    *dir_count = dir_count.saturating_add(1);
                } else {
                    *dir_count = 0;
                }
                *last_dir = Some(Dir::Up);
            } else if input.gamepad.contains(Button::DOWN) {
                if *last_dir == Some(Dir::Down) {
                    *dir_count = dir_count.saturating_add(1);
                } else {
                    *dir_count = 0;
                }
                *last_dir = Some(Dir::Down);
            } else if input.gamepad.contains(Button::LEFT) {
                if *last_dir == Some(Dir::Left) {
                    *dir_count = dir_count.saturating_add(1);
                } else {
                    *dir_count = 0;
                }
                *last_dir = Some(Dir::Left);
            } else if input.gamepad.contains(Button::RIGHT) {
                if *last_dir == Some(Dir::Right) {
                    *dir_count = dir_count.saturating_add(1);
                } else {
                    *dir_count = 0;
                }
                *last_dir = Some(Dir::Right);
            } else {
                *last_dir = None;
                *dir_count = 0;
            }

            if *dir_count > 8 || *dir_count == 0 {
                if *last_dir == Some(Dir::Up) {
                    if *current_index >= INVENTORY_WIDTH_CELLS {
                        *current_index -= INVENTORY_WIDTH_CELLS;
                    }
                } else if *last_dir == Some(Dir::Down) {
                    if *current_index + INVENTORY_WIDTH_CELLS <= INVENTORY_MAX_INDEX {
                        *current_index += INVENTORY_WIDTH_CELLS;
                    }
                } else if *last_dir == Some(Dir::Left) {
                    if *current_index > 0
                    && *current_index / INVENTORY_WIDTH_CELLS == (*current_index - 1) / INVENTORY_WIDTH_CELLS {
                        *current_index -= 1;
                    }
                } else if *last_dir == Some(Dir::Right) {
                    if *current_index + 1 <= INVENTORY_MAX_INDEX
                    && *current_index / INVENTORY_WIDTH_CELLS == (*current_index + 1) / INVENTORY_WIDTH_CELLS {
                        *current_index += 1;
                    }
                }
            }
        },
        Mode::Talking(talking) => {
            if talking_update(talking, &state.speeches, input) == Finished {
                state.mode = Mode::Walking;
            }
        },
    }
}



#[inline]
fn game_render(commands: &mut Commands, state: &game::State) {
    //
    // Render World
    //

    fn draw_talking(commands: &mut Commands, speeches: &Speeches, talking: &TalkingState) {
        commands.nine_slice(nine_slice::TALKING, speech::OUTER_RECT);

        if let Some(speech) = game::get_speech(speeches, talking.key, talking.speech_index) {
            commands.speech(speech);
        }

        commands.next_arrow_in_corner_of(next_arrow::TALKING, talking.arrow_timer, speech::INNER_RECT);
    }

    fn draw_tile(commands: &mut Commands, xy: XY, sprite: TileSprite) {
        draw_tile_sprite(commands, to_tile::min_corner(xy), sprite);
    }

    fn draw_tile_sprite(commands: &mut Commands, xy: unscaled::XY, sprite: TileSprite) {
        commands.sspr(
            to_tile::sprite_xy(sprite),
            command::Rect::from_unscaled(to_tile::rect(xy)),
        );
    }

    for i in 0..state.world.segment.tiles.len() {
        draw_tile(
            commands,
            i_to_xy(state.world.segment.width, i),
            state.world.segment.tiles[i].sprite,
        );
    }

    fn draw_entity(commands: &mut Commands, entity: &Entity) {
        commands.sspr(
            to_tile::sprite_xy(entity.sprite),
            command::Rect::from_unscaled(to_tile::entity_rect(entity)),
        );
    }

    for (_, item) in state.world.items.for_id(state.world.segment_id) {
        draw_entity(commands, item);
    }

    for (_, mob) in state.world.mobs.for_id(state.world.segment_id) {
        draw_entity(commands, mob);
    }

    draw_entity(commands, &state.world.player);

    for message in &state.fade_messages {
        commands.print_lines(
            message.xy,
            0,
            message.text.as_bytes(),
            6,
        );
    }

    //
    // Conditional rendering
    //

    match &state.mode {
        Mode::Walking => {
            // Nothing yet
        },
        Mode::Inventory {
            current_index,
            description_talking,
            ..
        } => {
            const SPACING: unscaled::Inner = 20;

            let outer_rect = unscaled::Rect {
                x: unscaled::X(SPACING),
                y: unscaled::Y(SPACING),
                w: unscaled::W(platform_types::command::WIDTH - (SPACING * 2)),
                h: unscaled::H(platform_types::command::HEIGHT - 120),
            };

            const CELL_W: unscaled::W = unscaled::W(24);
            const CELL_H: unscaled::H = unscaled::H(24);

            const CELL_INSET: unscaled::WH = unscaled::WH{
                w: unscaled::W(4),
                h: unscaled::H(4),
            };

            commands.nine_slice(nine_slice::INVENTORY, outer_rect);

            let inner_rect = nine_slice::inner_rect(outer_rect);

            let mut inventory_index = 0;

            let mut at = unscaled::XY { x: inner_rect.x, y: inner_rect.y };

            let x_max = inner_rect.x + inner_rect.w;
            let y_max = inner_rect.y + inner_rect.h;

            while at.x < x_max && at.y < y_max {
                // draw selectrum
                if inventory_index == *current_index {
                    commands.sspr(
                        sprite::XY {
                            x: sprite::X(24),
                            y: sprite::Y(8),
                        },
                        command::Rect::from_unscaled(
                            unscaled::Rect {
                                x: at.x,
                                y: at.y,
                                w: CELL_W,
                                h: CELL_H,
                            }
                        ),
                    );
                }

                if let Some(item) = state.player_inventory.get(inventory_index) {
                    draw_tile_sprite(commands, at + CELL_INSET, item.sprite);
                };

                at.x += CELL_W;
                if at.x >= x_max {
                    at.y += CELL_H;
                    at.x = inner_rect.x;
                }
                inventory_index += 1;
            }

            if let Some(talking) = description_talking {
                draw_talking(commands, &state.inventory_descriptions, talking);
            }
        },
        Mode::Talking(talking) => {
            draw_talking(commands, &state.speeches, talking);
        },
    }
}

#[derive(Default)]
enum Effect {
    #[default]
    NoOp,
    Reload,
}

#[inline]
fn update_and_render(
    commands: &mut Commands,
    game_state: &mut GameState,
    input: Input,
    speaker: &mut Speaker,
) -> Effect {
    match game_state {
        Ok(state) => {
            game_update(state, input, speaker);
            game_render(commands, state);

            <_>::default()
        },
        Err(err_state) => {
            let effect = err_update(err_state, input, speaker);
            err_render(commands, err_state);

            effect
        }
    }

}

#[inline]
fn err_update(
    error_state: &mut ErrorState,
    input: Input,
    _speaker: &mut Speaker,
) -> Effect {
    let mut effect = <_>::default();

    if error_state.show_are_you_sure {
        if input.pressed_this_frame(Button::A) {
            effect = Effect::Reload;
        } else if input.pressed_this_frame(Button::B) {
            error_state.show_are_you_sure = false;
        }
    } else {
        if input.pressed_this_frame(Button::A) {
            error_state.show_are_you_sure = true;
        }
    }

    effect
}

#[inline]
fn err_render(commands: &mut Commands, error_state: &ErrorState) {
    let error = &error_state.error;
    let width = 64;
    for i in 0..(width * width) {
        let x = unscaled::X(((i % width) * 16) as _);
        let y = unscaled::Y(((i / width) * 16) as _);
        let sprite = models::FLOOR_SPRITE as sprite::Inner;

        commands.sspr(
            sprite::XY {
                x: sprite::X(sprite * 16),
                y: sprite::Y(64),
            },
            command::Rect::from_unscaled(unscaled::Rect {
                x,
                y,
                w: unscaled::W(16),
                h: unscaled::H(16),
            })
        );
    }

    // TODO allow scrolling the text by allowing changing this.
    let top_index_with_offset = 0;

    // TODO? Maybe cache this so we aren't allocating every frame?
    let error_text = format!("{error}").to_lowercase();

    commands.print_lines(
        <_>::default(),
        top_index_with_offset,
        error_text.as_bytes(),
        6,
    );

    if error_state.show_are_you_sure {
        commands.nine_slice(nine_slice::TALKING, speech::OUTER_RECT);

        // TODO? Maybe cache this so we aren't allocating every frame?
        commands.speech(
            &Speech::from("Are you sure you want to try reloading?\n\n(A) to confirm, (B) to back out.")
        );
    }
}