use gfx::{Commands};
//use gfx_sizes::ARGB;
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
//use vec1::{Grid1, Grid1Spec, vec1, Vec1};
use xs::{Seed, Xs};

#[derive(Clone, Debug)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
}

impl State {
    pub fn new(rng: &mut Xs, specs: &sprite::Specs) -> Self {
        let seed = xs::new_seed(rng);

        Self::init(seed, specs)
    }

    fn init(seed: Seed, specs: &sprite::Specs) -> Self {
        let mut rng_ = xs::from_seed(seed);
        let rng = &mut rng_;

        Self {
            seed,
            rng: rng_,
            //tiles,
            //mobs
        }
    }

    fn restart(&mut self, specs: &sprite::Specs) {
        *self = Self::init(self.seed, specs);
    }

    pub fn is_complete(&self) -> bool {
        false
    }

    fn tick(&mut self) {

    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        specs: &sprite::Specs,
        input: Input,
        _speaker: &mut Speaker,
    ) {
        //
        //
        // Update Section
        //
        //

        self.tick();

        //
        //
        // Render Section
        //
        //

        commands.sspr(
            specs.hex_twiddle_tiles.xy_from_tile_sprite(0u16),
            command::Rect::from_unscaled(specs.hex_twiddle_tiles.rect(<_>::default())),
        );
    }
}
