///! B.O.L.D.
/// Boldly Or Leisurely Dashing
/// or
/// Boulders Often Lope Downwards

use gfx::{Commands};
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use xs::Xs;

#[derive(Clone, Debug)]
pub struct State {

}

impl State {
    pub fn new(rng: &mut Xs) -> Self {
        Self {
            
        }
    }

    pub fn is_complete(&self) -> bool {
        false
    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        bold_spec: &sprite::Spec::<sprite::BOLD>,
        input: Input,
        _speaker: &mut Speaker,
    ) {
        commands.sspr(
            bold_spec.xy_from_tile_sprite(0u16),
            command::Rect::from_unscaled(bold_spec.rect(<_>::default())),
        );
    }
}