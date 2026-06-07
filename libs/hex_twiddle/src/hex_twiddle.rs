use gfx::{Commands};
//use gfx_sizes::ARGB;
#[allow(unused)]
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use qrs::{QRS, QRSD, Q, R};
//use vec1::{Grid1, Grid1Spec, vec1, Vec1};
use xs::{Seed, Xs};

use std::collections::{BTreeMap};

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

mod offset {
    use platform_types::unscaled;
    use qrs;

    use super::*;

    #[derive(Clone, Copy, Debug, Default)]
    pub struct Offset {
        xyd: unscaled::XYD,
    }

    impl From<qrs::Targeting> for Offset {
        fn from(targeting: qrs::Targeting) -> Self {
            let source = qrs_to_unscaled(targeting.source);
            let target = qrs_to_unscaled(targeting.target);

            Self {
                xyd: source - target
            }
        }
    }

    const DECAY_RATE: unscaled::XYD = unscaled::XYD {
        xd: unscaled::XD(1),
        yd: unscaled::YD(1),
    };

    impl Offset {
        pub fn xyd(&self) -> unscaled::XYD {
            self.xyd
        }

        pub fn is_settled(&self) -> bool {
            self.xyd == unscaled::XYD::default()
        }

        pub fn advance(&mut self) {
            use unscaled::{XD, YD};

            if self.is_settled() { return }

            let x_started_positive = self.xyd.xd > XD(0);
            let y_started_positive = self.xyd.yd > YD(0);

            if x_started_positive {
                self.xyd.xd -= DECAY_RATE.xd;
                if self.xyd.xd < XD(0) {
                    self.xyd.xd = XD(0);
                }
            } else {
                self.xyd.xd += DECAY_RATE.xd;
                if self.xyd.xd > XD(0) {
                    self.xyd.xd = XD(0);
                }
            }
            
            if y_started_positive {
                self.xyd.yd -= DECAY_RATE.yd;
                if self.xyd.yd < YD(0) {
                    self.xyd.yd = YD(0);
                }
            } else {
                self.xyd.yd += DECAY_RATE.yd;
                if self.xyd.yd > YD(0) {
                    self.xyd.yd = YD(0);
                }
            }
        }
    }
}
use offset::Offset;

#[derive(Clone, Copy, Debug, Default)]
enum Twiddle {
    #[default]
    OneSixth,
    TwoSixths,
    ThreeSixths,
    MinusTwoSixths,
    MinusOneSixths,
}

impl Twiddle {
    fn signum(self) -> i8 {
        match self {
            Twiddle::OneSixth
            | Twiddle::TwoSixths
            | Twiddle::ThreeSixths => 1,

            Twiddle::MinusTwoSixths
            | Twiddle::MinusOneSixths => -1,
        }
    }
}

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

type Offsets = [Offset; 4];

#[derive(Clone, Copy, Debug, Default)]
pub struct Tile {
    pub kind: TileKind,
    pub offsets: Offsets,
}

pub type Key = QRS;

pub type Tiles = BTreeMap<Key, Tile>;

fn twiddle(tiles: &mut Tiles, key: Key, twiddle_amount: Twiddle) {
    let base: QRS = key;

    #[derive(Clone, Copy, Default)]
    struct TwiddleTargeting {
        offsets: Offsets,
        target: QRS,
    }

    let mut twiddled: [Option<(TwiddleTargeting, Tile)>; qrs::Dir::ALL.len()] = [None; qrs::Dir::ALL.len()];

    let mut dir_i = 0;

    for to_tile_to_move in qrs::Dir::ALL {
        let was_at = base.neighbor(to_tile_to_move);

        if let Some(tile) = tiles.remove(&was_at) {
            let mut targeting = TwiddleTargeting::default();
            targeting.target = was_at;
            let mut offsets_i = 0;

            let mut current_dir = to_tile_to_move.clockwise(2 * twiddle_amount.signum());

            let mut angle: Option<Twiddle> = Some(twiddle_amount);

            while let Some(twiddle) = angle {
                let source = targeting.target;
                targeting.target = source.neighbor(current_dir);

                targeting.offsets[offsets_i] = Offset::from(qrs::Targeting{ source, target: targeting.target });
                offsets_i += 1;

                // TODO handle each case properly, including direction
                angle = match twiddle {
                    Twiddle::OneSixth => None,
                    Twiddle::TwoSixths => Some(Twiddle::OneSixth),
                    Twiddle::ThreeSixths => Some(Twiddle::TwoSixths),

                    Twiddle::MinusTwoSixths => Some(Twiddle::MinusOneSixths),
                    Twiddle::MinusOneSixths => None,
                };

                current_dir = current_dir.clockwise(1 * twiddle.signum());
            }

            twiddled[dir_i] = Some((targeting, tile));
        }
        
        dir_i += 1;
    }

    for opt in twiddled {
        let Some((targeting, mut tile)) = opt else { continue };

        tile.offsets = targeting.offsets;
        tiles.insert(targeting.target, tile);
    }
}

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
                    .. <_>::default()
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
        for (_, tile) in &mut self.tiles {
            for offset in &mut tile.offsets {
                if !offset.is_settled() {
                    offset.advance();
                    break
                }
            }
        }
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

        const MENU_OPTIONS: [(Twiddle, &str); 5] = [
            (Twiddle::OneSixth, "+1/6"),
            (Twiddle::TwoSixths, "+2/6"),
            (Twiddle::ThreeSixths,"+3/6"),
            (Twiddle::MinusTwoSixths, "-2/6"),
            (Twiddle::MinusOneSixths, "-1/6"),
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
                    assert!(!player_moved);
                    twiddle(
                        &mut self.tiles,
                        self.selectrum_at,
                        MENU_OPTIONS[*selection].0,
                    );
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

        fn tile_xy(qrs: QRS, Tile { offsets, .. }: &Tile) -> unscaled::XY {
            let mut output = qrs_to_unscaled(qrs);

            for offset in offsets {
                output += offset.xyd();
            }

            output
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
                        MENU_OPTIONS[i].1.as_ref(),
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
