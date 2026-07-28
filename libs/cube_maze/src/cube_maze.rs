use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use qrs::{QRS, QRSD, Q, R, qr};
use xs::{Seed, Xs};

#[derive(Clone, Debug, Default)]
struct Tiles; // placeholder

#[derive(Clone, Debug, Default)]
struct Mobs; // placeholder

#[derive(Clone, Debug, Default)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
    pub tiles: Tiles,
    pub mobs: Mobs,
}

impl State {
    pub fn new(rng: &mut Xs, specs: &sprite::Specs) -> Self {
        let seed = xs::new_seed(rng);

        Self::init(seed, specs)
    }

    fn init(seed: Seed, _specs: &sprite::Specs) -> Self {
        let mut rng_ = xs::from_seed(seed);
        let rng = &mut rng_;

        let tiles = Tiles::default();
        let mobs = Mobs::default();

        Self {
            seed,
            rng: rng_,
            tiles,
            mobs,
            .. <_>::default()
        }
    }

    #[allow(unused)]
    fn restart(&mut self, specs: &sprite::Specs) {
        *self = Self::init(self.seed, specs);
    }

    pub fn all_offsets_settled(&self) -> bool {
        false
    }

    pub fn is_complete(&self) -> bool {
        // If the animations are not settled, delay completion
        if !self.all_offsets_settled() {
            return false
        }

        false
    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        specs: &sprite::Specs,
        input: Input,
        speaker: &mut Speaker,
    ) {
        commands.print_line(b"cube maze", <_>::default(), 6);
    }
}
