#![deny(unused_variables)]

use gfx::{Commands, AddDrawCommands};
use gfx_sizes::{ARGB, PALETTE};
use platform_types::{command, sprite, unscaled, Button, Dir, Input, Speaker};
use xs::{Seed, Xs};

mod board {
    pub type Inner = i16;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct X(pub Inner);
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Y(pub Inner);

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct XY {
        pub x: X,
        pub y: Y,
    }
}

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

        let xy = board::XY {
            x: board::X(0),
            y: board::Y(0),
        };

        fn board_to_unscaled(
            specs: &sprite::Specs,
            xy: board::XY,
        ) -> unscaled::XY {
            let board_wh = specs.pyramid_pitch_tiles.tile();

            // TODO accunt for isometric view

            unscaled::XY {
                x: unscaled::X((command::WIDTH_SIGNED / 8) * 3) + unscaled::W::new(xy.x.0 * board_wh.w.get()),
                y: unscaled::Y((command::HEIGHT_SIGNED / 8) * 3) + unscaled::H::new(xy.y.0 * board_wh.h.get()),
            }
        }

        struct TileSpec {
            xy: board::XY,
            top: ARGB,
            outline: ARGB,
            sides: ARGB,
        }

        fn draw_tile(
            cmds: &mut impl AddDrawCommands,
            specs: &sprite::Specs,
            TileSpec {
                xy: board_xy,
                top,
                outline,
                sides,
            }: TileSpec,
        ) {
            let xy = board_to_unscaled(specs, board_xy);

            cmds.sspr_override(
                specs.pyramid_pitch_tiles.xy_from_tile_sprite(0u16),
                specs.pyramid_pitch_tiles.rect(xy),
                top
            );
    
            cmds.sspr_override(
                specs.pyramid_pitch_tiles.xy_from_tile_sprite(1u16),
                specs.pyramid_pitch_tiles.rect(xy),
                sides
            );
    
            cmds.sspr_override(
                specs.pyramid_pitch_tiles.xy_from_tile_sprite(2u16),
                specs.pyramid_pitch_tiles.rect(xy),
                outline
            );
    
            cmds.sspr_override(
                specs.pyramid_pitch_tiles.xy_from_tile_sprite(3u16),
                specs.pyramid_pitch_tiles.rect(xy),
                outline
            );
        }

        draw_tile(
            commands,
            specs,
            TileSpec {
                xy,
                top: PALETTE[1],
                outline: PALETTE[4],
                sides: PALETTE[0],
            }
        );

        struct PyramidSpec {
            xy: board::XY,
            outline: ARGB,
            sides: ARGB,
        }

        fn draw_pyramid(
            cmds: &mut impl AddDrawCommands,
            specs: &sprite::Specs,
            PyramidSpec {
                xy: board_xy,
                outline,
                sides,
            }: PyramidSpec,
        ) {
            let base_xy = board_to_unscaled(specs, board_xy);

            let xy = base_xy + unscaled::W::new(6) - unscaled::H::new(9);

            cmds.sspr_override(
                specs.pyramid_pitch_pyramids.xy_from_tile_sprite(0u16),
                specs.pyramid_pitch_pyramids.rect(xy),
                outline
            );

            cmds.sspr_override(
                specs.pyramid_pitch_pyramids.xy_from_tile_sprite(1u16),
                specs.pyramid_pitch_pyramids.rect(xy),
                sides
            );
        }

        draw_pyramid(
            commands,
            specs,
            PyramidSpec {
                xy,
                outline: PALETTE[4],
                sides: PALETTE[3],
            }
        );
    }
}