#![deny(unused_variables)]

use gfx::{Commands, AddDrawCommands};
use gfx_sizes::{ARGB, PALETTE};
use platform_types::{command, sprite, unscaled, Button, Dir, Input, Speaker, TileSprite};
use xs::{Seed, Xs};

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum Colour {
    #[default]
    Blue,
    Green,
    Red,
    //Yellow,
}

impl Colour {
    const ALL: [Colour; 3] = [
        Colour::Blue,
        Colour::Green,
        Colour::Red,
        //Colour::Yellow,
    ];

    fn index(self) -> usize {
        match self {
            Colour::Blue => 0,
            Colour::Green => 1,
            Colour::Red => 2,
            //Colour::Yellow => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Pyramid {
    colour: Colour,
}

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

    pub fn xy_iter(base: XY) -> impl Iterator<Item = XY> {
        // TODO take as params probably.
        let width = 8;
        let height = 8;

        let max_x = base.x.0 + width;
        let max_y = base.y.0 + height;

        let mut x = base.x.0;
        let mut y = base.y.0;

        std::iter::from_fn(move || {
            let output = XY {
                x: X(x),
                y: Y(y),
            };

            if y > max_y {
                return None;
            }

            x += 1;

            if x > max_x {
                x = base.x.0;

                y += 1;
            }

            Some(output)
        })
    }
}

type CellHeight = u8;

// TODO add player
// TODO add exit tile
type Contents = Option<Pyramid>;

#[derive(Clone, Copy, Debug, Default)]
struct Cell {
    height: CellHeight,
    top_colour: Colour,
    contents: Contents,
}

type Board = BTreeMap<board::XY, Cell>;

#[derive(Clone, Debug)]
pub struct State {
    board: Board,
    selectrum_at: board::XY,
}

impl State {
    pub fn new(_rng: &mut Xs, _specs: &sprite::Specs) -> Self {
        let mut board = Board::new();

        let mut selectrum_at = board::XY::default();

        for y in -1..4 {
            for x in -2..4 {
                let top_i: usize = ((x + y) % Colour::ALL.len() as board::Inner).abs() as usize;

                let xy = board::XY {
                    x: board::X(x),
                    y: board::Y(y),
                };

                board.insert(
                    xy,
                    Cell {
                        height: x.abs() as _,
                        top_colour: Colour::ALL[top_i],
                        contents: Some(Pyramid {
                            colour: Colour::ALL[top_i]
                        }),
                    }
                );

                selectrum_at = xy;
            }
        }

        Self {
            board,
            selectrum_at,
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
        input: Input,
        _speaker: &mut Speaker,
    ) {
        //
        // Update
        //

        if let Some(dir) = input.dir_pressed_this_frame() {
            match dir {
                Dir::Up => {
                    self.selectrum_at.y.0 = self.selectrum_at.y.0.saturating_sub(1);
                },
                Dir::Down => {
                    self.selectrum_at.y.0 = self.selectrum_at.y.0.saturating_add(1);
                },
                Dir::Left => {
                    self.selectrum_at.x.0 = self.selectrum_at.x.0.saturating_sub(1);
                },
                Dir::Right => {
                    self.selectrum_at.x.0 = self.selectrum_at.x.0.saturating_add(1);
                },
            }
        }

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

        const TOP_FILL: TileSprite = 0;
        const BASE_FILL: TileSprite = 1;
        const TOP_OUTLINE: TileSprite = 2;
        const BASE_OUTLINE: TileSprite = 3;

        const OUTLINE_INDEX: usize = 4;
        const SELCTRUM_INDEX: usize = 3;

        fn draw_top_outline(
            cmds: &mut impl AddDrawCommands,
            specs: &sprite::Specs,
            xy: unscaled::XY,
            colour: ARGB,
        ) {
            cmds.sspr_override(
                specs.pyramid_pitch_tiles.xy_from_tile_sprite(TOP_OUTLINE),
                specs.pyramid_pitch_tiles.rect(xy),
                colour
            );
        }

        fn draw_cell(
            cmds: &mut impl AddDrawCommands,
            specs: &sprite::Specs,
            board_xy: board::XY,
            cell: &Cell,
            outline: ARGB,
        ) {
            let top = colour_to_argb(cell.top_colour);

            let board_wh = specs.pyramid_pitch_tiles.tile();

            let base_xy = board_to_unscaled(specs, board_xy);

            let mut xy = base_xy;

            for i in 0..=cell.height {
                if i > 0 {
                    xy.y -= board_wh.h / 4;
                }

                cmds.sspr_override(
                    specs.pyramid_pitch_tiles.xy_from_tile_sprite(BASE_FILL),
                    specs.pyramid_pitch_tiles.rect(xy),
                    PALETTE[5]
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

            draw_top_outline(
                cmds,
                specs,
                xy,
                outline,
            );

            match cell.contents {
                Some(pyramid) => {
                    let pyramid_xy = xy + unscaled::W::new(6) - unscaled::H::new(9);

                    cmds.sspr_override(
                        specs.pyramid_pitch_pyramids.xy_from_tile_sprite(0u16),
                        specs.pyramid_pitch_pyramids.rect(pyramid_xy),
                        outline
                    );

                    cmds.sspr_override(
                        specs.pyramid_pitch_pyramids.xy_from_tile_sprite(1u16),
                        specs.pyramid_pitch_pyramids.rect(pyramid_xy),
                        colour_to_argb(pyramid.colour)
                    );
                },
                None => {}
            }
        }

        fn colour_to_argb(colour: Colour) -> ARGB {
            PALETTE[colour.index()]
        }

        for xy in board::xy_iter(
            board::XY {
                x: board::X(-3),
                y: board::Y(-3),
            }
        ) {
            if let Some(cell) = self.board.get(&xy) {
                draw_cell(
                    commands,
                    specs,
                    xy,
                    cell,
                    if self.selectrum_at == xy {
                        PALETTE[SELCTRUM_INDEX]
                    } else {
                        PALETTE[OUTLINE_INDEX]
                    }
                );
            } else if self.selectrum_at == xy {
                draw_top_outline(
                    commands,
                    specs,
                    board_to_unscaled(specs, xy),
                    PALETTE[SELCTRUM_INDEX],
                );
            }
        }
    }
}