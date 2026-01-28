use features::invariant_assert;
use gfx::{Commands, nine_slice, next_arrow, speech, to_tile};
use gfx_sizes::{ARGB, GFX_WIDTH};
use pak_types::{Specs};
use platform_types::{command, sprite::{self, BaseUI}, unscaled, Button, Dir, Input, PakReader, Speaker, SFX};
pub use platform_types::StateParams;
use game::{FadeMessageSpec, HallwayState, Mode, TalkingState, PostTalkingAction};
use models::{Entity, i_to_xy, Pak, Speech, Speeches, Spritesheet, TileSprite, XY};

#[derive(Debug)]
pub enum Error {
    Pak(pak::Error),
    Game(game::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Error::*;
        match self {
            Pak(error) => write!(f, "Pak Error:\n{error}"),
            Game(error) => write!(f, "Game Error:\n{error:#?}"),
        }
    }
}

#[derive(Debug)]
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
    pub specs: Specs,
    pub input: Input,
    pub speaker: Speaker,
    pub spritesheet: Spritesheet,
    // Retained for restarting in error scenarios
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

        const HARDCODED_CONFIG: &str = include_str!("../../../examples/default/config.rn");

        let get_hardcoded_spritesheet = || Spritesheet {
            pixels: assets::GFX.into(),
            width: GFX_WIDTH,
        };

        let pak_reader_opt: Option<Box<dyn PakReader>> = params.pak_loader.and_then(|f| f());

        let pak_result = match pak_reader_opt {
            Some(reader) => {
                pak::from_reader(reader)
                    .map_err(Error::Pak)
            },
            None => {
                // TODO Can we construct a Zip in code? Or maybe we should embed one at compile time?
                match config::parse(HARDCODED_CONFIG) {
                    Ok(config) => Ok(Pak {
                        config,
                        spritesheet: get_hardcoded_spritesheet(),
                        specs: Specs::default(),
                    }),
                    Err(err) => {
                        Err(Error::Pak(pak::Error::Config(err)))
                    }
                }
            },
        };

        let (game_state, spritesheet, specs) = match pak_result {
            Ok(pak) => (
                game::State::new(seed, pak.config)
                    .map_err(Error::Game)
                    .map_err(ErrorState::from),
                pak.spritesheet,
                pak.specs,
            ),
            Err(e) => (
                Err(ErrorState::from(e)),
                get_hardcoded_spritesheet(),
                Specs::default(),
            )
        };

        Self {
            game_state,
            // This doesn't have to use the same seed, but there's currently no reason not to.
            commands: Commands::new(seed, specs.base_font.clone(), specs.base_ui.clone()),
            specs,
            input: Input::default(),
            speaker: Speaker::default(),
            spritesheet,
            params,
        }
    }
}

#[cfg_attr(feature = "reload", unsafe(no_mangle))]
pub fn frame(state: &mut State) -> (&[platform_types::Command], (&[ARGB], usize), &[SFX]) {
    let mut shake_amount_fallback = 0;
    let shake_amount = match &mut state.game_state {
        Ok(s) => &mut s.shake_amount,
        Err(_) => &mut shake_amount_fallback
    };

    state.commands.begin_frame(shake_amount);
    state.speaker.clear();
    let effect = update_and_render(
        &mut state.commands,
        &state.specs,
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

    state.commands.end_frame();

    state.input.previous_gamepad = state.input.gamepad;

    (state.commands.slice(), state.spritesheet.slice(), state.speaker.slice())
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

const INVENTORY_WIDTH_CELLS: usize = 13;
const INVENTORY_HEIGHT_CELLS: usize = 8;
const INVENTORY_MAX_INDEX: usize = (INVENTORY_WIDTH_CELLS * INVENTORY_HEIGHT_CELLS) - 1;

fn game_update(commands: &mut Commands, state: &mut game::State, input: Input, speaker: &mut Speaker) {
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
        Mode::DoorTo(_target, _animation) => {
            // TODO? Allow cancelling going in the door?
        },
        Mode::Hallway{ source, target } => {
            match state.hallway_states.get_mut(*source, *target) {
                Some(HallwayState::IcePuzzle(ice_puzzle)) => {
                    ice_puzzle.update_and_render(
                        commands,
                        input,
                        speaker,
                    );
                },
                Some(HallwayState::SWORD(sword)) => {
                    sword.update_and_render(
                        commands,
                        input,
                        speaker,
                    );
                },
                None => {
                    invariant_assert!(false, "Hallway was not found while in Hallway mode!");
                    state.mode = Mode::Walking;
                }
            }
        }
        Mode::Victory(animation) => {
            // TODO? Allow cancelling going in the door?
            if animation.is_done() {
                // Do we want to do something else here? Back to a main menu?
                if input.pressed_this_frame(Button::START) {
                    *animation = <_>::default();
                }
            }
        },
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

            if let Some(dir) = input.dir_pressed_this_frame() {
                state.walk(dir);
            }

            if input.pressed_this_frame(Button::A) {
                if let Some(dir) = input.contains_dir() {
                    state.interact(dir)
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
                if let Some(item) = state.world.player.inventory.get(*current_index) {
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
            if let Finished = talking_update(talking, &state.speeches, input) {
                match talking.post_action {
                    PostTalkingAction::NoOp => {},
                    PostTalkingAction::TakeItem(receiveing_entity_key, def_id) => 'take_item: {
                        let giving_entity_key = state.world.player_key();
                        let Some(giving_entity_len) = state.world.get_entity(giving_entity_key)
                            .map(|g_e| g_e.inventory.len())
                        else {
                            break 'take_item
                        };

                        // TODO? Worth checking if it's not there?
                        // TODO? Do we want to give every entity an inventory, and preserve every item?
                        // Iterate backward so we can remove without indexing errors
                        for i in (0..giving_entity_len).rev() {
                            let Some(item_def_id) = state.world.get_entity(giving_entity_key)
                                .map(|g_e| g_e.inventory[i].transformable.id)
                            else {
                                break 'take_item
                            };

                            if item_def_id == def_id {
                                let Some(taken) = state.world.get_entity_mut(giving_entity_key)
                                    .map(|g_e| g_e.inventory.remove(i)) else {
                                        break 'take_item
                                    };

                                // This option is a bit less hassle than implmenting a way to get mut refs to
                                // two distinct entities.
                                let mut reward_opt = None;

                                if let Some(receiveing_entity) = state.world.get_entity_mut(receiveing_entity_key) {
                                    for desire in &mut receiveing_entity.transformable.wants {
                                        if desire.def_id == def_id {
                                            desire.state = models::DesireState::Satisfied;
                                            break
                                        }
                                    }

                                    // TODO? Allow associating particular desires with particular rewards
                                    //       from the same NPC?

                                    // Extract the reward before putting the taken item into the receiveing_entity's
                                    // inventory, so they don't give it back.
                                    if let Some(reward) = receiveing_entity.inventory.pop() {
                                        reward_opt = Some(reward);
                                    }

                                    receiveing_entity.inventory.push(taken);
                                } else {
                                    debug_assert!(false, "Why did the item get taken if no one wants it?!");
                                }

                                if let Some(reward) = reward_opt {
                                    state.push_inventory(giving_entity_key, reward);
                                }
                            }
                        }
                    },
                }

                state.mode = Mode::Walking;
            }
        },
    }
}



#[inline]
fn game_render(commands: &mut Commands, specs: &Specs, state: &game::State) {
    //
    // Render World
    //

    let draw_talking = |commands: &mut Commands, speeches: &Speeches, talking: &TalkingState| {
        commands.nine_slice(nine_slice::TALKING, speech::OUTER_RECT);

        if let Some(speech) = game::get_speech(speeches, talking.key, talking.speech_index) {
            commands.speech(speech);
        }

        commands.next_arrow_in_corner_of(next_arrow::TALKING, talking.arrow_timer, speech::INNER_RECT);
    };

    let draw_tile_sprite = |commands: &mut Commands, xy: unscaled::XY, sprite: TileSprite| {
        commands.sspr(
            to_tile::sprite_xy(&specs.base_tiles, sprite),
            command::Rect::from_unscaled(to_tile::rect(xy)),
        );
    };

    let draw_tile = |commands: &mut Commands, xy: XY, sprite: TileSprite| {
        draw_tile_sprite(commands, to_tile::min_corner(xy), sprite);
    };

    let draw_tile_sprite_centered_at = |commands: &mut Commands, xy: unscaled::XY, sprite: TileSprite| {
        commands.sspr(
            to_tile::sprite_xy(&specs.base_tiles, sprite),
            command::Rect::from_unscaled(to_tile::rect(to_tile::center_to_min_corner(xy))),
        );
    };

    let draw_entity = |commands: &mut Commands, entity: &Entity| {
        commands.sspr(
            to_tile::sprite_xy(&specs.base_tiles, entity.transformable.tile_sprite),
            command::Rect::from_unscaled(to_tile::entity_rect(entity)),
        );
    };

    let render_world = |commands: &mut Commands, state: &game::State| {
        let Some(segment) = state.world.segments.get(state.world.segment_id as usize) else {
            debug_assert!(false, "We somehow went to a non-existant segment?!");
            return
        };

        for i in 0..segment.tiles.len() {
            draw_tile(
                commands,
                i_to_xy(segment.width, i),
                segment.tiles[i].sprite,
            );
        }

        for (_, steppable) in state.world.steppables.for_id(state.world.segment_id) {
            draw_entity(commands, steppable);
        }

        for (_, mob) in state.world.mobs.for_id(state.world.segment_id) {
            draw_entity(commands, mob);
        }
    };

    let render_walking = |commands: &mut Commands, state: &game::State| {
        render_world(commands, state);

        draw_entity(commands, &state.world.player);
    };

    //
    // Conditional rendering
    //

    match &state.mode {
        Mode::Hallway { source, target } => {
            let source: &game::EntityKey = source;
            if let Some(_hallway) = state.hallway_states.get(*source, *target) {
                // The hallway is expected to be rendered elsewhere
            } else {
                commands.print_lines(
                    unscaled::XY {
                        x: unscaled::X(100),
                        y: unscaled::Y(50),
                    },
                    0,
                    // TODO Get this text from the config file
                    b"No hallway found!",
                    6,
                );
            }
        },
        Mode::DoorTo(_, animation) => {
            render_world(commands, state);

            if !animation.is_done() {
                draw_tile(commands, state.world.player.xy(), animation.sprite());
            }
        },
        Mode::Victory(animation) => {
            if animation.is_done() {
                commands.print_lines(
                    unscaled::XY {
                        x: unscaled::X(100),
                        y: unscaled::Y(50),
                    },
                    0,
                    // TODO Get this text from the config file
                    b"congraturation\n\nthis story is happy end",
                    6,
                );
                draw_tile_sprite(
                    commands,
                    // TODO Put this in the config file. Maybe allow defining any number of sprites to be drawn?
                    unscaled::XY {
                        x: unscaled::X(200),
                        y: unscaled::Y(150),
                    },
                    state.world.player.transformable.tile_sprite
                );
            } else {
                render_world(commands, state);
                draw_tile(commands, state.world.player.xy(), animation.sprite());
            }
        },
        Mode::Walking => {
            render_walking(commands, state);
        },
        Mode::Inventory {
            current_index,
            description_talking,
            ..
        } => {
            render_walking(commands, state);

            const SPACING: unscaled::Inner = 20;

            let menu_y = unscaled::Y(SPACING);
            let menu_h = unscaled::H(platform_types::command::HEIGHT - 120);

            let goal_outer_w = unscaled::W(120 - SPACING);

            let inv_outer_rect = unscaled::Rect {
                x: unscaled::X(SPACING),
                y: menu_y,
                w: unscaled::W(platform_types::command::WIDTH - (SPACING * 3)) - goal_outer_w,
                h: menu_h,
            };

            let goal_outer_rect = unscaled::Rect {
                x: inv_outer_rect.x + inv_outer_rect.w + unscaled::W(SPACING),
                y: menu_y,
                w: goal_outer_w,
                h: menu_h,
            };

            //
            //  Draw the goal description
            //

            commands.nine_slice(nine_slice::INVENTORY, goal_outer_rect);

            let goal_inner_rect = nine_slice::inner_rect(goal_outer_rect);

            commands.print_lines(
                goal_inner_rect.xy(),
                0,
                // TODO move this into the config.
                b"goal:\nfind the key\nto open this\ndoor.",
                6,
            );

            let image_xy = unscaled::XY {
                // TODO center this once we have the tile dimensions moved into the right spot
                x: goal_inner_rect.x + goal_inner_rect.w.halve(),
                y: goal_inner_rect.y + goal_inner_rect.h.halve(),
            };

            draw_tile_sprite_centered_at(commands, image_xy, state.goal_door_tile_sprite);


            //
            //  Draw the inventory
            //

            const CELL_W: unscaled::W = unscaled::W(24);
            const CELL_H: unscaled::H = unscaled::H(24);

            const CELL_INSET: unscaled::WH = unscaled::WH{
                w: unscaled::W(4),
                h: unscaled::H(4),
            };

            commands.nine_slice(nine_slice::INVENTORY, inv_outer_rect);

            let inv_inner_rect = nine_slice::inner_rect(inv_outer_rect);

            let mut inventory_index = 0;

            let mut at = unscaled::XY { x: inv_inner_rect.x, y: inv_inner_rect.y };

            let inv_x_max = inv_inner_rect.x + inv_inner_rect.w;
            let inv_y_max = inv_inner_rect.y + inv_inner_rect.h;

            while at.x < inv_x_max && at.y < inv_y_max {
                // draw selectrum
                if inventory_index == *current_index {
                    commands.sspr(
                        sprite::XY::<BaseUI> {
                            x: sprite::x::<BaseUI>(24),
                            y: sprite::y::<BaseUI>(8),
                        }.apply(&specs.base_ui),
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

                if let Some(item) = state.world.player.inventory.get(inventory_index) {
                    draw_tile_sprite(commands, at + CELL_INSET, item.transformable.tile_sprite);
                };

                at.x += CELL_W;
                if at.x >= inv_x_max {
                    at.y += CELL_H;
                    at.x = inv_inner_rect.x;
                }
                inventory_index += 1;
            }

            if let Some(talking) = description_talking {
                draw_talking(commands, &state.inventory_descriptions, talking);
            }
        },
        Mode::Talking(talking) => {
            render_walking(commands, state);

            draw_talking(commands, &state.speeches, talking);
        },
    }

    #[cfg(feature = "invariant-checking")]
    {
        match &state.mode {
            Mode::Walking => {
                commands.print_lines(
                    <_>::default(),
                    0,
                    format!(
                        "S:{} @:{},{}",
                        state.world.segment_id,
                        state.world.player.x.get(),
                        state.world.player.y.get(),
                    ).as_bytes(),
                    6,
                );
            },
            _ => {}
        }
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
    specs: &Specs,
    game_state: &mut GameState,
    input: Input,
    speaker: &mut Speaker,
) -> Effect {
    #[cfg(feature = "refresh")]
    {
        if input.pressed_this_frame(Button::RESET) {
            return Effect::Reload;
        }
    }

    match game_state {
        Ok(state) => {
            game_update(commands, state, input, speaker);
            // Empty message queue
            for FadeMessageSpec { message, xy } in state.fade_message_specs.drain(..) {
                commands.push_fade_message(message.into(), xy);
            }
            game_render(commands, specs, state);

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

#[test]
fn something_gets_drawn_for_ice_puzzle_alone() {
    let seed = <_>::default();

    let mut rng  = xs::from_seed(seed);

    let mut state = ice_puzzle::State::new(&mut rng);

    let mut commands = Commands::new(seed);
    let input = <_>::default();
    let mut speaker = <_>::default();

    assert!(commands.slice().len() <= 0, "precondition failure");

    state.update_and_render(
        &mut commands,
        input,
        &mut speaker,
    );

    assert!(commands.slice().len() > 0, "{:#?}", commands.slice());
}

#[test]
fn something_gets_drawn_for_ice_puzzle_within_app_state() {
    let seed = <_>::default();

    let params = StateParams {
        config_loader: None,
        logger: None,
        error_logger: None,
        seed,
    };

    let mut state = State::new(params);

    let source = <_>::default();
    let mut target = game::EntityKey::default();
    target.xy.x = models::xy::x(1);

    state.game_state.as_mut().expect("should not be in an error state").mode = Mode::Hallway { source, target };

    let mut rng = xs::from_seed(seed);

    state.game_state.as_mut().expect("should not be in an error state").hallway_states.insert(source, target, HallwayState::IcePuzzle(ice_puzzle::State::new(&mut rng)));

    assert!(state.commands.slice().len() <= 0, "precondition failure");

    frame(&mut state);

    // This asserts that the ice puzzle stuff showed up
    let mut count_of_20s = 0;
    let mut sizes = Vec::with_capacity(state.commands.slice().len());
    for command in state.commands.slice() {
        let w = (command.rect.x_max.get() - command.rect.x_min.get()).get();
        let h = (command.rect.y_max.get() - command.rect.y_min.get()).get();
        if w == 19 && h == 19 {
            count_of_20s += 1;
        }
        sizes.push((w, h));
    }

    assert!(sizes.len() > 0, "{:#?}", sizes);
    assert!(count_of_20s > 0, "{:#?}", sizes);
}

