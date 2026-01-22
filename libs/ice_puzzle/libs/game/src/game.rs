use common::*;
use gfx::{Commands};
use platform_types::{command, sprite::{self, IcePuzzles}, unscaled, Button, Input, Speaker};
use xs::{Seed};

pub struct State {
    pub state: common::State,
    platform: Platform,
    events: Vec<Event>,
}

const TILE_SIZE: unscaled::Inner = 20;

fn str_to_sprite_xy(s: &str) -> sprite::XY<sprite::Renderable> {
    let (sx, sy) = match s {
        "☐" => (0, 0),
        "☒" => (1 * TILE_SIZE, 0),
        "\u{E010}" => (2 * TILE_SIZE, 0),
        "\u{E011}" => (3 * TILE_SIZE, 0),
        "\u{E012}" => (4 * TILE_SIZE, 0),
        "\u{E013}" => (5 * TILE_SIZE, 0),
        "\u{E014}" => (6 * TILE_SIZE, 0),
        "\u{E015}" => (7 * TILE_SIZE, 0),
        "\u{E016}" => (8 * TILE_SIZE, 0),
        "\u{E017}" => (9 * TILE_SIZE, 0),
        "\u{E018}" => (10 * TILE_SIZE, 0),
        "@" => (0 * TILE_SIZE, 1 * TILE_SIZE),
        "#" => (1 * TILE_SIZE, 1 * TILE_SIZE),
        "$" => (2 * TILE_SIZE, 1 * TILE_SIZE),
        "%" => (3 * TILE_SIZE, 1 * TILE_SIZE),
        "R" => (4 * TILE_SIZE, 1 * TILE_SIZE),
        "↑" => (0 * TILE_SIZE, 2 * TILE_SIZE),
        "←" => (1 * TILE_SIZE, 2 * TILE_SIZE),
        "↓" => (2 * TILE_SIZE, 2 * TILE_SIZE),
        "→" => (3 * TILE_SIZE, 2 * TILE_SIZE),
        "┌" => (0 * TILE_SIZE, 3 * TILE_SIZE),
        "─" => (1 * TILE_SIZE, 3 * TILE_SIZE),
        "╖" => (2 * TILE_SIZE, 3 * TILE_SIZE),
        "│" => (3 * TILE_SIZE, 3 * TILE_SIZE),
        "╘" => (4 * TILE_SIZE, 3 * TILE_SIZE),
        "┘" => (5 * TILE_SIZE, 3 * TILE_SIZE),
        "╔" => (0 * TILE_SIZE, 4 * TILE_SIZE),
        "═" => (1 * TILE_SIZE, 4 * TILE_SIZE),
        "╕" => (2 * TILE_SIZE, 4 * TILE_SIZE),
        "║" => (3 * TILE_SIZE, 4 * TILE_SIZE),
        "╙" => (4 * TILE_SIZE, 4 * TILE_SIZE),
        "╝" => (5 * TILE_SIZE, 4 * TILE_SIZE),
        _ => {
            debug_assert!(false, "unknown tile str: \"{s}\"");
            (0, 0)
        }
    };

    // TODO make this a parameter that ultimately comes from the config file.
    // + 128 to put us at the start of the spritesheet section for this sub-game
    let spec = sprite::spec::<IcePuzzles>(sprite::WH{ w: sprite::W(128), h: sprite::H(0) });

    sprite::XY {
        x: sprite::x::<IcePuzzles>(sx),
        y: sprite::y::<IcePuzzles>(sy),
    }.apply(&spec)
}

fn p_xy(commands: &mut Commands, x_in: i32, y_in: i32, s: &'static str) {
    type X = unscaled::Inner;
    type Y = unscaled::Inner;

    assert_eq!(s.chars().count(), 1, "{s}");

    match (X::try_from(x_in), Y::try_from(y_in)) {
        (Ok(x), Ok(y)) => {
            commands.sspr(
                str_to_sprite_xy(s),
                command::Rect::from_unscaled(unscaled::Rect {
                    x: unscaled::X((x * TILE_SIZE) as _),
                    y: unscaled::Y((y * TILE_SIZE) as _),
                    w: unscaled::W(TILE_SIZE),
                    h: unscaled::H(TILE_SIZE),
                })
            );
        },
        _ => {
            assert!(false, "bad (x, y): ({x_in}, {y_in})");
        }
    }
}

impl State {
    pub fn new(seed: Seed) -> State {
        State {
            state: state_manipulation::new_state(
                platform::size(),
                seed,
            ),
            platform: Platform {
                p_xy,
                size: platform::size,
            },
            events: Vec::with_capacity(1),
        }
    }

    pub fn update_and_render(
        commands: &mut Commands,
        state: &mut State,
        input: Input,
        _speaker: &mut Speaker,
    ) {
        state.events.clear();

        for button in Button::ALL {
            macro_rules! button_to_key {
                ($button: ident) => {
                    match $button {
                        Button::UP => KeyCode::Up,
                        Button::DOWN => KeyCode::Down,
                        Button::LEFT => KeyCode::Left,
                        Button::RIGHT => KeyCode::Right,
                        _ => KeyCode::R,
                    }
                }
            }

            if input.pressed_this_frame(button) {
                state.events.push(Event::KeyPressed {
                    key: button_to_key!(button),
                    ctrl: false,
                    shift: false,
                });
            }

            if input.released_this_frame(button) {
                state.events.push(Event::KeyReleased {
                    key: button_to_key!(button),
                    ctrl: false,
                    shift: false,
                });
            }
        }

        state_manipulation::update_and_render(
            commands,
            &state.platform,
            &mut state.state,
            &mut state.events
        );

        platform::push_commands(commands);

        platform::end_frame();
    }
}

mod platform {
    use super::*;
    use std::{
        collections::HashMap,
        sync::{Mutex}
    };

    type X = unscaled::Inner;
    type Y = unscaled::Inner;

    use std::hash::{BuildHasherDefault, DefaultHasher};

    pub type Chars = HashMap<(X, Y), &'static str, BuildHasherDefault<DefaultHasher>>;

    pub(crate) struct State {
        pub(crate) chars: Chars,
    }
    
    pub static STATE: Mutex<State> =
        Mutex::new(State{
            chars: HashMap::with_hasher(BuildHasherDefault::new()),
        });

    macro_rules! state {
        () => {
            STATE.lock().expect("should not be poisoned")
        }
    }

    /// `Platform` function pointers
    pub fn size() -> Size {
        Size::new(24, 16)
    }

    /// `platform` state management
    pub fn push_commands(commands: &mut Commands) {
        for ((x, y), s) in state!().chars.iter() {
            commands.sspr(
                str_to_sprite_xy(s),
                command::Rect::from_unscaled(unscaled::Rect {
                    x: unscaled::X((x * TILE_SIZE) as _),
                    y: unscaled::Y((y * TILE_SIZE) as _),
                    w: unscaled::W(TILE_SIZE),
                    h: unscaled::H(TILE_SIZE),
                })
            );
        }
    }
        
    pub fn end_frame() {
        state!().chars.clear();
    }
}

#[test]
fn something_gets_drawn() {
    let seed = <_>::default();

    let mut state = State::new(seed);

    let mut commands = Commands::new(seed);
    let input = <_>::default();
    let mut speaker = <_>::default();

    assert!(commands.slice().len() <= 0, "precondition failure");

    State::update_and_render(
        &mut commands,
        &mut state,
        input,
        &mut speaker,
    );

    assert!(commands.slice().len() > 0, "{:#?}", commands.slice());
}