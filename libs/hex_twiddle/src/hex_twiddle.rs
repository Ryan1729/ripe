use gfx::{Commands};
//use gfx_sizes::ARGB;
#[allow(unused)]
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use qrs::{QRS, QRSD, Q, R, qr};
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

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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

    pub fn direct(dir: qrs::Dir) -> Offset {
        use qrs::Dir::*;
        let basis = match dir {
            // Up
            DecRIncS => unscaled::XYD {
                xd: unscaled::XD(X_Q_FACTOR * 0 + X_R_FACTOR * -1),
                yd: unscaled::YD(Y_Q_FACTOR * 0 + Y_R_FACTOR * -1),
            },
            // Up-Right
            DecRIncQ => unscaled::XYD {
                xd: unscaled::XD(X_Q_FACTOR * 1 + X_R_FACTOR * -1),
                yd: unscaled::YD(Y_Q_FACTOR * 1 + Y_R_FACTOR * -1),
            },
            // Down-Right
            DecSIncQ => unscaled::XYD {
                xd: unscaled::XD(X_Q_FACTOR * 1 + X_R_FACTOR * 0),
                yd: unscaled::YD(Y_Q_FACTOR * 1 + Y_R_FACTOR * 0),
            },
            // Down
            DecSIncR => unscaled::XYD {
                xd: unscaled::XD(X_Q_FACTOR * 0 + X_R_FACTOR * 1),
                yd: unscaled::YD(Y_Q_FACTOR * 0 + Y_R_FACTOR * 1),
            },
            // Down-Left
            DecQIncR => unscaled::XYD {
                xd: unscaled::XD(X_Q_FACTOR * -1 + X_R_FACTOR * 1),
                yd: unscaled::YD(Y_Q_FACTOR * -1 + Y_R_FACTOR * 1),
            },
            // Up-Left
            DecQIncS => unscaled::XYD {
                xd: unscaled::XD(X_Q_FACTOR * -1 + X_R_FACTOR * 0),
                yd: unscaled::YD(Y_Q_FACTOR * -1 + Y_R_FACTOR * 0),
            },
        };

        // Point the other way because so we start exactly where we visually were before the move
        let xyd = unscaled::XYD {
            xd: basis.xd * -1 * HEX_X_SCALE,
            yd: basis.yd * -1 * HEX_Y_SCALE,
        };

        Offset {
            xyd
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


#[derive(Clone, Copy, Debug)]
enum MenuOption {
    Twiddle(Twiddle),
    Move,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Symbol {
    #[default]
    A,
    B,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileKind {
    Symbol(Symbol),
    Warp,
    Split,
}

impl Default for TileKind {
    fn default() -> Self {
        Self::Symbol(Symbol::default())
    }
}

impl TileKind {
    const ALL: [TileKind; 4] = [
        Self::Symbol(Symbol::A),
        Self::Symbol(Symbol::B),
        Self::Warp,
        Self::Split,
    ];
}

type Offsets = [Offset; 4];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tile {
    pub kind: TileKind,
    pub offsets: Offsets,
}

pub type Key = QRS;

pub type Tiles = BTreeMap<Key, Tile>;

type TileSprite = u16;

const SELECTRUM: TileSprite = 1;

type MobSprite = u16;

const DIR_COUNT: MobSprite = qrs::Dir::ALL.len() as _;

const PLAYER_MAIN_BASE: MobSprite = 0;
const PLAYER_HELPER_BASE: MobSprite = PLAYER_MAIN_BASE + DIR_COUNT;
const CPU_BASE: MobSprite = PLAYER_HELPER_BASE + DIR_COUNT;
const ARROW_BASE: MobSprite = CPU_BASE + DIR_COUNT;

//type Facing = Dir;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Entity {
    pub offsets: Offsets,
    pub sprite: MobSprite,
    // TODO add and set as needed to make sprites turn in the direction they move
    //pub facing: Facing,
}

mod mobs {
    use super::*;

    #[repr(u8)]
    #[derive(Clone, Copy, Debug, Default)]
    pub enum Index {
        #[default]
        Zero,
        One,
        Two,
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Target {
        Player(Index),
        NonPlayer(Index)
    }

    impl Target {
        const ALL: [Self; 6] = [
            Self::Player(Index::Zero),
            Self::Player(Index::One),
            Self::Player(Index::Two),
            Self::NonPlayer(Index::Zero),
            Self::NonPlayer(Index::One),
            Self::NonPlayer(Index::Two),
        ];
    }

    const PIECES_PER_PLAYER: usize = 3;

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct Mobs {
        player_mobs: [(Key, Entity); PIECES_PER_PLAYER],
        cpu_mobs: [(Key, Entity); PIECES_PER_PLAYER],
    }

    macro_rules! get_ref {
        ($mobs: ident $target: expr) => {
            match $target {
                Target::Player(index) => &mut $mobs.player_mobs[index as u8 as usize],
                Target::NonPlayer(index) => &mut $mobs.cpu_mobs[index as u8 as usize],
            }
        }
    }

    impl Mobs {
        pub fn new(center: QRS) -> Self {
            let mut output = Self::default();

            output.set(
                Target::Player(Index::Zero),
                center.neighbor(qrs::Dir::ALL[0]),
                Entity {
                    sprite: PLAYER_MAIN_BASE,
                    ..<_>::default()
                }
            );

            output.set(
                Target::NonPlayer(Index::Zero),
                center.neighbor(qrs::Dir::ALL[1]),
                Entity {
                    sprite: CPU_BASE,
                    ..<_>::default()
                }
            );

            output.set(
                Target::Player(Index::One),
                center.neighbor(qrs::Dir::ALL[2]),
                Entity {
                    sprite: PLAYER_HELPER_BASE,
                    ..<_>::default()
                }
            );

            output.set(
                Target::NonPlayer(Index::One),
                center.neighbor(qrs::Dir::ALL[3]),
                Entity {
                    sprite: CPU_BASE,
                    ..<_>::default()
                }
            );

            output.set(
                Target::Player(Index::Two),
                center.neighbor(qrs::Dir::ALL[4]),
                Entity {
                    sprite: PLAYER_HELPER_BASE,
                    ..<_>::default()
                }
            );

            output.set(
                Target::NonPlayer(Index::Two),
                center.neighbor(qrs::Dir::ALL[5]),
                Entity {
                    sprite: CPU_BASE,
                    ..<_>::default()
                }
            );

            output
        }

        fn set(&mut self, target: Target, key: Key, entity: Entity) {
            let current = get_ref!(self target);

            current.0 = key;
            current.1 = entity;
        }

        pub fn iter(&self) -> impl Iterator<Item = &(Key, Entity)> {
            self.cpu_mobs.iter().chain(self.player_mobs.iter())
        }

        pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (Key, Entity)> {
            self.cpu_mobs.iter_mut().chain(self.player_mobs.iter_mut())
        }

        pub fn is_free(&self, needle: Key) -> bool {
            let mut is_free = true;
            for (key, _) in self.iter() {
                if key == needle {
                    is_free = false;
                    break
                }
            }
            is_free
        }

        pub fn apply_dir(&mut self, target: Target, dir: qrs::Dir) {
            let current = match target {
                Target::Player(index) => &mut self.player_mobs[index as u8 as usize],
                Target::NonPlayer(index) => &mut self.cpu_mobs[index as u8 as usize],
            };

            let new_qrs = current.0 + QRSD::from(dir);

            if self.is_free(new_qrs) {
                let current = get_ref!(self target);

                current.0 = new_qrs;
                current.1.offsets = [offset::direct(dir), Offset::default(), Offset::default(), Offset::default()];
            }
        }

        pub fn get_target(&self, key: Key) -> Option<Target> {
            for target in Target::ALL {
                let current = match target {
                    Target::Player(index) => &self.player_mobs[index as u8 as usize],
                    Target::NonPlayer(index) => &self.cpu_mobs[index as u8 as usize],
                };
                if current.0 == key {
                    return Some(target);
                }
            }

            None
        }

        pub fn mutate(&mut self, target: Target, f: impl FnOnce(&mut (Key, Entity))) {
            let current = get_ref!(self target);
            f(current);
        }
    }
}
use mobs::Mobs;

// TODO write tests to drive out the bug
fn twiddle(tiles: &mut Tiles, mobs: &mut Mobs, key: Key, twiddle_amount: Twiddle) {
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

        if let Some(mob_target) = mobs.get_target(targeting.target) {
            mobs.mutate(
                mob_target,
                |(key, mob)| {
                    // TODO? Is it worth trying to ensure we can't put two different pieces 
                    // at the same key at the level of the Mob API?
                    *key = targeting.target;
                    mob.offsets = targeting.offsets;
                }
            );
        }
    }
}

#[cfg(test)]
mod twiddle_works {
    use super::*;

    #[test]
    fn on_this_basic_example() {
        let ring_coords = [
            qr!(0, -1),
            qr!(1, -1),
            qr!(1, 0),
            qr!(0, 1),
            qr!(-1, 1),
            qr!(-1, 0),
        ];

        macro_rules! ring_insert {
            ($tiles: expr, $ring_i: expr, $kind: expr $(,)?) => {
                $tiles.insert(
                    ring_coords[$ring_i],
                    Tile {
                        kind: $kind,
                        .. <_>::default()
                    }
                );
            }
        }

        let mut index = 0;

        let mut tiles = Tiles::default();
        for i in 0..ring_coords.len() {
            index %= TileKind::ALL.len();

            ring_insert!(tiles, i, TileKind::ALL[index]);

            index += 1;
        }

        let mut index = 0;

        let mut expected_tiles = Tiles::default();
        for mut i in 0..ring_coords.len() {
            i += 1;
            i %= ring_coords.len();

            index %= TileKind::ALL.len();

            ring_insert!(expected_tiles, i, TileKind::ALL[index]);

            index += 1;
        }

        let mut mobs = Mobs::new(<_>::default());

        let mut expected_mobs = mobs.clone();
        let Some(mut previous_ref) = expected_mobs.iter_mut().last() else {
            panic!("No mobs")
        };
        let mut previous = previous_ref.clone();

        for entry in expected_mobs.iter_mut() {
            let temp = entry.clone();
            *entry = previous.clone();
            previous = temp;
        }

        twiddle(
            &mut tiles,
            &mut mobs,
            <_>::default(),
            Twiddle::OneSixth
        );
        
        let mut broke_early = false;
        loop {
            broke_early = false;
            // tick {
            for (_, tile) in &mut tiles {
                for offset in &mut tile.offsets {
                    if !offset.is_settled() {
                        offset.advance();
                        broke_early = true;
                        break
                    }
                }
            }
    
            for (_, mob) in mobs.iter_mut() {
                for offset in &mut mob.offsets {
                    if !offset.is_settled() {
                        offset.advance();
                        broke_early = true;
                        break
                    }
                }
            }
            // }

            if !broke_early {
                break
            }
        }

        assert_eq!(tiles, expected_tiles);
        assert_eq!(mobs, expected_mobs);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum UiMode {
    #[default]
    Select,
    ContextMenuOpen { selection: usize },
    Move { start: QRS },
    Bump { start: QRS, dir: qrs::Dir },
}

fn viable_bump_dirs(tiles: &Tiles, mobs: &Mobs, target: QRS) -> impl Iterator<Item = qrs::Dir> + use<> {
    qrs::Dir::ALL.iter()
        .filter(|&&dir| {
            let at = target.neighbor(dir);

            tiles.get(&at).is_some() && mobs.is_free(at)
        })
        .cloned()
        //Yes this is a technically unneeded allocation. We can care if it ever matters.
        .collect::<Vec<_>>()
        .into_iter()
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
    pub tiles: Tiles,
    pub mobs: Mobs,
    pub selectrum_at: QRS,
    pub ui_mode: UiMode,
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

        for at in qrs::spiral(2, qr!(0, 0)) {
            tiles.insert(
                at,
                Tile {
                    kind: TileKind::ALL[xs::range(rng, 0..TileKind::ALL.len() as u32) as usize],
                    .. <_>::default()
                }
            );
        }

        let start_center = qr!(0, 0);

        let mobs = Mobs::new(start_center);

        Self {
            seed,
            rng: rng_,
            tiles,
            mobs,
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

        for (_, mob) in self.mobs.iter_mut() {
            for offset in &mut mob.offsets {
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

        const MENU_OPTIONS: [(MenuOption, &str); 6] = [
            (MenuOption::Move, "move piece"),
            (MenuOption::Twiddle(Twiddle::OneSixth), "+1/6"),
            (MenuOption::Twiddle(Twiddle::TwoSixths), "+2/6"),
            (MenuOption::Twiddle(Twiddle::ThreeSixths),"+3/6"),
            (MenuOption::Twiddle(Twiddle::MinusTwoSixths), "-2/6"),
            (MenuOption::Twiddle(Twiddle::MinusOneSixths), "-1/6"),
        ];

        // TODO track turns and have CPU players move, and allow the player to only move the player pieces

        let mut player_moved = false;

        macro_rules! move_selectrum {
            () => {
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
                }
            }
        }

        match &mut self.ui_mode {
            UiMode::Select | UiMode::Move { .. } => {
                move_selectrum!();

                if input.pressed_this_frame(Button::A) {
                    match &mut self.ui_mode {
                        UiMode::Move { start } => {
                            let target = self.selectrum_at;

                            if let Some(mob_target) = self.mobs.get_target(*start)
                            && let Some(dir) = qrs::adjacent_dir(
                                qrs::Targeting { source: *start, target }
                            ) {
                                if self.mobs.is_free(target) {                                
                                    self.mobs.apply_dir(mob_target, dir);
                                    // TODO check for whether goal is met, and open door if so
                                    self.ui_mode = UiMode::Select;
                                } else {
                                    self.ui_mode = UiMode::Bump { start: *start, dir };
                                }
                            }
                        },
                        _ => {
                            assert!(matches!(self.ui_mode, UiMode::Select));
                            self.ui_mode = UiMode::ContextMenuOpen { selection: 0 };
                        }
                    }
                } else if input.pressed_this_frame(Button::B) {
                    self.ui_mode = UiMode::Select; // Useful for UiMode::Move
                }
            },
            UiMode::ContextMenuOpen { selection } => {
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
                    match MENU_OPTIONS[*selection].0 {
                        MenuOption::Move => {
                            self.ui_mode = UiMode::Move { start: self.selectrum_at };
                        },
                        MenuOption::Twiddle(twiddle_) => {
                            twiddle(
                                &mut self.tiles,
                                &mut self.mobs,
                                self.selectrum_at,
                                twiddle_,
                            );
                            self.ui_mode = UiMode::Select;
                        },
                    }

                } else if input.pressed_this_frame(Button::B) {
                    self.ui_mode = UiMode::Select;
                }
            },
            UiMode::Bump { start, dir } => {
                move_selectrum!();
                
                let target = start.neighbor(*dir);

                if input.pressed_this_frame(Button::A) {
                    if !player_moved {
                        for bump_dir in viable_bump_dirs(&self.tiles, &self.mobs, target) {
                            if target.neighbor(bump_dir) == self.selectrum_at {
                                // Perform the bump
                                if let Some(bumpee_target) = self.mobs.get_target(target) {
                                    self.mobs.apply_dir(bumpee_target, bump_dir);

                                    if let Some(bumper_target) = self.mobs.get_target(*start) {
                                        self.mobs.apply_dir(bumper_target, *dir);

                                        self.ui_mode = UiMode::Select;
                                    }
                                }

                                break
                            }
                        }
                    }
                } else if input.pressed_this_frame(Button::B) {
                    self.ui_mode = UiMode::Select;
                }
            }
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

            // base
            commands.sspr_override(
                specs.hex_twiddle_tiles.xy_from_tile_sprite(
                    match tile.kind {
                        TileKind::Symbol(_) => 0,
                        TileKind::Warp => specs.hex_twiddle_tiles.tiles_per_row(),
                        TileKind::Split => specs.hex_twiddle_tiles.tiles_per_row() * 2,
                    }
                ),
                command::Rect::from_unscaled(specs.hex_twiddle_tiles.rect(xy)),
                match tile.kind {
                    TileKind::Symbol(_) => 0xFF3352E1,
                    TileKind::Warp => 0xFF3352E1,
                    TileKind::Split => 0xFFDE4949,
                }
            );

            // overlay
            commands.sspr_override(
                specs.hex_twiddle_tiles.xy_from_tile_sprite(
                    match tile.kind {
                        TileKind::Symbol(Symbol::A) => 2,
                        TileKind::Symbol(Symbol::B) => 3,
                        TileKind::Warp => specs.hex_twiddle_tiles.tiles_per_row() + 1,
                        TileKind::Split => specs.hex_twiddle_tiles.tiles_per_row() * 2 + 1,
                    }
                ),
                command::Rect::from_unscaled(specs.hex_twiddle_tiles.rect(xy)),
                match tile.kind {
                    TileKind::Symbol(_) => 0xFF222222,
                    TileKind::Warp => 0xFFDE4949,
                    TileKind::Split => 0xFF30B06E,
                }
            );
        }

        //
        // Render Pieces
        //

        let hex_center_offset = specs.hex_twiddle_tiles.tile() / 2;
        let piece_center_offset = specs.hex_twiddle_pieces.tile() / 2;

        for (qrs, mob) in self.mobs.iter() {
            let mut xy = qrs_to_unscaled(*qrs);
            for offset in mob.offsets {
                xy += offset.xyd();
            }
            xy += hex_center_offset;
            xy -= piece_center_offset;

            commands.sspr(
                specs.hex_twiddle_pieces.xy_from_tile_sprite(mob.sprite),
                command::Rect::from_unscaled(specs.hex_twiddle_pieces.rect(xy)),
            );
        }

        //
        // Render UI
        //

        let selectrum_xy = qrs_to_unscaled(self.selectrum_at);

        macro_rules! draw_selectrum {
            () => {
                commands.sspr_override(
                    specs.hex_twiddle_tiles.xy_from_tile_sprite(SELECTRUM),
                    command::Rect::from_unscaled(specs.hex_twiddle_tiles.rect(selectrum_xy)),
                    0xFFFFB937
                );
            }
        }

        // Context-sensitive UI
        match &mut self.ui_mode {
            UiMode::Select => {
                draw_selectrum!();
            },
            UiMode::ContextMenuOpen{ selection } => {
                draw_selectrum!();

                const OPTION_W: unscaled::W = unscaled::W(120);
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
            UiMode::Move { start } => {
                for dir in qrs::Dir::ALL {
                    let at = qrs_to_unscaled(start.neighbor(dir));
    
                    commands.sspr_override(
                        specs.hex_twiddle_tiles.xy_from_tile_sprite(SELECTRUM),
                        command::Rect::from_unscaled(specs.hex_twiddle_tiles.rect(at)),
                        0xFF30B06E
                    );
                }

                draw_selectrum!();
            },
            UiMode::Bump { start, dir } => {
                let target = start.neighbor(*dir);

                let arrow_sprite: MobSprite = ARROW_BASE + MobSprite::from(dir.index());

                let mut start_xy = qrs_to_unscaled(*start);
                start_xy += hex_center_offset;
                start_xy -= piece_center_offset;
                let mut target_xy = qrs_to_unscaled(target);
                target_xy += hex_center_offset;
                target_xy -= piece_center_offset;

                for dir in viable_bump_dirs(&self.tiles, &self.mobs, target) {
                    let at = qrs_to_unscaled(target.neighbor(dir));
    
                    commands.sspr_override(
                        specs.hex_twiddle_tiles.xy_from_tile_sprite(SELECTRUM),
                        command::Rect::from_unscaled(specs.hex_twiddle_tiles.rect(at)),
                        0xFF30B06E
                    );
                }

                let arrow_xy = unscaled::XY::lerp(start_xy, 0.5, target_xy);
                
                commands.sspr(
                    specs.hex_twiddle_pieces.xy_from_tile_sprite(arrow_sprite),
                    command::Rect::from_unscaled(specs.hex_twiddle_pieces.rect(arrow_xy)),
                );

                draw_selectrum!();
            }
        }
    }
}
