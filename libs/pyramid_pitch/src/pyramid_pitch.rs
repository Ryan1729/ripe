#![deny(unused_variables)]

use gfx::{Commands, AddDrawCommands};
use gfx_sizes::{PALETTE};
use platform_types::{command, sprite, unscaled, Button, Dir, Input, Speaker};
use xs::{Seed, Xs};

#[derive(Clone, Debug)]
pub struct State {
    
}

impl State {
    pub fn new(_rng: &mut Xs, _specs: &sprite::Specs) -> Self {
        Self {
            
        }
    }

    fn all_offsets_settled(&self) -> bool {
        // TODO actual checking
        true
    }

    pub fn is_complete(&self) -> bool {
        // If the animations are not settled, delay completion
        if !self.all_offsets_settled() {
            return false
        }

        // TODO actual checking
        false
    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        specs: &sprite::Specs,
        _input: Input,
        _speaker: &mut Speaker,
    ) {
        //
        // Update
        //

        //
        // Render
        //

        let xy = unscaled::XY {
            x: unscaled::X(0) + unscaled::W::new((command::WIDTH_SIGNED / 8) * 3),
            y: unscaled::Y(0) + unscaled::H::new((command::HEIGHT_SIGNED / 8) * 3),
        };

        commands.sspr_override(
            specs.pyramid_pitch_tiles.xy_from_tile_sprite(0u16),
            specs.pyramid_pitch_tiles.rect(xy),
            PALETTE[1]
        );

        commands.sspr_override(
            specs.pyramid_pitch_tiles.xy_from_tile_sprite(1u16),
            specs.pyramid_pitch_tiles.rect(xy),
            PALETTE[0]
        );

        commands.sspr_override(
            specs.pyramid_pitch_tiles.xy_from_tile_sprite(2u16),
            specs.pyramid_pitch_tiles.rect(xy),
            PALETTE[4]
        );

        commands.sspr_override(
            specs.pyramid_pitch_tiles.xy_from_tile_sprite(3u16),
            specs.pyramid_pitch_tiles.rect(xy),
            PALETTE[4]
        );
    }
}