use common::*;
use gfx::{Commands};
use platform_types::{command, sprite::{self, IcePuzzles}, unscaled::{self, H, W}, Button, Input, Speaker};
use xs::{Seed};

#[derive(Clone, Debug)]
pub struct State {
    pub state: common::State,
    platform: Platform,
    events: Vec<Event>,
}

fn str_to_sprite_xy(spec: &sprite::Spec::<IcePuzzles>, s: &str) -> sprite::XY<IcePuzzles> {
    let tile = spec.tile();
    let tile_w = tile.w;
    let tile_h = tile.h;
    let (w, h) = match s {
        "☐" => (W(0), H(0)),
        "☒" => (W(0), 1 * tile_h),
        "\u{E010}" => (W(0), 2 * tile_h),
        "\u{E011}" => (W(0), 3 * tile_h),
        "\u{E012}" => (W(0), 4 * tile_h),
        "\u{E013}" => (W(0), 5 * tile_h),
        "\u{E014}" => (W(0), 6 * tile_h),
        "\u{E015}" => (W(0), 7 * tile_h),
        "\u{E016}" => (W(0), 8 * tile_h),
        "\u{E017}" => (W(0), 9 * tile_h),
        "\u{E018}" => (W(0), 10 * tile_h),
        "@" => (1 * tile_w, 0 * tile_h),
        "#" => (1 * tile_w, 1 * tile_h),
        "$" => (1 * tile_w, 2 * tile_h),
        "%" => (1 * tile_w, 3 * tile_h),
        //"R" => (4 * tile_w, 1 * tile_h),
        //"↑" => (0 * tile_w, 2 * tile_h),
        //"←" => (1 * tile_w, 2 * tile_h),
        //"↓" => (2 * tile_w, 2 * tile_h),
        //"→" => (3 * tile_w, 2 * tile_h),
        //"┌" => (0 * tile_w, 3 * tile_h),
        //"─" => (1 * tile_w, 3 * tile_h),
        //"╖" => (2 * tile_w, 3 * tile_h),
        //"│" => (3 * tile_w, 3 * tile_h),
        //"╘" => (4 * tile_w, 3 * tile_h),
        //"┘" => (5 * tile_w, 3 * tile_h),
        //"╔" => (0 * tile_w, 4 * tile_h),
        //"═" => (1 * tile_w, 4 * tile_h),
        //"╕" => (2 * tile_w, 4 * tile_h),
        //"║" => (3 * tile_w, 4 * tile_h),
        //"╙" => (4 * tile_w, 4 * tile_h),
        //"╝" => (5 * tile_w, 4 * tile_h),
        _ => {
            debug_assert!(false, "unknown tile str: \"{s}\"");
            (W(0), H(0))
        }
    };

    sprite::XY {
        x: sprite::x::<IcePuzzles>(0) + w,
        y: sprite::y::<IcePuzzles>(0) + h,
    }
}

fn p_xy(commands: &mut Commands, spec: &sprite::Spec::<IcePuzzles>, x_in: i32, y_in: i32, s: &'static str) {
    type X = unscaled::Inner;
    type Y = unscaled::Inner;

    assert_eq!(s.chars().count(), 1, "{s}");

    match (X::try_from(x_in), Y::try_from(y_in)) {
        (Ok(x), Ok(y)) => {
            let tile = spec.tile();
            let w = tile.w;
            let h = tile.h;

            commands.sspr(
                str_to_sprite_xy(spec, s).apply(spec),
                command::Rect::from_unscaled(unscaled::Rect {
                    x: unscaled::X(x * w.get()),
                    y: unscaled::Y(y * h.get()),
                    w,
                    h,
                })
            );
        },
        _ => {
            assert!(false, "bad (x, y): ({x_in}, {y_in})");
        }
    }
}

impl State {
    pub fn new(
        seed: Seed,
        spec: &sprite::Spec<IcePuzzles>,
    ) -> State {
        State {
            state: state_manipulation::new_state(
                platform::size(spec),
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
        spec: &sprite::Spec<IcePuzzles>,
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
            spec,
            &state.platform,
            &mut state.state,
            &mut state.events
        );

        platform::push_commands(commands, spec);

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
    pub fn size(spec: &sprite::Spec::<IcePuzzles>) -> Size {
        let tile = spec.tile();
        let tile_w = tile.w;
        let tile_h = tile.h;
        
        Size::new(
            (command::WIDTH / tile_w.get()).into(),
            (command::HEIGHT / tile_h.get()).into(),
        )
    }

    /// `platform` state management
    pub fn push_commands(commands: &mut Commands, spec: &sprite::Spec::<IcePuzzles>) {
        let tile = spec.tile();
        let tile_w = tile.w;
        let tile_h = tile.h;
        for ((x, y), s) in state!().chars.iter() {
            commands.sspr(
                str_to_sprite_xy(spec, s).apply(spec),
                command::Rect::from_unscaled(unscaled::Rect {
                    x: unscaled::X(*x * tile_w.get()),
                    y: unscaled::Y(*y * tile_h.get()),
                    w: tile_w,
                    h: tile_h,
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
    let spec = Specs::default().ice_puzzles;
    let input = <_>::default();
    let mut speaker = <_>::default();

    assert!(commands.slice().len() <= 0, "precondition failure");

    State::update_and_render(
        &mut commands,
        &spec,
        &mut state,
        input,
        &mut speaker,
    );

    assert!(commands.slice().len() > 0, "{:#?}", commands.slice());
}