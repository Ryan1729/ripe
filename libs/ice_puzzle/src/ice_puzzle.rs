use gfx::{Commands};
use platform_types::{command, unscaled, Button, Input, Speaker, SFX};
use xs::Xs;

//#[derive(Clone, Debug)]
pub struct State {
    pub count: u8, // Temp to just have something easy but visible
    pub state: game::State,
}

// See if we can get away with not modifying the code
impl Clone for State {
    fn clone(&self) -> Self {
        Self {
            count: 0,
            state: game::State::new(<_>::default()),
        }
    }
}
impl core::fmt::Debug for State {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ice_puzzle_game::State")
    }
}

impl State {
    pub fn new(rng: &mut Xs) -> Self {
        Self {
            count: 0,
            state: game::State::new(xs::new_seed(rng)),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.count == u8::MAX
    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        input: Input,
        speaker: &mut Speaker,
    ) {
        self.count = self.count.saturating_add(1);

        game::State::update_and_render(
            commands,
            &mut self.state,
            input,
            speaker,
        );
    }
}