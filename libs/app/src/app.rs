use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Input, Speaker, SFX};
pub use platform_types::StateParams;

pub struct State {
    pub game_state: game::State,
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

        const HARDCODED_CONFIG: &str = "
        const W = 0; // Wall
        const F = 1; // Floor

        #{
            segments: [
                #{
                    width: 5,
                    tiles: [
                        W, W, F, W, W,
                        W, F, F, F, W,
                        F, F, W, F, F,
                        W, F, F, F, W,
                        W, W, F, W, W,
                    ]
                }
            ]
        }
        ";

        // TODO: Should this error bubble up instead? Or maybe have the app display an error message?
        let config = match config::parse(HARDCODED_CONFIG) {
            Ok(c) => c,
            Err(err) => {
                features::log(&format!("{:?}", err));

                game::Config::default()
            }
        };

        let mut game_state = game::State::new(seed, config);

        Self {
            game_state,
            commands: Commands::default(),
            input: Input::default(),
            speaker: Speaker::default(),
        }
    }
}

impl platform_types::State for State {
    fn frame(&mut self) -> (&[platform_types::Command], &[SFX]) {
        self.commands.clear();
        self.speaker.clear();
        update_and_render(
            &mut self.commands,
            &mut self.game_state,
            self.input,
            &mut self.speaker,
        );

        self.input.previous_gamepad = self.input.gamepad;

        (self.commands.slice(), self.speaker.slice())
    }

    fn press(&mut self, button: Button) {
        if self.input.previous_gamepad.contains(button) {
            //This is meant to pass along the key repeat, if any.
            //Not sure if rewriting history is the best way to do this.
            self.input.previous_gamepad.remove(button);
        }

        self.input.gamepad.insert(button);
    }

    fn release(&mut self, button: Button) {
        self.input.gamepad.remove(button);
    }
}

fn update(state: &mut game::State, input: Input, speaker: &mut Speaker) {
    if input.gamepad != <_>::default() {
        speaker.request_sfx(SFX::CardPlace);
    }
}

#[inline]
fn render(commands: &mut Commands, state: &game::State) {
    // TODO pull these 16s into named constant(s).
    for i in 0..state.world.segment.tiles.len() {
        let x = unscaled::X(((i % state.world.segment.width) * 16) as _);
        let y = unscaled::Y(((i / state.world.segment.width) * 16) as _);
        let sprite = state.world.segment.tiles[i].sprite as sprite::Inner;

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
}

#[inline]
fn update_and_render(
    commands: &mut Commands,
    state: &mut game::State,
    input: Input,
    speaker: &mut Speaker,
) {
    update(state, input, speaker);
    render(commands, state);
}
