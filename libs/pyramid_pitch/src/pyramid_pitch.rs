#![deny(unused_variables)]

use gfx::{Commands, AddDrawCommands};
use gfx_sizes::{ARGB, PALETTE};
use platform_types::{command, sprite, unscaled, Button, Dir, Input, Speaker, TileSprite};
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

type CellHeight = u8;

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

        fn board_to_unscaled(
            _specs: &sprite::Specs,
            xy: board::XY,
        ) -> unscaled::XY {
            //let board_wh = specs.pyramid_pitch_tiles.tile();

            unscaled::XY {
                x: unscaled::X(
                    //(xy.x.0 * board_wh.w.get() / 4)
                    //- (xy.y.0 * board_wh.h.get() / 2)
                    (xy.x.0 * 32)
                    + (xy.y.0 * -32)
                )
                + unscaled::W::new((command::WIDTH_SIGNED / 8) * 3),
                y: unscaled::Y(
                    xy.x.0 * 16
                    + xy.y.0 * 16
                )
                + unscaled::H::new((command::HEIGHT_SIGNED / 8) * 3),
            }
        }

        struct TileSpec {
            xy: board::XY,
            height: CellHeight,
            top: ARGB,
            outline: ARGB,
            sides: ARGB,
        }

        fn draw_tile(
            cmds: &mut impl AddDrawCommands,
            specs: &sprite::Specs,
            TileSpec {
                xy: board_xy,
                height,
                top,
                outline,
                sides,
            }: TileSpec,
        ) {
            const TOP_FILL: TileSprite = 0;
            const BASE_FILL: TileSprite = 1;
            const TOP_OUTLINE: TileSprite = 2;
            const BASE_OUTLINE: TileSprite = 3;

            let board_wh = specs.pyramid_pitch_tiles.tile();

            let base_xy = board_to_unscaled(specs, board_xy);

            let mut xy = base_xy;

            for i in 0..=height {
                if i > 0 {
                    xy.y -= board_wh.h / 4;
                }

                cmds.sspr_override(
                    specs.pyramid_pitch_tiles.xy_from_tile_sprite(BASE_FILL),
                    specs.pyramid_pitch_tiles.rect(xy),
                    sides
                );
    
                cmds.sspr_override(
                    specs.pyramid_pitch_tiles.xy_from_tile_sprite(BASE_OUTLINE),
                    specs.pyramid_pitch_tiles.rect(xy),
                    outline
                );
            }
            
            cmds.sspr_override(
                specs.pyramid_pitch_tiles.xy_from_tile_sprite(TOP_FILL),
                specs.pyramid_pitch_tiles.rect(xy),
                top
            );
    
            cmds.sspr_override(
                specs.pyramid_pitch_tiles.xy_from_tile_sprite(TOP_OUTLINE),
                specs.pyramid_pitch_tiles.rect(xy),
                outline
            );
    
            
        }

        for y in 0..4 {
            for x in 0..4 {
                let top_i = ((x + y) % 6) as usize;

                draw_tile(
                    commands,
                    specs,
                    TileSpec {
                        xy: board::XY {
                            x: board::X(x),
                            y: board::Y(y),
                        },
                        height: x as _,
                        top: PALETTE[top_i],
                        outline: PALETTE[4],
                        sides: PALETTE[0],
                    }
                );
            }
        }

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

        // FIXME Seems like we need to iterate over the tiles and other things on 
        // the tile together, and probably in an order related to x+y, to make
        // the overlapping work out properly
        draw_pyramid(
            commands,
            specs,
            PyramidSpec {
                xy: board::XY {
                    x: board::X(1),
                    y: board::Y(1),
                },
                outline: PALETTE[4],
                sides: PALETTE[2],
            }
        );

        draw_pyramid(
            commands,
            specs,
            PyramidSpec {
                xy: board::XY {
                    x: board::X(1),
                    y: board::Y(2),
                },
                outline: PALETTE[4],
                sides: PALETTE[3],
            }
        );

        
    }
}