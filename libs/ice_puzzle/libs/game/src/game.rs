use common::*;
use gfx::{Commands};
use platform_types::{command, unscaled, Button, Input, Speaker, SFX};
use xs::{Xs, Seed};

use platform::Chars;

pub struct State {
    pub rng: Xs,
    pub state: common::State,
    platform: Platform,
    events: Vec<Event>,
}

const TILE_SIZE: unscaled::Inner = 20;

fn p_xy(commands: &mut Commands, x_in: i32, y_in: i32, s: &'static str) {
    use platform_types::{sprite};
    type X = unscaled::Inner;
    type Y = unscaled::Inner;

    assert_eq!(s.chars().count(), 1, "{s}");

    match (X::try_from(x_in), Y::try_from(y_in)) {
        (Ok(x), Ok(y)) => {
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
                "R" => (1 * TILE_SIZE, 1 * TILE_SIZE),
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

            commands.sspr(
                sprite::XY {
                    // + 128 to put us at the start of the spritesheet section for this sub-game
                    x: sprite::X(sx + 128),
                    y: sprite::Y(sy),
                },
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
        let rng = xs::from_seed(seed);

        State {
            rng,
            state: state_manipulation::new_state(platform::size()),
            platform: Platform {
                p_xy,
                print_xy: platform::print_xy,
                clear: platform::clear,
                size: platform::size,
                pick: platform::pick,
                mouse_position: platform::mouse_position,
                clicks: platform::clicks,
                key_pressed: platform::key_pressed,
                set_colors: platform::set_colors,
                get_colors: platform::get_colors,
                set_foreground: platform::set_foreground,
                get_foreground: platform::get_foreground,
                set_background: platform::set_background,
                get_background: platform::get_background,
                set_layer: platform::set_layer,
                get_layer: platform::get_layer,
            },
            events: Vec::with_capacity(1),
        }
    }

    pub fn update_and_render(
        commands: &mut Commands,
        state: &mut State,
        input: Input,
        speaker: &mut Speaker,
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

        let ignored = state_manipulation::update_and_render(
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
    use platform_types::{sprite};
    use std::{
        collections::HashMap,
        sync::{Mutex, OnceLock}
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

    pub fn print_xy(x_in: i32, y_in: i32, s: &'static str) {
        assert_eq!(s.chars().count(), 1, "{s}");

        match (X::try_from(x_in), Y::try_from(y_in)) {
            (Ok(x), Ok(y)) => {
                { state!().chars.insert((x, y), s); }
                {
                    let c: &Chars = &(state!().chars);
                    eprintln!("{:p} print_xy {}", c, c.len());
                }
            },
            _ => {
                assert!(false, "bad (x, y): ({x_in}, {y_in})");
            }
        }
    }
    pub fn clear(rect: Option<Rect>) {

    }
    pub fn size() -> Size {
        Size::new(24, 16)
    }
    pub fn pick(point: Point, _: i32) -> char {
        '\0'
    }
    pub fn mouse_position() -> Point {
        Point::default()
    }
    pub fn clicks() -> i32 {
        0
    }
    pub fn key_pressed(key: KeyCode) -> bool {
        false
    }
    pub fn set_colors(foreground: Color, background: Color) {
        
    }
    pub fn get_colors() -> (Color, Color) {
        (
            Color { red: 255, green: 0, blue: 255, alpha: 255 },
            Color { red: 255, green: 0, blue: 255, alpha: 255 },
        )
    }
    pub fn set_foreground(foreground: Color) {

    }
    pub fn get_foreground() -> (Color) {
        get_colors().0
    }
    pub fn set_background(background: Color) {

    }
    pub fn get_background() -> (Color) {
        get_colors().1
    }
    pub fn set_layer(layer: i32) {

    }
    pub fn get_layer() -> i32 {
        0
    }

    /// `platform` state management
    pub fn push_commands(commands: &mut Commands) {
        for ((x, y), s) in state!().chars.iter() {
            let (sx, sy) = match *s {
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
                "@" => (3 * TILE_SIZE, 1 * TILE_SIZE),
                "R" => (4 * TILE_SIZE, 1 * TILE_SIZE),
                "↑" => (3 * TILE_SIZE, 2 * TILE_SIZE),
                "←" => (4 * TILE_SIZE, 2 * TILE_SIZE),
                "↓" => (5 * TILE_SIZE, 2 * TILE_SIZE),
                "→" => (6 * TILE_SIZE, 2 * TILE_SIZE),
                "┌" => (3 * TILE_SIZE, 3 * TILE_SIZE),
                "─" => (4 * TILE_SIZE, 3 * TILE_SIZE),
                "╖" => (5 * TILE_SIZE, 3 * TILE_SIZE),
                "│" => (6 * TILE_SIZE, 3 * TILE_SIZE),
                "╘" => (7 * TILE_SIZE, 3 * TILE_SIZE),
                "┘" => (8 * TILE_SIZE, 3 * TILE_SIZE),
                "╔" => (3 * TILE_SIZE, 4 * TILE_SIZE),
                "═" => (4 * TILE_SIZE, 4 * TILE_SIZE),
                "╕" => (5 * TILE_SIZE, 4 * TILE_SIZE),
                "║" => (6 * TILE_SIZE, 4 * TILE_SIZE),
                "╙" => (7 * TILE_SIZE, 4 * TILE_SIZE),
                "╝" => (8 * TILE_SIZE, 4 * TILE_SIZE),
                _ => {
                    debug_assert!(false, "unknown tile str: \"{s}\"");
                    (0, 0)
                }
            };
            dbg!(*s);
            commands.sspr(
                sprite::XY {
                    x: sprite::X(sx),
                    y: sprite::Y(sy),
                },
                command::Rect::from_unscaled(unscaled::Rect {
                    x: unscaled::X((x * TILE_SIZE) as _),
                    y: unscaled::Y((y * TILE_SIZE) as _),
                    w: unscaled::W(TILE_SIZE),
                    h: unscaled::H(TILE_SIZE),
                })
            );
        }
        //eprintln!("{:p} post push_commands {}", &state!().chars, &state!().chars.len());
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