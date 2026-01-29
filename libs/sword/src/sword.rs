use gfx::{Commands};
use platform_types::{sprite, Input, Speaker};
use xs::Xs;

#[derive(Clone, Debug)]
pub struct State {
    rng: Xs,
}

impl State {
    pub fn new(rng: &mut Xs) -> Self {
        let seed = xs::new_seed(rng);

        Self {
            rng: xs::from_seed(seed),
        }
    }

    pub fn is_complete(&self) -> bool {
        true
    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        spec: &sprite::Spec::<sprite::SWORD>,
        input: Input,
        speaker: &mut Speaker,
    ) {
        // TODO
    }
}
