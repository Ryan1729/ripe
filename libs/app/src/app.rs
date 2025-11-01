use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Input, Speaker, SFX};
pub use platform_types::StateParams;
use game::{Dir, Mode};
use models::{Entity, XY, i_to_xy};

#[derive(Debug)]
pub enum Error {
    Game(game::Error),
}

type GameState = Result<game::State, Error>;

pub struct State {
    pub game_state: GameState,
    pub commands: Commands,
    pub input: Input,
    pub speaker: Speaker,
}

impl State {
    pub fn new((seed, logger, error_logger): StateParams) -> Self {
        unsafe {
            features::GLOBAL_LOGGER = logger;
            features::GLOBAL_ERROR_LOGGER = error_logger;
        }

        // We always want to log the seed, if there is a logger available, so use the function,
        // not the macro.
        features::log(&format!("{:?}", seed));

        const HARDCODED_CONFIG: &str = r#"
        import "tile_flags" as TF;
        const W = TF::WALL;
        const F = TF::FLOOR | TF::PLAYER_START;
        const B = TF::FLOOR; // Bare Floor
        const I = TF::FLOOR | TF::ITEM_START | TF::NPC_START;

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
                    ]
                }
            ]
        }
        "#;

        // TODO: Should this error bubble up instead? Or maybe have the app display an error message?
        let config = match config::parse(HARDCODED_CONFIG) {
            Ok(c) => c,
            Err(err) => {
                features::log(&format!("{:?}", err));

                game::Config::default()
            }
        };

        let game_state = game::State::new(seed, config)
            .map_err(Error::Game);

        Self {
            game_state,
            commands: Commands::default(),
            input: Input::default(),
            speaker: Speaker::default(),
        }
    }
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub fn frame(state: &mut State) -> (&[platform_types::Command], &[SFX]) {
    state.commands.clear();
    state.speaker.clear();
    update_and_render(
        &mut state.commands,
        &mut state.game_state,
        state.input,
        &mut state.speaker,
    );

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

fn game_update(state: &mut game::State, input: Input, _speaker: &mut Speaker) {
    match &mut state.mode {
        Mode::Walking => {
            if input.pressed_this_frame(Button::START) {
                state.mode = Mode::Inventory {};
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
        Mode::Inventory {} => {
            if input.pressed_this_frame(Button::START) {
                state.mode = Mode::Walking;
                return
            }
        },
    }
}

const TILE_W: unscaled::W = unscaled::W(16);
const TILE_H: unscaled::H = unscaled::H(16);

fn tile_xy_to_rect(xy: XY) -> command::Rect {
    let x = unscaled::X(xy.x.get() * TILE_W.get());
    let y = unscaled::Y(xy.y.get() * TILE_H.get());

    command::Rect::from_unscaled(unscaled::Rect {
        x: x.saturating_add(TILE_W),
        y: y.saturating_add(TILE_H),
        w: TILE_W,
        h: TILE_H,
    })
}

/// Where the tiles start on the spreadsheet.
const TILES_Y: sprite::Y = sprite::Y(64);

#[inline]
fn game_render(commands: &mut Commands, state: &game::State) {
    //
    // Render World
    //

    for i in 0..state.world.segment.tiles.len() {
        let sprite = state.world.segment.tiles[i].sprite as sprite::Inner;

        commands.sspr(
            sprite::XY {
                x: sprite::X(sprite * TILE_W.get()),
                y: TILES_Y,
            },
            tile_xy_to_rect(i_to_xy(state.world.segment.width, i)),
        );
    }

    // TODO make a convenient way to draw a tile/sprite at tile boundaries, since we do it multiple times.
    fn draw_entity(commands: &mut Commands, entity: &Entity) {
        let sprite = entity.sprite as sprite::Inner;

        commands.sspr(
            sprite::XY {
                x: sprite::X(sprite * TILE_W.get()),
                y: TILES_Y,
            },
            tile_xy_to_rect(entity.xy())
        );
    }

    for (_, item) in state.world.items.for_id(state.segment_id) {
        draw_entity(commands, item);
    }

    for (_, mob) in state.world.mobs.for_id(state.segment_id) {
        draw_entity(commands, mob);
    }

    draw_entity(commands, &state.player);

    //
    // Conditional rendering
    //

    match state.mode {
        Mode::Walking => {
            
        },
        Mode::Inventory {} => {
            const SPACING: unscaled::Inner = 20;

            commands.nine_slice(
                unscaled::X(SPACING),
                unscaled::Y(SPACING),
                unscaled::W(platform_types::command::WIDTH - (SPACING * 2)),
                unscaled::H(platform_types::command::HEIGHT - 120),
            );
        },
    }
}

#[inline]
fn update_and_render(
    commands: &mut Commands,
    game_state: &mut GameState,
    input: Input,
    speaker: &mut Speaker,
) {
    match game_state {
        Ok(state) => {
            game_update(state, input, speaker);
            game_render(commands, state);
        },
        Err(err) => {
            // TODO? A way to restart within the app?
            err_render(commands, err);
        }
    }

}

#[inline]
fn err_render(commands: &mut Commands, error: &Error) {
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
                x: x.saturating_add(unscaled::W(16)),
                y: y.saturating_add(unscaled::H(16)),
                w: unscaled::W(16),
                h: unscaled::H(16),
            })
        );
    }

    // TODO allow scrolling the text by allowing changing this.
    let top_index_with_offset = 0;

    commands.print_lines(
        <_>::default(),
        top_index_with_offset,
        // TODO? Maybe cache this so we aren't allocating every frame?
        format!("{error:?}").as_bytes(),
        6,
    );
}