#![deny(unreachable_patterns)]

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

    #[derive(Clone, Copy, Default, PartialEq, Eq)]
    pub struct Offset {
        xyd: unscaled::XYD,
    }

    impl core::fmt::Debug for Offset {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            if self == &Offset::default() {
                write!(f, "Offset::default()")
            } else {
                f.debug_struct("Offset")
                 .field("xyd", &self.xyd)
                 .finish()
            }
        }
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
    const ALL: [Self; 5] = [
        Self::OneSixth,
        Self::TwoSixths,
        Self::ThreeSixths,
        Self::MinusTwoSixths,
        Self::MinusOneSixths,
    ];

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Symbol {
    #[default]
    A,
    B,
}

impl Symbol {
    const ALL: [Symbol; 2] = [
        Symbol::A,
        Symbol::B,
    ];

    fn index(&self) -> usize {
        for i in 0..Self::ALL.len() {
            if Self::ALL[i] == *self {
                return i
            }
        }
        panic!("No index for {self:?} found. Is the ALL missing some values?")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileKind {
    Symbol(Symbol),
    Warp,
    Door,
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
        Self::Door,
    ];

    pub fn index(&self) -> usize {
        // Who care if it's O(n) when n is like 4?
        for (i, other) in Self::ALL.iter().enumerate() {
            if other == self {
                return i
            }
        }
        unreachable!();
    }

    pub fn symbol(&self) -> Option<Symbol> {
        match self {
            Self::Symbol(s) => Some(*s),
            Self::Warp
            | Self::Door => None
        }
    }
}

type Offsets = [Offset; 4];

fn offsets_are_settled(offsets: &Offsets) -> bool {
    offsets.iter().all(|o| o.is_settled())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExitAnimationState {
    #[default]
    HalfOpen,
    OpenFrame1,
    OpenFrame2,
    OpenFrame3,
    OpenFrame4,
    OpenFrame5,
}

impl ExitAnimationState {
    fn advance(&mut self) {
        *self = match *self {
            Self::HalfOpen | Self::OpenFrame5 => Self::OpenFrame1,
            Self::OpenFrame1 => Self::OpenFrame2,
            Self::OpenFrame2 => Self::OpenFrame3,
            Self::OpenFrame3 => Self::OpenFrame4,
            Self::OpenFrame4 => Self::OpenFrame5,
        };
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DoorMode {
    #[default]
    Closed,
    Player(ExitAnimationState),
}

impl DoorMode {
    fn advance(&mut self) {
        match self {
            Self::Closed => {}
            Self::Player(animation_state) => {
                animation_state.advance();
            }
        };
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tile {
    pub kind: TileKind,
    pub offsets: Offsets,
    pub door_mode: DoorMode,
}

impl Tile {
    fn is_door(&self) -> bool {
        self.kind == TileKind::Door
    }
}

pub type Key = QRS;

pub type Tiles = BTreeMap<Key, Tile>;

// 64k tiles ought to be enough for anybody!
type TileCount = u16;

fn is_uncompletable(tiles: &Tiles) -> bool {
    let mut counts: [TileCount; TileKind::ALL.len()] = <_>::default();

    for (_, tile) in tiles {
        counts[tile.kind.index()] = counts[tile.kind.index()].saturating_add(1);
    }

    counts[TileKind::Symbol(Symbol::A).index()] < 1
    || counts[TileKind::Symbol(Symbol::B).index()] < 1
    || counts[TileKind::Warp.index()] == 1 // 0 warps is fine, but one that you can't use is not
    || counts[TileKind::Door.index()] < 1
}

type TileSprite = u8;

const SELECTRUM: TileSprite = 1;

type MobSprite = u16;

const DIR_COUNT: MobSprite = qrs::Dir::ALL.len() as _;

const PLAYER_MAIN_BASE: MobSprite = 0;
const PLAYER_MAIN_LAST: MobSprite = PLAYER_MAIN_BASE + DIR_COUNT - 1;
const PLAYER_HELPER_BASE: MobSprite = PLAYER_MAIN_BASE + DIR_COUNT;
const PLAYER_HELPER_LAST: MobSprite = PLAYER_HELPER_BASE + DIR_COUNT - 1;
const CPU_BASE: MobSprite = PLAYER_HELPER_BASE + DIR_COUNT;
const CPU_LAST: MobSprite = CPU_BASE + DIR_COUNT - 1;
const ARROW_BASE: MobSprite = CPU_BASE + DIR_COUNT;

type Facing = qrs::Dir;

#[derive(Clone, Default, PartialEq, Eq)]
pub enum Action {
    #[default]
    NoOp,
    WarpTo(QRS)
}

impl Action {
    pub fn take(&mut self) -> Self {
        match self {
            Self::NoOp => Self::NoOp,
            Self::WarpTo(qrs) => {
                let output = Self::WarpTo(*qrs);
                *self = Self::NoOp;
                output
            },
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Entity {
    pub offsets: Offsets,
    pub on_offset_done: Action,
    pub sprite: MobSprite,
    pub facing: Facing,
}

impl core::fmt::Debug for Entity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.offsets == Offsets::default() {
            f.debug_struct("Entity")
             .field("offsets", &"Offsets::default()")
             .field("sprite", &self.sprite)
             .field("facing", &self.facing)
             .finish()
        } else {
            f.debug_struct("Entity")
             .field("offsets", &self.offsets)
             .field("sprite", &self.sprite)
             .field("facing", &self.facing)
             .finish()
        }
    }
}

mod mobs {
    use super::*;

    #[repr(u8)]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub enum Index {
        #[default]
        Zero,
        One,
        Two,
    }

    impl Index {
        pub fn wrapping_inc(self) -> Self {
            match self {
                Self::Zero => Self::One,
                Self::One => Self::Two,
                Self::Two => Self::Zero,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Target {
        Player(Index),
        NonPlayer(Index)
    }

    impl Default for Target {
        fn default() -> Self {
            Self::Player(<_>::default())
        }
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
                Target::Player(index) => &$mobs.player_mobs[index as u8 as usize],
                Target::NonPlayer(index) => &$mobs.cpu_mobs[index as u8 as usize],
            }
        };
        (mut $mobs: ident $target: expr) => {
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
            let current = get_ref!(mut self target);

            current.0 = key;
            current.1 = entity;
        }

        pub fn player(&self) -> &(Key, Entity) {
            &self.player_mobs[0]
        }

        pub fn get(&self, target: Target) -> &(Key, Entity) {
            get_ref!(self target)
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
            self.apply_movment(target, dir, None);
        }

        pub fn apply_warp(&mut self, target: Target, dir: qrs::Dir, target_key: Key) {
            self.apply_movment(target, dir, Some(target_key));
        }

        fn apply_movment(&mut self, target: Target, dir: qrs::Dir, warp_target: Option<QRS>) {
            let current = get_ref!(self target);

            let new_qrs = current.0 + QRSD::from(dir);

            if self.is_free(new_qrs) {
                let current = get_ref!(mut self target);

                current.0 = new_qrs;
                current.1.offsets = [offset::direct(dir), Offset::default(), Offset::default(), Offset::default()];
                if let Some(qrs) = warp_target {
                    current.1.on_offset_done = Action::WarpTo(qrs);
                }
                current.1.facing = dir;
            }
        }

        pub fn get_target(&self, key: Key) -> Option<Target> {
            for target in Target::ALL {
                let current = get_ref!(self target);
                if current.0 == key {
                    return Some(target);
                }
            }

            None
        }

        pub fn mutate(&mut self, target: Target, f: impl FnOnce(&mut (Key, Entity))) {
            let current = get_ref!(mut self target);
            f(current);
        }
    }
}
use mobs::Mobs;

fn twiddle(tiles: &mut Tiles, mobs: &mut Mobs, key: Key, twiddle_amount: Twiddle) {
    let base: QRS = key;

    #[derive(Clone, Copy, Debug, Default)]
    struct TwiddleTargeting {
        offsets: Offsets,
        source: QRS,
        target: QRS,
    }

    let mut twiddled: [Option<(TwiddleTargeting, Tile, Option<mobs::Target>)>; qrs::Dir::ALL.len()] = [None; qrs::Dir::ALL.len()];

    let mut dir_i = 0;

    for to_tile_to_move in qrs::Dir::ALL {
        let was_at = base.neighbor(to_tile_to_move);

        if let Some(tile) = tiles.remove(&was_at) {
            let mut targeting = TwiddleTargeting::default();
            targeting.source = was_at;
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

            twiddled[dir_i] = Some((targeting, tile, mobs.get_target(targeting.source)));
        }

        dir_i += 1;
    }

    for opt in twiddled {
        let Some((targeting, mut tile, mob_target_opt)) = opt else { continue };

        tile.offsets = targeting.offsets;
        tiles.insert(targeting.target, tile);

        if let Some(mob_target) = mob_target_opt {
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

        let mut broke_early;
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
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum UiMode {
    #[default]
    Select,
    ContextMenuOpen { selection: usize },
    Move { start: QRS },
    Bump { start: QRS, dir: qrs::Dir },
    Warp { start: QRS, dir: qrs::Dir },
}

fn viable_move_dir(tiles: &Tiles, targeting: qrs::Targeting) -> Option<qrs::Dir> {
    qrs::adjacent_dir(targeting).filter(|_dir| {
        tiles.get(&targeting.target).is_some()
    })
}

fn viable_move_dirs<'tiles>(tiles: &'tiles Tiles, from: QRS) -> impl Iterator<Item = qrs::Dir> + use<'tiles> {
    qrs::Dir::ALL.into_iter().filter(move |dir| {
        tiles.get(&from.neighbor(*dir)).is_some()
    })
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

// The source is the location of the tile that the mob plans to step on.
fn viable_warp_spots<'tiles, 'mobs>(tiles: &'tiles Tiles, mobs: &'mobs Mobs, source: QRS)  -> impl Iterator<Item = qrs::QRS> + use<'tiles, 'mobs> {
    tiles.iter()
        .filter_map(move |(&key, tile)| {
            if key != source
            && tile.kind == TileKind::Warp
            && mobs.is_free(key) {
                Some(key)
            } else {
                None
            }
        })
}

type FrameCount = u64;
type Turn = mobs::Target;

#[derive(Clone, Debug, Default)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
    pub tiles: Tiles,
    pub mobs: Mobs,
    pub selectrum_at: QRS,
    pub ui_mode: UiMode,
    pub frame_count: FrameCount,
    pub turn: Turn,
}

fn next_turn(turn: Turn) -> Turn {
    match turn {
        Turn::Player(index) => Turn::NonPlayer(index),
        Turn::NonPlayer(index) => Turn::Player(index.wrapping_inc()),
    }
}

mod menu_option {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub(crate) enum MenuOption {
        Twiddle(Twiddle),
        Move,
    }

    pub(crate) type Entry = (MenuOption, &'static str);
    
    const FULL_MENU_OPTIONS: [Entry; 6] = [
        (MenuOption::Move, "move piece"),
        (MenuOption::Twiddle(Twiddle::OneSixth), "+1/6"),
        (MenuOption::Twiddle(Twiddle::TwoSixths), "+2/6"),
        (MenuOption::Twiddle(Twiddle::ThreeSixths),"+3/6"),
        (MenuOption::Twiddle(Twiddle::MinusTwoSixths), "-2/6"),
        (MenuOption::Twiddle(Twiddle::MinusOneSixths), "-1/6"),
        //(MenuOption::SkipTurn, "Skip turn"), // Do we need this ever?
    ];
    
    pub(crate) fn get_available_menu_options(
        mobs: &Mobs,
        key: Key,
    ) -> &'static [Entry] {
        if mobs.player().0 == key {
            &FULL_MENU_OPTIONS[..]
        } else {
            &FULL_MENU_OPTIONS[1..]
        }
    }
}
use menu_option::{MenuOption, get_available_menu_options};

impl State {
    pub fn new(rng: &mut Xs, specs: &sprite::Specs) -> Self {
        let seed = xs::new_seed(rng);

        Self::init(seed, specs)
    }

    fn init(seed: Seed, _specs: &sprite::Specs) -> Self {
        let mut rng_ = xs::from_seed(seed);
        let rng = &mut rng_;

        let mut tiles = Tiles::new();
        let mut start_center: QRS = qr!(0, 0);

        let mut tries_left = 16;
        while tries_left > 0 {
            let has_holes = true; //xs::range(rng, 0..2) == 0;

            macro_rules! insert_tile {
                ($at: expr) => {
                    tiles.insert(
                        $at,
                        Tile {
                            kind: TileKind::ALL[xs::range(rng, 0..TileKind::ALL.len() as u32) as usize],
                            .. <_>::default()
                        }
                    );
                }
            }

            for at in qrs::spiral(2, qr!(0, 0)) {
                if has_holes && xs::range(rng, 0..4) == 0 { continue }

                insert_tile!(at);
            }

            start_center = tiles.iter().nth(xs::range(rng, 0..tiles.len() as u32) as usize).map(|t| *t.0).unwrap_or(qr!(0, 0));

            // Ensure the center is surrounded on all sides
            for at in qrs::spiral(1, start_center) {
                if !tiles.contains_key(&at) {
                    insert_tile!(at);
                }
            }

            tries_left -= 1;

            if is_uncompletable(&tiles) {
                tiles.clear();
                //loop again, unless we are out of tries
            } else {
                break
            }
        }

        if is_uncompletable(&tiles) {
            // Known completable fallback
            for (i, at) in qrs::spiral(2, qr!(0, 0)).enumerate() {
                tiles.insert(
                    at,
                    Tile {
                        kind: TileKind::ALL[i % TileKind::ALL.len()],
                        .. <_>::default()
                    }
                );
            }
        }

        let mobs = Mobs::new(start_center);

        let mut output = Self {
            seed,
            rng: rng_,
            tiles,
            mobs,
            .. <_>::default()
        };

        output.sync_doors();

        output
    }

    #[allow(unused)]
    fn restart(&mut self, specs: &sprite::Specs) {
        *self = Self::init(self.seed, specs);
    }

    pub fn all_offsets_settled(&self) -> bool {
        for (_, tile) in &self.tiles {
            if !offsets_are_settled(&tile.offsets) {
                return false
            }
        }

        for (_at, mob) in self.mobs.iter() {
            if !offsets_are_settled(&mob.offsets) {
                return false
            }
        }

        true
    }

    pub fn is_complete(&self) -> bool {
        // If the animations are not settled, delay completion
        if !self.all_offsets_settled() {
            return false
        }

        let (key, _player) = self.mobs.player();

        if let Some(tile) = self.tiles.get(key) {
            return matches!(tile.door_mode, DoorMode::Player(_));
        }

        false
    }

    // TODO? Wrap tiles and mobs in a struct/module to make it impossible to forget to call this?
    fn sync_doors(&mut self) {
        type GoalTracker = [bool; Symbol::ALL.len()];

        let mut player_tracker: GoalTracker = <_>::default();
        let mut cpu_tracker: GoalTracker = <_>::default();

        for (key, mob) in self.mobs.iter() {
            if let Some(symbol) = self.tiles.get(&key).and_then(|t| t.kind.symbol()) {
                let tracker: &mut GoalTracker = match mob.sprite {
                    PLAYER_MAIN_BASE..=PLAYER_MAIN_LAST
                    | PLAYER_HELPER_BASE..=PLAYER_HELPER_LAST
                    => &mut player_tracker,
                    CPU_BASE..=CPU_LAST
                    => &mut cpu_tracker,
                    _ => {
                        debug_assert!(false, "Unexpected mob sprite: {:?}", mob.sprite);
                        continue
                    }
                };

                tracker[symbol.index()] = true;
            }
        }

        // Player wins ties.
        if player_tracker.iter().all(|&b| b) {
            // Open door for player

            for (_, tile) in &mut self.tiles {
                if tile.is_door() && !matches!(tile.door_mode, DoorMode::Player(_)) {
                    tile.door_mode = DoorMode::Player(<_>::default());
                }
            }
        } else if cpu_tracker.iter().all(|&b| b) {
            // Open door for CPUs
            // TODO? What should this case do? Nothing is a potential option.
        } else {
            // Close the doors

            for (_, tile) in &mut self.tiles {
                if tile.is_door() {
                    tile.door_mode = DoorMode::Closed;
                }
            }
        }
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

        for (at, mob) in self.mobs.iter_mut() {
            if offsets_are_settled(&mob.offsets) { continue }

            for offset in &mut mob.offsets {
                if !offset.is_settled() {
                    offset.advance();

                    break
                }
            }

            // We checked before in this loop and it wasn't settled before,
            // so this means "if it just became settled".
            if offsets_are_settled(&mob.offsets) {
                match mob.on_offset_done.take() {
                    Action::NoOp => {}
                    Action::WarpTo(qrs) => {
                        // TODO? Warping animation?
                        *at = qrs;

                        // We don't need to sync doors here unless we add like warp tiles with symbols on them
                        //self.sync_doors();
                    }
                }
            }
        }

        if self.frame_count & 0b1111 == 0 {
            for (_, tile) in &mut self.tiles {
                tile.door_mode.advance();
            }
        }
        self.frame_count = self.frame_count.wrapping_add(1);
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

        

        // TODO either add a way to pan the screen, or ensure that the twiddles that move hexes off screen are not allowed
        //      Current seems like a pan control on the side that you can move the selectrix to would make sense
        //          Simplest to implement design is probably 4 (6?) buttons
        //          Another option would be like a virtual joystick thing that you can press A to grip and then move around smoothly
        //          Could put in both?

        if self.all_offsets_settled() {
            match self.turn {
                // The player
                Turn::Player(mobs::Index::Zero) => {
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
                                player_moved = true;
                                self.selectrum_at = target_qrs;
                            } else if input.pressed_this_frame(Button::DOWN) {
                                let dir = if input.gamepad.contains(Button::LEFT) {
                                    qrs::Dir::DecQIncR
                                } else if input.gamepad.contains(Button::RIGHT) {
                                    qrs::Dir::DecSIncQ
                                } else {
                                    qrs::Dir::DecSIncR
                                };

                                let target_qrs = self.selectrum_at.neighbor(dir);
                                player_moved = true;
                                self.selectrum_at = target_qrs;
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
                                        && mob_target == self.turn
                                        && let Some(dir) = viable_move_dir(&self.tiles, qrs::Targeting { source: *start, target }) {
                                            if self.mobs.is_free(target) && self.tiles.get(&target).map(|t| t.kind) == Some(TileKind::Warp) {
                                                self.ui_mode = UiMode::Warp { start: *start, dir };
                                            } else if self.mobs.is_free(target) {
                                                self.mobs.apply_dir(mob_target, dir);
                                                // TODO? More/variable goals?
                                                // TODO? Only one space of each symbol?

                                                self.sync_doors();
                                                self.turn = next_turn(self.turn);

                                                self.ui_mode = UiMode::Select;
                                            } else {
                                                self.ui_mode = UiMode::Bump { start: *start, dir };
                                            }
                                        }
                                    },
                                    _ => {
                                        assert!(matches!(self.ui_mode, UiMode::Select));
                                        if self.tiles.get(&self.selectrum_at).is_some() {
                                            self.ui_mode = UiMode::ContextMenuOpen { selection: 0 };
                                        }
                                    }
                                }
                            } else if input.pressed_this_frame(Button::B) {
                                self.ui_mode = UiMode::Select; // Useful for UiMode::Move
                            }
                        },
                        UiMode::ContextMenuOpen { selection } => {
                            let menu_options = get_available_menu_options(&self.mobs, self.selectrum_at);

                            // TODO disallow the move piece option if there is no player piece there
                            if input.pressed_this_frame(Button::UP) {
                                if *selection == 0 {
                                    *selection = menu_options .len();
                                }
                                *selection -= 1;
                            } else if input.pressed_this_frame(Button::DOWN) {
                                *selection += 1;
                                if *selection == menu_options .len() {
                                    *selection = 0;
                                }
                            } else if input.pressed_this_frame(Button::A) {
                                match menu_options [*selection].0 {
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

                                        self.sync_doors();
                                        self.turn = next_turn(self.turn);

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

                                                    self.sync_doors();
                                                    self.turn = next_turn(self.turn);

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
                        UiMode::Warp { start, dir } => {
                            move_selectrum!();

                            let target = start.neighbor(*dir);

                            if input.pressed_this_frame(Button::A) {
                                if !player_moved {
                                    if let Some(mob_target) = self.mobs.get_target(*start) {
                                        let mut warp_target = None;

                                        for qrs in viable_warp_spots(&self.tiles, &self.mobs, target) {
                                            if qrs == self.selectrum_at && self.mobs.is_free(qrs) {
                                                warp_target = Some(qrs);

                                                break
                                            }
                                        }

                                        if let Some(qrs) = warp_target {
                                            self.mobs.apply_warp(mob_target, *dir, qrs);

                                            self.sync_doors();
                                            self.turn = next_turn(self.turn);

                                            self.ui_mode = UiMode::Select;
                                        }
                                    }
                                }
                            } else if input.pressed_this_frame(Button::B) {
                                self.ui_mode = UiMode::Select;
                            }
                        }
                    }
                }
                other => {
                    let mob_target = self.turn;
                    let (mob_at, entity) = self.mobs.get(mob_target);

                    // TODO have the choices be made with more purpose
                    //     Player allies should move towards the goal piece if the doors are not open
                    //         Otherwise, twiddle something that traps the enemy pieces
                    //     Player enemies should attempt each of the following, in this order, until one is possible:
                    //        * move to bump the player/player allies off of the goal tiles
                    //        * move to a space where they can bump usefully next turn
                    //            * bump the other piece to a spot where they can? Maybe only if next in turn order relative to player?
                    //        * twiddle to trap the player away from an exit
                    //        * twiddle to trap 
                    // If after all that is implemented, it still seems too easy to win, add enemy doors and have them be able to escape
                    //    Should make the graphics clear, with more than just colour, somehow
                    //    Will need to figure out where in the move goal order trying to win should be.
                    //        Likely still after trying to prevent the other player from winning in at least a turn or two
                    //    Should revisit player ally logic at that point, to balance tryign to win vs trying to prevent enemies winning
                    //
                    // What is the best way to compute the predicates over the different moves?
                    // * Option one: Compute all possible moves, and the state once they are done, check first predicate with early out, then loop again with second predicate, etc.
                    // * Option two: Check each move, computing the states as needed, calculating all the predicates as we go, retaining only the answer and the move, so only need one extra state in memory?
                    // 
                    enum MoveSelection {
                        NoMove,
                        Dir(qrs::Dir),
                        Warp(qrs::Dir, QRS),
                        Bump(qrs::Dir, (mobs::Target, qrs::Dir)),
                        Twiddle(QRS, Twiddle),
                    }

                    let mut move_selection = MoveSelection::NoMove;

                    if xs::range(&mut self.rng, 0..6) == 0 {
                        let random_index = xs::index(&mut self.rng, 0..self.tiles.len());

                        if let Some(qrs) = self.tiles.keys().nth(random_index) {
                            move_selection = MoveSelection::Twiddle(
                                *qrs,
                                Twiddle::ALL[xs::index(&mut self.rng, 0..Twiddle::ALL.len())]
                            );
                        }
                    } else {
                        let move_dirs = viable_move_dirs(&self.tiles, *mob_at).collect::<Vec<_>>();
                        let move_dir_offset = xs::index(&mut self.rng, 0..move_dirs.len());

                        'move_dir: for i in 0..move_dirs.len() {
                            let dir = move_dirs[(i + move_dir_offset) % move_dirs.len()];
                            let target = mob_at.neighbor(dir);

                            if self.mobs.is_free(target)
                            && self.tiles.get(&target).map(|t| t.kind) == Some(TileKind::Warp) {
                                let warp_spots = viable_warp_spots(&self.tiles, &self.mobs, target).collect::<Vec<_>>();
                                let warp_spots_offset = xs::index(&mut self.rng, 0..warp_spots.len());

                                for i in 0..warp_spots.len() {
                                    let qrs = warp_spots[(i + warp_spots_offset) % warp_spots.len()];
                                    if self.mobs.is_free(qrs) {
                                        move_selection = MoveSelection::Warp(dir, qrs);

                                        break 'move_dir
                                    }
                                }

                            } else if self.mobs.is_free(target) {
                                move_selection = MoveSelection::Dir(dir);

                                break 'move_dir
                            } else {
                                if let Some(bumpee_target) = self.mobs.get_target(target) {
                                    let bump_dirs = viable_bump_dirs(&self.tiles, &self.mobs, target).collect::<Vec<_>>();
                                    let bump_dir_index = xs::index(&mut self.rng, 0..bump_dirs.len());
    
                                    let bump_dir = bump_dirs[bump_dir_index];
                                    
                                    move_selection = MoveSelection::Bump(dir, (bumpee_target, bump_dir));
                                }
                            }
                        }
                    }

                    match move_selection {
                        MoveSelection::NoMove => {} // Must skip turn.
                        MoveSelection::Dir(dir) => {
                            self.mobs.apply_dir(mob_target, dir);
                        }
                        MoveSelection::Warp(dir, qrs) => {
                            self.mobs.apply_warp(mob_target, dir, qrs);
                        }
                        MoveSelection::Bump(dir, (bumpee_target, bump_dir)) => {
                            self.mobs.apply_dir(bumpee_target, bump_dir);

                            // mob_target points at the bumper
                            self.mobs.apply_dir(mob_target, dir);
                        }
                        MoveSelection::Twiddle(qrs, twiddle_) => {
                            twiddle(
                                &mut self.tiles,
                                &mut self.mobs,
                                qrs,
                                twiddle_,
                            );
                        }
                    }

                    self.sync_doors();
                    self.turn = next_turn(self.turn);
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

        // TODO? Display whose turn it is?

        fn tile_xy(qrs: QRS, Tile { offsets, .. }: &Tile) -> unscaled::XY {
            let mut output = qrs_to_unscaled(qrs);

            for offset in offsets {
                output += offset.xyd();
            }

            output
        }

        macro_rules! draw_tile {
            ($sprite: expr, $xy: expr, $colour: expr $(,)?) => ({
                let sprite: TileSprite = $sprite;

                commands.sspr_override(
                    specs.hex_twiddle_tiles.xy_from_tile_sprite(sprite),
                    command::Rect::from_unscaled(specs.hex_twiddle_tiles.rect($xy)),
                    $colour
                );
            })
        }

        //
        // Render Tiles
        //

        for (at, tile) in self.tiles.iter() {
            let xy = tile_xy(*at, &tile);

            match tile.kind {
                TileKind::Symbol(symbol) => {
                    draw_tile!(0, xy, 0xFF3352E1);
                    draw_tile!(
                        match symbol {
                            Symbol::A => 2,
                            Symbol::B => 3,
                        },
                        xy,
                        0xFF222222
                    );
                },
                TileKind::Warp => {
                    draw_tile!(
                        specs.hex_twiddle_tiles.tiles_per_row(),
                        xy,
                        0xFF3352E1,
                    );
                    draw_tile!(
                        specs.hex_twiddle_tiles.tiles_per_row() + 1,
                        xy,
                        0xFFDE4949,
                    );
                },
                TileKind::Door => {
                    draw_tile!(
                        specs.hex_twiddle_tiles.tiles_per_row() * 2,
                        xy,
                        0xFFDE4949,
                    );

                    macro_rules! open_background {
                        () => {
                            draw_tile!(
                                specs.hex_twiddle_tiles.tiles_per_row() * 2 + 1,
                                xy,
                                0xFF222222,
                            );
                        }
                    }

                    match tile.door_mode {
                        DoorMode::Closed => {
                            draw_tile!(
                                specs.hex_twiddle_tiles.tiles_per_row() + 2,
                                xy,
                                0xFF5A7D8B,
                            );
                            draw_tile!(
                                specs.hex_twiddle_tiles.tiles_per_row() + 3,
                                xy,
                                0xFFFFB937,
                            );
                        }
                        DoorMode::Player(ExitAnimationState::HalfOpen) => {
                            draw_tile!(
                                specs.hex_twiddle_tiles.tiles_per_row() + 4,
                                xy,
                                0xFF5A7D8B,
                            );
                            draw_tile!(
                                specs.hex_twiddle_tiles.tiles_per_row() + 5,
                                xy,
                                0xFFFFB937,
                            );
                        }
                        DoorMode::Player(ExitAnimationState::OpenFrame1) => {
                            open_background!();
                        }
                        DoorMode::Player(ExitAnimationState::OpenFrame2) => {
                            open_background!();
                            draw_tile!(
                                specs.hex_twiddle_tiles.tiles_per_row() * 2 + 2,
                                xy,
                                0xFFFFB937,
                            );
                        }
                        DoorMode::Player(ExitAnimationState::OpenFrame3) => {
                            open_background!();
                            draw_tile!(
                                specs.hex_twiddle_tiles.tiles_per_row() * 2 + 3,
                                xy,
                                0xFFFFB937,
                            );
                        }
                        DoorMode::Player(ExitAnimationState::OpenFrame4) => {
                            open_background!();
                            draw_tile!(
                                specs.hex_twiddle_tiles.tiles_per_row() * 2 + 4,
                                xy,
                                0xFFFFB937,
                            );
                        }
                        DoorMode::Player(ExitAnimationState::OpenFrame5) => {
                            open_background!();
                            draw_tile!(
                                specs.hex_twiddle_tiles.tiles_per_row() * 2 + 5,
                                xy,
                                0xFFFFB937,
                            );
                        }
                    }
                },
            }
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
                specs.hex_twiddle_pieces.xy_from_tile_sprite(mob.sprite + mob.facing.index() as MobSprite),
                command::Rect::from_unscaled(specs.hex_twiddle_pieces.rect(xy)),
            );
        }

        //
        // Render UI
        //

        let selectrum_xy = qrs_to_unscaled(self.selectrum_at);

        macro_rules! draw_selectrum {
            () => {
                draw_tile!(
                    SELECTRUM,
                    selectrum_xy,
                    if self.tiles.get(&self.selectrum_at).is_some() { 0xFFFFB937 } else { 0xFFDE4949 },
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

                let menu_options = get_available_menu_options(&self.mobs, self.selectrum_at);

                const OPTION_W: unscaled::W = unscaled::W(120);
                const OPTION_H: unscaled::H = unscaled::H(25);

                commands.nine_slice(
                    gfx::nine_slice::CONTEXT_MENU,
                    unscaled::Rect {
                        x: selectrum_xy.x,
                        y: selectrum_xy.y,
                        w: OPTION_W,
                        h: OPTION_H * menu_options.len() as _,
                    },
                );

                let mut at = selectrum_xy;

                for i in 0..menu_options.len() {
                    commands.print_line(
                        menu_options[i].1.as_ref(),
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
                    let target = start.neighbor(dir);

                    if let Some(viable_dir) = viable_move_dir(&self.tiles, qrs::Targeting { source: *start, target }) {
                        assert_eq!(viable_dir, dir);
                        let at = qrs_to_unscaled(target);

                        draw_tile!(
                            SELECTRUM,
                            at,
                            0xFF30B06E
                        );
                    }
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
            },
            UiMode::Warp { start, dir } => {
                let target = start.neighbor(*dir);

                let arrow_sprite: MobSprite = ARROW_BASE + MobSprite::from(dir.index());

                let mut start_xy = qrs_to_unscaled(*start);
                start_xy += hex_center_offset;
                start_xy -= piece_center_offset;
                let mut target_xy = qrs_to_unscaled(target);
                target_xy += hex_center_offset;
                target_xy -= piece_center_offset;

                for spot in viable_warp_spots(&self.tiles, &self.mobs, target) {
                    let at = qrs_to_unscaled(spot);

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
