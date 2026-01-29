use gfx::{Commands};
use platform_types::{sprite, Input, Speaker};
use xs::Xs;

#[derive(Clone, Debug)]
pub struct State {
    pub state: game::State,
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
        spec: &sprite::Spec::<sprite::IcePuzzles>,
        input: Input,
        speaker: &mut Speaker,
    ) {
        // TODO allow backing out in case the player wants to give up on the puzzle

        game::State::update_and_render(
            commands,
            spec,
            &mut self.state,
            input,
            speaker,
        );
    }
}