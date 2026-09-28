#![deny(unused_variables)]

use gfx::{Commands, AddDrawCommands};
use gfx_sizes::{ARGB, PALETTE};
use platform_types::{command, sprite, unscaled, Button, Dir, Input, Speaker, TileSprite};
use xs::{Seed, Xs};

const MOVE_HIGHLIGHT_COLOUR: ARGB = (0x00FF_FFFF & PALETTE[6]) | 0xAA00_0000;

mod board {
    use std::collections::BTreeMap;

    pub type Inner = i16;

    pub type Distance = u8;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct X(pub Inner);
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Y(pub Inner);

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct XY {
        pub x: X,
        pub y: Y,
    }

    macro_rules! _xy {
        ($x: literal $y: literal) => {
            XY { x: X($x), y: Y($y) }
        }
    }
    pub(crate) use _xy as xy;

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

    pub type CellHeight = u8;
    
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Colour {
        Blue,
        Green,
        Red,
        //Yellow,
    }
    
    impl Colour {
        pub const ALL: [Colour; 3] = [
            Colour::Blue,
            Colour::Green,
            Colour::Red,
            //Colour::Yellow,
        ];

        pub const DEFAULT: Colour = Colour::ALL[0];
    
        pub fn index(self) -> usize {
            match self {
                Colour::Blue => 0,
                Colour::Green => 1,
                Colour::Red => 2,
                //Colour::Yellow => 3,
            }
        }
    }

    impl Default for Colour {
        fn default() -> Self {
            Self::DEFAULT
        }
    }

    #[derive(Clone, Copy, Debug, Default)]
    pub struct Pyramid {
        pub colour: Colour,
    }

    // TODO add exit tile
    pub type Contents = Option<Pyramid>;
    
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Cell {
        pub height: CellHeight,
        pub top_colour: Colour,
        pub contents: Contents,
    }

    pub type Board = BTreeMap<XY, Cell>;

    #[allow(unused)]
    pub fn manhattan_distance(a: XY, b: XY) -> Distance {
        ((a.x.0 as i8 - b.x.0 as i8).abs()
        + (a.y.0 as i8 - b.y.0 as i8).abs()) as Distance
    }

    pub fn path_from(_board: &Board, _source: XY, _target: XY) -> Option<Vec<XY>> {
        // TODO floodfill
        None
    }

    #[cfg(test)]
    mod path_from_works_on {
        use super::*;

        const BLANK_CELL: Cell = Cell {
            height: 0,
            top_colour: Colour::DEFAULT,
            contents: None,
        };
        const WALL_CELL: Cell = Cell {
            height: 0,
            top_colour: Colour::DEFAULT,
            contents: Some(Pyramid { colour: Colour::DEFAULT }),
        };

        fn split_board() -> Board {
            let mut output = Board::default();

            output.insert(xy!(0 0), BLANK_CELL);
            output.insert(xy!(1 0), BLANK_CELL);
            output.insert(xy!(2 0), BLANK_CELL);

            output.insert(xy!(0 1), WALL_CELL);
            output.insert(xy!(1 1), WALL_CELL);
            output.insert(xy!(2 1), WALL_CELL);

            output.insert(xy!(0 2), BLANK_CELL);
            output.insert(xy!(1 2), BLANK_CELL);
            output.insert(xy!(2 2), BLANK_CELL);

            output
        }

        #[test]
        fn this_case_with_no_path() {
            assert_eq!(path_from(&split_board(), xy!(0 0), xy!(2 2)), None);
        }

        #[test]
        fn this_case_with_a_path() {
            assert_eq!(
                path_from(&split_board(), xy!(0 0), xy!(2 0)),
                Some(vec![xy!(0 0), xy!(1 0), xy!(2 0)])
            );
        }
    }
    
}
use board::{Board, Cell, Colour, Pyramid};

#[derive(Clone, Debug)]
pub struct World {
    board: Board,
    selectrum_at: board::XY,
    player_at: board::XY,
}

// TODO? Worth caching this and/or precomputing it for each cell?
fn can_move_to(world: &World, xy: board::XY) -> bool {
    xy != world.player_at
    && board::path_from(&world.board, xy, world.player_at)
        .map(|path| path.len() < 3)
        .unwrap_or_default()
    && world.board.get(&xy)
        .map(|cell| cell.contents.is_none())
        .unwrap_or_default()
}

type PlayerFrame = u16;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ContextOption {
    Move,
}

type ContextOptionFlags = u8;

impl ContextOption {
    const ALL: [ContextOption; 1] = [
        ContextOption::Move,
    ];

    fn flag(self) -> ContextOptionFlags {
        let mut flag = 1;

        for option in Self::ALL {
            if option == self {
                return flag;
            }

            flag <<= 1;
        }

        0
    }

    fn label(self) -> &'static [u8] {
        match self {
            ContextOption::Move => b"move",
        }
    }
}

fn available_options(
    world: &World,
) -> ContextOptionFlags {
    let mut output = 0;

    if world.selectrum_at == world.player_at {
        output |= ContextOption::Move.flag();
    }

    output
}

fn first_set_context_option(flags: ContextOptionFlags) -> Option<ContextOption> {
    for i in 0..ContextOption::ALL.len() as ContextOptionFlags {
        if flags & (1 << i) != 0 {
            return Some(ContextOption::ALL[i as usize]);
        }
    }

    None
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum Menu {
    #[default]
    Closed,
    Context(ContextOption),
    Move,
}

#[derive(Clone, Debug)]
pub struct State {
    world: World,
    player_frame: PlayerFrame,
    menu: Menu,
}

impl State {
    pub fn new(rng: &mut Xs, _specs: &sprite::Specs) -> Self {
        let mut board = Board::new();

        let mut selectrum_at = board::XY::default();

        for y in -1..4 {
            for x in -2..4 {
                let top_i: usize = ((x + y) % Colour::ALL.len() as board::Inner).abs() as usize;

                let xy = board::XY {
                    x: board::X(x),
                    y: board::Y(y),
                };

                let contents = if xs::range(rng, 0..2) == 0 {
                    Some(Pyramid {
                        colour: Colour::ALL[top_i]
                    })
                } else {
                    None
                };

                board.insert(
                    xy,
                    Cell {
                        height: x.abs() as _,
                        top_colour: Colour::ALL[top_i],
                        contents,
                    }
                );

                selectrum_at = xy;
            }
        }

        Self {
            world: World {
                board,
                selectrum_at,
                player_at: <_>::default(),
            },
            player_frame: <_>::default(),
            menu: Menu::default(),
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
            match self.menu {
                Menu::Closed | Menu::Move => {
                    match dir {
                        Dir::Up => {
                            self.world.selectrum_at.y.0 = self.world.selectrum_at.y.0.saturating_sub(1);
                        },
                        Dir::Down => {
                            self.world.selectrum_at.y.0 = self.world.selectrum_at.y.0.saturating_add(1);
                        },
                        Dir::Left => {
                            self.world.selectrum_at.x.0 = self.world.selectrum_at.x.0.saturating_sub(1);
                        },
                        Dir::Right => {
                            self.world.selectrum_at.x.0 = self.world.selectrum_at.x.0.saturating_add(1);
                        },
                    }
                }
                Menu::Context(ContextOption::Move) => {}
            }
        } else if input.pressed_this_frame(Button::A) {
            match self.menu {
                Menu::Closed => {
                    let flags = available_options(
                        &self.world,
                    );

                    if let Some(context_option) = first_set_context_option(flags) {
                        self.menu = Menu::Context(context_option);
                    }
                }
                Menu::Context(ContextOption::Move) => {
                    self.menu = Menu::Move;
                }
                Menu::Move => {
                    if can_move_to(&self.world, self.world.selectrum_at) {
                        self.world.player_at = self.world.selectrum_at;
                        self.menu = Menu::Closed;
                    }
                }
            }
        } else if input.pressed_this_frame(Button::B) {
            self.menu = Menu::Closed;
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

        fn draw_top_fill(
            cmds: &mut impl AddDrawCommands,
            specs: &sprite::Specs,
            xy: unscaled::XY,
            colour: ARGB,
        ) {
            cmds.sspr_override(
                specs.pyramid_pitch_tiles.xy_from_tile_sprite(TOP_FILL),
                specs.pyramid_pitch_tiles.rect(xy),
                colour
            );
        }

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

        fn draw_player(
            cmds: &mut impl AddDrawCommands,
            specs: &sprite::Specs,
            board_xy: board::XY,
            player_frame: u16,
        ) {
            let xy = board_to_unscaled(specs, board_xy)
                + unscaled::W::new(10)
                - unscaled::H::new(25);

            cmds.sspr(
                specs.pyramid_pitch_player.xy_from_tile_sprite(player_frame),
                specs.pyramid_pitch_player.rect(xy),
            );
        }

        struct CellSpec {
            at: board::XY,
            selectrum_at: board::XY,
            show_move_highlight: bool,
        }

        fn draw_cell(
            cmds: &mut impl AddDrawCommands,
            specs: &sprite::Specs,
            cell: &Cell,
            CellSpec {
                at,
                selectrum_at,
                show_move_highlight,
            }: CellSpec
        ) {
            let top = colour_to_argb(cell.top_colour);

            let board_wh = specs.pyramid_pitch_tiles.tile();

            let base_xy = board_to_unscaled(specs, at);

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
                    PALETTE[OUTLINE_INDEX]
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
                if selectrum_at == at {
                    PALETTE[SELCTRUM_INDEX]
                } else {
                    PALETTE[OUTLINE_INDEX]
                },
            );

            if show_move_highlight {
                draw_top_fill(
                    cmds,
                    specs,
                    xy,
                    MOVE_HIGHLIGHT_COLOUR,
                );
            }

            match cell.contents {
                Some(pyramid) => {
                    let pyramid_xy = xy + unscaled::W::new(6) - unscaled::H::new(9);

                    cmds.sspr_override(
                        specs.pyramid_pitch_pyramids.xy_from_tile_sprite(0u16),
                        specs.pyramid_pitch_pyramids.rect(pyramid_xy),
                        PALETTE[OUTLINE_INDEX]
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

        let show_move_options = self.menu == Menu::Move;

        for xy in board::xy_iter(
            board::XY {
                x: board::X(-3),
                y: board::Y(-3),
            }
        ) {
            if let Some(cell) = self.world.board.get(&xy) {
                draw_cell(
                    commands,
                    specs,
                    cell,
                    CellSpec {
                        at: xy,
                        selectrum_at: self.world.selectrum_at,
                        show_move_highlight:
                            show_move_options
                            && can_move_to(&self.world, xy)
                    }
                );
            }

            if self.world.player_at == xy {
                draw_player(
                    commands,
                    specs,
                    xy,
                    self.player_frame,
                );
            }
        }

        // Render menu

        match self.menu {
            // TODO highlight plces that can be moved to in move mode
            Menu::Closed | Menu::Move => {}
            Menu::Context(selection) => {
                let menu_options_flags = available_options(
                    &self.world,
                );

                if menu_options_flags != 0 {
                    const OPTION_W: unscaled::W = unscaled::W::new(120);
                    const OPTION_H: unscaled::H = unscaled::H::new(25);

                    let selectrum_xy = board_to_unscaled(specs, self.world.selectrum_at);

                    commands.nine_slice(
                        gfx::nine_slice::CONTEXT_MENU,
                        unscaled::Rect {
                            x: selectrum_xy.x,
                            y: selectrum_xy.y,
                            w: OPTION_W,
                            h: OPTION_H * menu_options_flags.count_ones() as _,
                        },
                    );

                    let mut at = selectrum_xy;

                    let mut flags = menu_options_flags;
                    let mut index = 0;

                    while flags != 0 && index < ContextOption::ALL.len() {
                        let current_flag = 1 << index;

                        if flags & current_flag != 0 {
                            let option = ContextOption::ALL[index];

                            commands.print_line(
                                option.label(),
                                at + unscaled::WH{ w: unscaled::W::new(6), h: unscaled::H::new(9) },
                                4
                            );


                            if option == selection {
                                commands.nine_slice(
                                    gfx::nine_slice::SELECTRUM,
                                    unscaled::Rect {
                                        x: at.x,
                                        y: at.y,
                                        w: OPTION_W,
                                        h: OPTION_H,
                                    },
                                );
                            }

                            at += OPTION_H;

                            flags = flags &!current_flag;
                        }

                        index += 1;
                    }
                }
            }
        }
    }
}