use gfx::{Commands};
//use gfx_sizes::ARGB;
#[allow(unused)]
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use qrs::{QRS, QRSD, Q, R};
//use vec1::{Grid1, Grid1Spec, vec1, Vec1};
use xs::{Seed, Xs};

use std::collections::{BTreeMap};

#[derive(Clone, Copy, Debug, Default)]
pub enum TileKind {
    #[default]
    Symbol,
    Warp,
}

impl TileKind {
    const ALL: [TileKind; 2] = [
        Self::Symbol,
        Self::Warp,
    ];
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Tile {
    pub kind: TileKind,
}

pub type Key = QRS;

pub type Tiles = BTreeMap<Key, Tile>;

#[derive(Clone, Copy, Debug, Default)]
pub enum ContextMenu {
    #[default]
    Closed,
    Open { selection: usize },
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
    pub tiles: Tiles,
    pub selectrum_at: QRS,
    pub context_menu: ContextMenu,
}

impl State {
    pub fn new(rng: &mut Xs, specs: &sprite::Specs) -> Self {
        let seed = xs::new_seed(rng);

        Self::init(seed, specs)
    }

    fn init(seed: Seed, _specs: &sprite::Specs) -> Self {
        let mut rng_ = xs::from_seed(seed);
        let rng = &mut rng_;

        let mut tiles = Tiles::new();

        macro_rules! qr {
            ($q_inner: literal $(,)? $r_inner: literal) => {
                QRS {
                    q: Q($q_inner),
                    r: R($r_inner),
                }
            }
        }

        for at in qrs::spiral(2, qr!(0, 0)) {
            tiles.insert(
                at,
                Tile {
                    kind: TileKind::ALL[xs::range(rng, 0..TileKind::ALL.len() as u32) as usize],
                }
            );
        }

        Self {
            seed,
            rng: rng_,
            tiles,
            //mobs
            .. <_>::default()
        }
    }

    #[allow(unused)]
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

        // TODO On selecting a location, pop up a menu to twiddle it different amounts

        const MENU_OPTIONS: [&str; 5] = [
            "+1/6",
            "+2/6",
            "+3/6",
            "-2/6",
            "-1/6",
        ];

        let mut player_moved = false;

        match &mut self.context_menu {
            ContextMenu::Closed => {
                if input.pressed_this_frame(Button::UP) {
                    let dir = if input.gamepad.contains(Button::LEFT) {
                        qrs::Dir::DecQIncS
                    } else if input.gamepad.contains(Button::RIGHT) {
                        qrs::Dir::DecRIncQ
                    } else {
                        qrs::Dir::DecRIncS
                    };
                    let target_qrs = self.selectrum_at.neighbor(dir);
                    if self.tiles.get(&target_qrs).is_some() {
                        player_moved = true;
                        self.selectrum_at = target_qrs;
                    }
                } else if input.pressed_this_frame(Button::DOWN) {
                    let dir = if input.gamepad.contains(Button::LEFT) {
                        qrs::Dir::DecQIncR
                    } else if input.gamepad.contains(Button::RIGHT) {
                        qrs::Dir::DecSIncQ
                    } else {
                        qrs::Dir::DecSIncR
                    };
        
                    let target_qrs = self.selectrum_at.neighbor(dir);
                    if self.tiles.get(&target_qrs).is_some() {
                        player_moved = true;
                        self.selectrum_at = target_qrs;
                    }
                } else if input.pressed_this_frame(Button::A) {
                    self.context_menu = ContextMenu::Open { selection: 0 };
                }
            },
            ContextMenu::Open { selection } => {
                if input.pressed_this_frame(Button::UP) {
                    if *selection == 0 {
                        *selection = MENU_OPTIONS.len();
                    }
                    *selection -= 1;
                } else if input.pressed_this_frame(Button::DOWN) {
                    *selection += 1;
                    if *selection == MENU_OPTIONS.len() {
                        *selection = 0;
                    }
                } else if input.pressed_this_frame(Button::A) {
                    dbg!(MENU_OPTIONS[*selection]);
                    self.context_menu = ContextMenu::Closed;
                } else if input.pressed_this_frame(Button::B) {
                    self.context_menu = ContextMenu::Closed;
                }
            },
        }

        if input.pressed_this_frame(Button::START) {
            self.restart(specs);
        }
    
        self.tick();

        //
        //
        // Render Section
        //
        //

        const X_Q_FACTOR: i16 = 2;
        const X_R_FACTOR: i16 = 0;
        
        const Y_Q_FACTOR: i16 = 1;
        const Y_R_FACTOR: i16 = 2;

        const HEX_X_SCALE: i16 = 22;
        const HEX_Y_SCALE: i16 = 25;
        
        const HEX_X_OFFSET: i16 = 160;
        const HEX_Y_OFFSET: i16 = 140;

        fn qrs_to_unscaled(qrs: QRS) -> unscaled::XY {
            let q = qrs.q.0;
            let r = qrs.r.0;

            let x = (X_Q_FACTOR * q + X_R_FACTOR * r) * HEX_X_SCALE + HEX_X_OFFSET;
            let y = (Y_Q_FACTOR * q + Y_R_FACTOR * r) * HEX_Y_SCALE + HEX_Y_OFFSET;

            unscaled::XY {
                x: unscaled::X(x.try_into().unwrap_or(0)),
                y: unscaled::Y(y.try_into().unwrap_or(0)),
            }
        }

        fn tile_xy(qrs: QRS, Tile { .. }: &Tile) -> unscaled::XY {
            qrs_to_unscaled(qrs)
        }

        //
        // Render Tiles
        //

        for (at, tile) in self.tiles.iter() {
            let xy = tile_xy(*at, &tile);

            commands.sspr_override(
                specs.hex_twiddle_tiles.xy_from_tile_sprite(0u16),
                command::Rect::from_unscaled(specs.hex_twiddle_tiles.rect(xy)),
                match tile.kind {
                    TileKind::Symbol => 0xFF3352E1,
                    TileKind::Warp => 0xFF30B06E,
                }
            );
        }

        //
        // Render UI
        //

        // Selectrum
        let selectrum_xy = qrs_to_unscaled(self.selectrum_at);

        commands.sspr_override(
            specs.hex_twiddle_tiles.xy_from_tile_sprite(1u16),
            command::Rect::from_unscaled(specs.hex_twiddle_tiles.rect(selectrum_xy)),
            0xFFFFB937
        );

        // Context Menu
        match &mut self.context_menu {
            ContextMenu::Closed => {},
            ContextMenu::Open{ selection } => {
                const OPTION_W: unscaled::W = unscaled::W(50);
                const OPTION_H: unscaled::H = unscaled::H(25);

                commands.nine_slice(
                    gfx::nine_slice::CONTEXT_MENU,
                    unscaled::Rect {
                        x: selectrum_xy.x,
                        y: selectrum_xy.y,
                        w: OPTION_W,
                        h: OPTION_H * MENU_OPTIONS.len() as _,
                    },
                );

                let mut at = selectrum_xy;

                for i in 0..MENU_OPTIONS.len() {
                    commands.print_line(
                        MENU_OPTIONS[i].as_ref(),
                        at + unscaled::WH{ w: unscaled::W(6), h: unscaled::H(9) },
                        4
                    );

                    if i == *selection {
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
                }
            },
        }
    }
}
