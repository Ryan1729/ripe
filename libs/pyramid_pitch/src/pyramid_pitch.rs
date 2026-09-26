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
pub struct World {
    board: Board,
    selectrum_at: board::XY,
    player_at: board::XY,
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

#[derive(Copy, Clone, Debug, Default)]
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
            match dbg!(self.menu) {
                Menu::Closed => {
                    let flags = available_options(
                        &self.world,
                    );

                    if let Some(context_option) = dbg!(first_set_context_option(flags)) {
                        self.menu = Menu::Context(context_option);
                    }
                }
                Menu::Context(ContextOption::Move) => {
                    self.menu = Menu::Move;
                }
                Menu::Move => {
                    // TODO check whether the move should be allowed
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
        }

        fn draw_cell(
            cmds: &mut impl AddDrawCommands,
            specs: &sprite::Specs,
            cell: &Cell,
            CellSpec {
                at,
                selectrum_at,
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
                    }
                );
            } else if self.world.selectrum_at == xy {
                draw_top_outline(
                    commands,
                    specs,
                    board_to_unscaled(specs, xy),
                    PALETTE[SELCTRUM_INDEX],
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