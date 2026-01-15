use gfx::{Commands};
use platform_types::{Input, Speaker};
use xs::Xs;

//#[derive(Clone, Debug)]
pub struct State {
    pub state: game::State,
}

// See if we can get away with not modifying the code
impl Clone for State {
    fn clone(&self) -> Self {
        Self {
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
            state: game::State::new(xs::new_seed(rng)),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.state.state.max_steps >= 5
    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        input: Input,
        speaker: &mut Speaker,
    ) {
        // TODO allow backing out in case the palyer wants to give up on the puzzle

        game::State::update_and_render(
            commands,
            &mut self.state,
            input,
            speaker,
        );
    }
}