use gfx::{Commands, AddDrawCommands};
use gfx_sizes::{ARGB};
use platform_types::{command, sprite, unscaled, Button, Dir, Input, Speaker};
//use vec1::{Grid1, Grid1Spec};
use xs::{Seed, Xs};

type Index = usize;
type Distance = qrs::Distance;

const TAU: f32 = core::f32::consts::TAU;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum CardColour {
    #[default]
    Blue,
    Green,
    Red,
    Yellow,
    Purple,
    Cyan,
}

impl CardColour {
    const ALL: [Self; 6] = [
        Self::Blue,
        Self::Green,
        Self::Red,
        Self::Yellow,
        Self::Purple,
        Self::Cyan,
    ];

    fn index(self) -> u16 {
        match self {
            CardColour::Blue => 0,
            CardColour::Green => 1,
            CardColour::Red => 2,
            CardColour::Yellow => 3,
            CardColour::Purple => 4,
            CardColour::Cyan => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum CardSymbol {
    #[default]
    None,
    OnePip,
    TwoPips,
}

impl CardSymbol {
    const ALL: [Self; 3] = [
        Self::None,
        Self::OnePip,
        Self::TwoPips,
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CardKind {
    colour: CardColour,
    symbol: CardSymbol,
}

type InventoryItem = CardKind;

#[derive(Clone, Debug, Default)]
pub struct Inventory {
    cells: Vec<InventoryItem>,
    index: Index,
}

mod world {
    pub use platform_types::{unscaled::{W, H, XD, YD, XYD}};
    use qrs::QRS;

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

    macro_rules! signed_paired_impls {
        ($($a_name: ident, $b_name: ident, $a_inner: ident)+) => {$(
            impl core::ops::AddAssign<$b_name> for $a_name {
                fn add_assign(&mut self, other: $b_name) {
                    if other.0 < 0 {
                        // Adding a negative by subtracting the absolute value
                        self.0 -= (other.0.abs()) as $a_inner;
                    } else if other.0 > 0 {
                        self.0 += other.0 as $a_inner;
                    } else {
                        // Nothing to do
                    }
                }
            }

            impl core::ops::Add<$b_name> for $a_name {
                type Output = Self;

                fn add(mut self, other: $b_name) -> Self::Output {
                    self += other;
                    self
                }
            }

            impl core::ops::SubAssign<$b_name> for $a_name {
                fn sub_assign(&mut self, other: $b_name) {
                    if other.0 < 0 {
                        // Subtracting a negative by adding the absolute value
                        self.0 += (other.0.abs()) as $a_inner;
                    } else if other.0 > 0 {
                        self.0 -= other.0 as $a_inner;
                    } else {
                        // Nothing to do
                    }
                }
            }

            impl core::ops::Sub<$b_name> for $a_name {
                type Output = Self;

                fn sub(mut self, other: $b_name) -> Self::Output {
                    self -= other;
                    self
                }
            }
        )+}
    }

    signed_paired_impls!{
        X, XD, Inner
        Y, YD, Inner
    }

    impl core::ops::Sub<X> for X {
        type Output = XD;

        fn sub(self, other: X) -> Self::Output {
            XD(self.0 - other.0)
        }
    }

    impl core::ops::Sub<Y> for Y {
        type Output = YD;

        fn sub(self, other: Y) -> Self::Output {
            YD(self.0 - other.0)
        }
    }

    impl core::ops::Sub for XY {
        type Output = XYD;

        fn sub(self, other: XY) -> Self::Output {
            XYD {
                xd: XD(self.x.0 as Inner - other.x.0 as Inner),
                yd: YD(self.y.0 as Inner - other.y.0 as Inner),
            }
        }
    }

    macro_rules! shared_delta_impl {
        ($($name: ident $component_1: ident : $type_1: ident  $component_2: ident : $type_2: ident $inner: ident),+ $(,)?) => {
            $(
                impl core::ops::AddAssign<$name> for XY {
                    fn add_assign(&mut self, other: $name) {
                        self.x += other.$component_1;
                        self.y += other.$component_2;
                    }
                }

                impl core::ops::Add<$name> for XY {
                    type Output = Self;

                    fn add(mut self, other: $name) -> Self::Output {
                        self += other;
                        self
                    }
                }

                impl core::ops::SubAssign<$name> for XY {
                    fn sub_assign(&mut self, other: $name) {
                        self.x -= other.$component_1;
                        self.y -= other.$component_2;
                    }
                }

                impl core::ops::Sub<$name> for XY {
                    type Output = Self;

                    fn sub(mut self, other: $name) -> Self::Output {
                        self -= other;
                        self
                    }
                }

                impl core::ops::AddAssign<$type_1> for XY {
                    fn add_assign(&mut self, other: $type_1) {
                        self.x += other;
                    }
                }

                impl core::ops::Add<$type_1> for XY {
                    type Output = Self;

                    fn add(mut self, other: $type_1) -> Self::Output {
                        self += other;
                        self
                    }
                }

                impl core::ops::SubAssign<$type_1> for XY {
                    fn sub_assign(&mut self, other: $type_1) {
                        self.x -= other;
                    }
                }

                impl core::ops::Sub<$type_1> for XY {
                    type Output = Self;

                    fn sub(mut self, other: $type_1) -> Self::Output {
                        self -= other;
                        self
                    }
                }

                impl core::ops::AddAssign<$type_2> for XY {
                    fn add_assign(&mut self, other: $type_2) {
                        self.y += other;
                    }
                }

                impl core::ops::Add<$type_2> for XY {
                    type Output = Self;

                    fn add(mut self, other: $type_2) -> Self::Output {
                        self += other;
                        self
                    }
                }

                impl core::ops::SubAssign<$type_2> for XY {
                    fn sub_assign(&mut self, other: $type_2) {
                        self.y -= other;
                    }
                }

                impl core::ops::Sub<$type_2> for XY {
                    type Output = Self;

                    fn sub(mut self, other: $type_2) -> Self::Output {
                        self -= other;
                        self
                    }
                }
            )+
        }
    }

    shared_delta_impl!{
        XYD xd: XD yd: YD Inner,
    }

    pub const fn x_const_add_w(x: X, w: W) -> X {
        X(x.0 + w.get())
    }

    pub const fn y_const_add_h(y: Y, h: H) -> Y {
        Y(y.0 + h.get())
    }

    const X_Q_FACTOR: Inner = 2;
    const Y_Q_FACTOR: Inner = 0;

    const X_R_FACTOR: Inner = 1;
    const Y_R_FACTOR: Inner = 2;

    const HEX_X_SCALE: Inner = 3;
    const HEX_Y_SCALE: Inner = 2;

    const HEX_X_OFFSET: Inner = 0;
    const HEX_Y_OFFSET: Inner = 0;

    fn qrs_to_world(qrs: QRS) -> XY {
        let q = qrs.q.0;
        let r = qrs.r.0;

        let x = (X_Q_FACTOR * q + X_R_FACTOR * r) * HEX_X_SCALE + HEX_X_OFFSET;
        let y = (Y_Q_FACTOR * q + Y_R_FACTOR * r) * HEX_Y_SCALE + HEX_Y_OFFSET;

        XY {
            x: X(x.try_into().unwrap_or(0)),
            y: Y(y.try_into().unwrap_or(0)),
        }
    }

    pub fn sprial_iter(
        radius: qrs::Distance,
        center: XY,
    ) -> impl Iterator<Item = XY> {
        let offset = center - XY::default();

        qrs::spiral(radius, <_>::default())
            .map(move |qrs| qrs_to_world(qrs) + offset)
    }
}

const LIGHT_COUNT: u8 = 3;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum LockLightState {
    #[default]
    Off,
    Correct,
    Wrong
}

// Use a type alias in case we want to support like "any blue card" etc. later
type LockMatcher = CardKind;

#[derive(Copy, Clone, Debug, Default)]
pub struct LockLight {
    state: LockLightState,
    matcher: LockMatcher,
}

type LightsSpec<'spec> = &'spec [LockLight];

#[derive(Clone, Debug, Default)]
struct Lights {
    lights: [LockLight; LIGHT_COUNT as usize],
    length: u8,
}

impl Lights {
    fn is_empty(&self) -> bool {
        self.length == 0
    }

    fn iter(&self) -> impl Iterator<Item = &LockLight> {
        self.lights[0..usize::from(self.length)].iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut LockLight> {
        self.lights[0..usize::from(self.length)].iter_mut()
    }
}

impl From<LightsSpec<'_>> for Lights {
    fn from(spec: LightsSpec<'_>) -> Self {
        let len = spec.len();
        assert!(len <= 3);
        assert!(len <= u8::MAX as usize);

        let mut lights: [LockLight; LIGHT_COUNT as usize] = <_>::default();

        for i in 0..len {
            lights[i] = spec[i];
        }

        Self {
            lights,
            length: len as u8
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum Reward {
    #[default]
    Win,
    Item(InventoryItem)
}

#[derive(Clone, Debug, Default)]
pub struct Lock {
    xy: world::XY,
    lights: Lights,
    reward: Option<Reward>,
}

type LightIndex = usize;

impl Lock {
    fn is_open(&self) -> bool {
        // Assume true so a 0 lights lock is always open
        let mut output = true;
        for light in self.lights.iter() {
            if light.state != LockLightState::Correct {
                output = false;
            }
        }
        output
    }

    fn matching_light_mut(&mut self, matcher: LockMatcher) -> Option<&mut LockLight> {
        for light in self.lights.iter_mut() {
            if light.state != LockLightState::Correct {
                if light.matcher == matcher {
                    return Some(light)
                }
            }
        }

        None
    }
}

#[derive(Clone, Debug, Default)]
pub struct Locks {
    locks: Vec<Lock>,
    index: Index,
}

type FrameCount = u16;

const MAX_INSERT_FRAME: FrameCount = 60;
const MAX_INSIDE_FRAME: FrameCount = 20;
const MAX_REMOVE_FRAME: FrameCount = 60;

#[derive(Clone, Debug)]
pub enum LockAnimationState {
    Insert(FrameCount),
    Inside(FrameCount),
    Remove(FrameCount),
}

impl Default for LockAnimationState {
    fn default() -> Self { LockAnimationState::Insert(0) }
}

#[derive(Clone, Debug, Default)]
pub struct LockAnimation {
    state: LockAnimationState,
    inventory_index: Index,
    lock_index: Index,
}

#[derive(Clone, Debug, Default)]
pub struct Animations {
    lock: Option<LockAnimation>,
}

const FLAG_ZERO_FRAMES: FrameCount = 45;

#[derive(Clone, Debug)]
pub enum FlagState {
    Zero(FrameCount),
    One(FrameCount),
    Two(FrameCount),
    Three(FrameCount),
}

impl Default for FlagState {
    fn default() -> Self {
        Self::Zero(FLAG_ZERO_FRAMES)
    }
}

const CARD_X_MIN: unscaled::X = unscaled::X(((command::WIDTH_SIGNED / 4) * 3) - 40);

const INVENTORY_OUTER_RECT: unscaled::Rect = unscaled::Rect {
    x: unscaled::X(0),
    y: unscaled::Y(command::HEIGHT_SIGNED / 2),
    w: unscaled::W::new(command::WIDTH_SIGNED),
    h: unscaled::H::new(command::HEIGHT_SIGNED / 2),
};

const SPACING: unscaled::Inner = 4;

const LOCK_SCENE_OUTER_RECT: unscaled::Rect = unscaled::Rect {
    x: unscaled::X(0),
    y: unscaled::Y(0),
    w: unscaled::W::new(CARD_X_MIN.get() - SPACING),
    h: unscaled::H::new(INVENTORY_OUTER_RECT.y.get() - SPACING),
};

// This kinda cheats to make things fit, by knowing that we are unlikely to change sizes at the moment
// but, if we make it change more often, we have implmented basic scrolling that should keep things playable.
const MAP_WH: unscaled::WH = unscaled::WH {
    w: unscaled::w_const_div(unscaled::w_const_mul(LOCK_SCENE_OUTER_RECT.w, 3), 4),
    h: unscaled::h_const_div(unscaled::h_const_mul(LOCK_SCENE_OUTER_RECT.h, 3), 4),
};

const MAP_CENTER: world::XY = world::XY {
    x: world::x_const_add_w(world::X(0), MAP_WH.w.halve()),
    y: world::y_const_add_h(world::Y(0), MAP_WH.h.halve()),
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum UiSection {
    #[default]
    Map,
    Inventory,
    AboveMap,
    AboveInventory,
}

#[derive(Clone, Debug, Default)]
pub struct Splotch {
    xy: world::XY,
    colour: CardColour,
    radius: Distance,
}

const MAX_SPLOTCH_COUNT: u8 = 6;

type Splotches = [Splotch; MAX_SPLOTCH_COUNT as usize];

#[derive(Clone, Debug, Default)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
    pub inventory: Inventory,
    pub inventory_scroll: unscaled::XYD,
    pub locks: Locks,
    pub world_scroll: unscaled::XYD,
    pub animations: Animations,
    pub flag_state: FlagState,
    pub ui_section: UiSection,
    pub splotches: Splotches,
    pub won: bool,
}

impl State {
    pub fn new(rng: &mut Xs, specs: &sprite::Specs) -> Self {
        let seed = xs::new_seed(rng);

        Self::init(seed, specs)
    }

    fn init(seed: Seed, _specs: &sprite::Specs) -> Self {
        let mut rng_ = xs::from_seed(seed);
        let rng = &mut rng_;

        // Generate all the splotches first.
        let mut splotches = Splotches::default();
        {
            let x_radius = MAP_WH.w.halve().get() as f32 * 0.9;
            let y_radius = MAP_WH.h.halve().get() as f32 * 0.9;

            let ring_length = (MAX_SPLOTCH_COUNT - 1) as usize;

            let center_index = 0;

            splotches[center_index] = Splotch {
                xy: MAP_CENTER,
                colour: CardColour::ALL[CardColour::ALL.len() - 1],
                radius: 6,
            };

                    let first_ring_index = center_index + 1;

            // Ring second so it is drawn on top of the center
            for raw_i in 0..ring_length {
                let i = raw_i + first_ring_index;
                let angle = TAU * i as f32 / ring_length as f32;

                splotches[i] = Splotch {
                    xy: world::XY {
                        x: MAP_CENTER.x + world::XD((x_radius * f32::cos(angle)) as world::Inner),
                        y: MAP_CENTER.y + world::YD((y_radius * f32::sin(angle)) as world::Inner),
                    },
                    colour: CardColour::ALL[raw_i],
                    radius: 8,
                }
            }
        }

        // Shuffle up a deck of cards to draw from.
        let mut deck = Vec::with_capacity(CardColour::ALL.len() * CardSymbol::ALL.len());

        for colour in CardColour::ALL {
            for symbol in CardSymbol::ALL {
                deck.push(
                    CardKind { colour, symbol },
                );
            }
        }

        xs::shuffle(rng, &mut deck);

        // Pick a random spot for the final reward.
        let win_xy = world::XY {
            x: world::X(xs::range(rng, 0..MAP_WH.w.get() as u32) as unscaled::Inner),
            y: world::Y(xs::range(rng, 0..MAP_WH.h.get() as u32) as unscaled::Inner),
        };

        let mut lock_xy = win_xy;
        let mut light_specs = Vec::with_capacity(3);

        use std::collections::VecDeque;

        let mut to_place = VecDeque::with_capacity(deck.len());

        struct PlacementState {
            deck: Vec<CardKind>,
            to_place: VecDeque<Reward>,
            light_specs: Vec<LockLight>,
            lock_xy: world::XY,
        }

        let mut placement_state = PlacementState {
            deck,
            to_place,
            light_specs,
            lock_xy,
        };

        fn place_lock_for_reward(placement_state: &mut PlacementState, locks: &mut Vec<Lock>, rng: &mut Xs, reward: Reward) {
            // Place the passed in reward
            // TODO pull out cards based on the colour of where the xy is
            while placement_state.light_specs.len() != 3 && !placement_state.deck.is_empty() {
                let card = placement_state.deck.pop().expect("We just checked that the deck isn't empty!");

                placement_state.light_specs.push(LockLight { matcher: card, state: <_>::default() });

                // TODO? randomize the ordering to produce different tree shapes?
                placement_state.to_place.push_back(Reward::Item(card));
            }

            let lights: Lights = (&placement_state.light_specs[..]).into();
            placement_state.light_specs.clear();

            let lock = Lock {
                xy: placement_state.lock_xy,
                lights,
                reward: Some(reward),
            };

            locks.push(lock);

            placement_state.lock_xy = world::XY {
                x: world::X(xs::range(rng, 0..MAP_WH.w.get() as u32) as unscaled::Inner),
                y: world::Y(xs::range(rng, 0..MAP_WH.h.get() as u32) as unscaled::Inner),
            };

            // Place rewards needed to unlock previously placed locks
            while let Some(stack_reward) = placement_state.to_place.pop_front() {
                place_lock_for_reward(placement_state, locks, rng, stack_reward);
            }
        }

        let mut locks = Locks::default();

        place_lock_for_reward(&mut placement_state, &mut locks.locks, rng, Reward::Win);

        // TODO
        // Put the leftovers as free cards
        // ... or do leftovers show up?
        assert!(placement_state.to_place.is_empty());
        assert!(placement_state.light_specs.is_empty());

        locks.locks.sort_by_key(|l| l.xy);

        let mut inventory = Inventory {
            cells: Vec::with_capacity(CardColour::ALL.len() * CardSymbol::ALL.len()),
            index: 0,
        };

        Self {
            seed,
            rng: rng_,
            inventory,
            locks,
            splotches,
            .. <_>::default()
        }
    }

    #[allow(unused)]
    fn restart(&mut self, specs: &sprite::Specs) {
        *self = Self::init(self.seed, specs);
    }

    fn all_offsets_settled(&self) -> bool {
        true
    }

    pub fn is_complete(&self) -> bool {
        // If the animations are not settled, delay completion
        if !self.all_offsets_settled() {
            return false
        }

        self.won
    }

    fn apply_reward(&mut self, reward: Reward) {
        match reward {
            Reward::Win => { self.won = true },
            Reward::Item(card) => {
                // TODO maintain sorted order?
                self.inventory.cells.push(card);
                self.inventory.index = self.inventory.cells.len() - 1;
            },
        }
    }

    fn tick(&mut self) {
        // Advance animations
        // We can make an iterator if we actually need at least 3 distinct animations that are handled the same.
        if let Some(animation) = &mut self.animations.lock {
            match &mut animation.state {
                LockAnimationState::Insert(at_frame) => {
                    *at_frame += 1;
                    if *at_frame > MAX_INSERT_FRAME {
                        // SFX Good place for click sound effect
                        animation.state = LockAnimationState::Inside(0);
                    }
                },
                LockAnimationState::Inside(at_frame) => {
                    *at_frame += 1;
                    if *at_frame > MAX_INSIDE_FRAME {
                        // SFX Good place for click sound effect
                        animation.state = LockAnimationState::Remove(0);

                        if let (Some(lock), Some(card)) =
                            (
                                self.locks.locks.get_mut(animation.lock_index),
                                self.inventory.cells.get(animation.inventory_index)
                            )
                        {
                            if let Some(light) = lock.matching_light_mut(*card) {
                                light.state = LockLightState::Correct;
                            } else {
                                for light in lock.lights.iter_mut() {
                                    if light.state != LockLightState::Correct {
                                        light.state = LockLightState::Wrong;
                                    }
                                }
                            }
                        }
                    }
                },
                LockAnimationState::Remove(at_frame) => {
                    *at_frame += 1;
                    if *at_frame > MAX_REMOVE_FRAME {
                        let mut reward = None;

                        if let (Some(lock), Some(card)) =
                            (
                                self.locks.locks.get_mut(animation.lock_index),
                                self.inventory.cells.get(animation.inventory_index)
                            )
                        {
                            // Dispense reward

                            if lock.is_open() {
                                reward = lock.reward.take();
                            }

                            // Reset back to off
                            // Not sure if this is the right place to do that

                            for light in lock.lights.iter_mut() {
                                if light.state != LockLightState::Correct {
                                    light.state = LockLightState::Off;
                                }
                            }
                        }

                        if let Some(reward) = reward {
                            self.apply_reward(reward);
                        }

                        // Removal
                        self.animations.lock = None;
                    }
                },
            };
        }

        self.flag_state = match self.flag_state {
            FlagState::Zero(0) => FlagState::One(FLAG_ZERO_FRAMES),
            FlagState::Zero(frames) => FlagState::Zero(frames - 1),
            FlagState::One(0) => FlagState::Two(FLAG_ZERO_FRAMES),
            FlagState::One(frames) => FlagState::One(frames - 1),
            FlagState::Two(0) => FlagState::Three(FLAG_ZERO_FRAMES),
            FlagState::Two(frames) => FlagState::Two(frames - 1),
            FlagState::Three(0) => FlagState::Zero(FLAG_ZERO_FRAMES),
            FlagState::Three(frames) => FlagState::Three(frames - 1),
        };
    }

    pub fn update_and_render(
        &mut self,
        mut commands: &mut Commands,
        specs: &sprite::Specs,
        input: Input,
        _speaker: &mut Speaker,
    ) {
        let edge_wh = commands.ui_edge_wh();

        let lock_scene_outline_rect: unscaled::Rect = gfx::nine_slice::inner_rect(
            edge_wh,
            LOCK_SCENE_OUTER_RECT
        );

        let lock_scene_inner_rect: unscaled::Rect = gfx::nine_slice::inner_rect(
            edge_wh,
            lock_scene_outline_rect
        );

        let world_to_unscaled = |xy: world::XY, world_scroll: unscaled::XYD| -> unscaled::XY {
            unscaled::XY {
                x: unscaled::X(xy.x.0) + (lock_scene_inner_rect.x - unscaled::X(0)),
                y: unscaled::Y(xy.y.0) + (lock_scene_inner_rect.y - unscaled::Y(0)),
            } + world_scroll
        };

        //
        // Update
        //

        {
            // FIXME this should start an animation moving the card to the inventory, instead
            let lock = &mut self.locks.locks[self.locks.index];
            if lock.lights.is_empty() && matches!(lock.reward, Some(Reward::Item(_))) {
                let reward = lock.reward.take().expect("We just checked that the reward was an item!");
                self.apply_reward(reward);
            }
        }

        let inventory_cell_wh = edge_wh + specs.keycard_shuffle_cards.tile() + edge_wh;

        let inventory_inner_rect = nine_slice::inner_rect(edge_wh, INVENTORY_OUTER_RECT);

        let inventory_x_max = inventory_inner_rect.x + inventory_inner_rect.w;
        let inventory_y_max = inventory_inner_rect.y + inventory_inner_rect.h;

        if let Some(dir) = input.dir_pressed_this_frame() {
            match self.ui_section {
                UiSection::Map => {
                    match dir {
                        Dir::Up => {

                        }
                        Dir::Down => {

                        }
                        Dir::Left => {
                            if self.locks.index > 0 {
                                self.locks.index -= 1;
                            }
                        }
                        Dir::Right => {
                            if self.locks.index < self.locks.locks.len() - 1 {
                                self.locks.index += 1;
                            }
                        }
                    }


                    let lock = &self.locks.locks[self.locks.index];

                    let mut xy = world_to_unscaled(lock.xy, self.world_scroll);

                    let lock_tile = specs.keycard_shuffle_lights.tile();

                    while !lock_scene_inner_rect.contains(xy) {
                        // If the top of the card is above the clip rect, adjust scroll so that it is in view, at the top
                        if xy.y < lock_scene_inner_rect.y {
                            self.world_scroll.yd += lock_scene_inner_rect.h.into();
                        }

                        // If the bottom of the card is below the clip rect, adjust scroll so that it is in view, at the bottom
                        if xy.y + lock_tile.h > lock_scene_inner_rect.y + lock_scene_inner_rect.h {
                            self.world_scroll.yd -= lock_scene_inner_rect.h.into();
                        }

                        // If the left of the card is above the clip rect, adjust scroll so that it is in view, at the top
                        if xy.x < lock_scene_inner_rect.x {
                            self.world_scroll.xd += lock_scene_inner_rect.w.into();
                        }

                        // If the right of the card is below the clip rect, adjust scroll so that it is in view, at the bottom
                        if xy.x + lock_tile.w > lock_scene_inner_rect.x + lock_scene_inner_rect.w {
                            self.world_scroll.xd -= lock_scene_inner_rect.w.into();
                        }

                        xy = world_to_unscaled(lock.xy, self.world_scroll);
                    }
                },
                UiSection::Inventory => {
                    let inventory_cells_wide_count = usize::from(inventory_inner_rect.w / inventory_cell_wh.w.get());

                    match dir {
                        Dir::Up => {
                            if self.inventory.index >= inventory_cells_wide_count {
                                self.inventory.index -= inventory_cells_wide_count;
                            }
                        }
                        Dir::Down => {
                            if self.inventory.index + inventory_cells_wide_count < self.inventory.cells.len() {
                                self.inventory.index += inventory_cells_wide_count;
                            }
                        }
                        Dir::Left => {
                            if self.inventory.index > 0 {
                                self.inventory.index -= 1;
                            }
                        }
                        Dir::Right => {
                            if self.inventory.index < self.inventory.cells.len() - 1 {
                                self.inventory.index += 1;
                            }
                        }
                    }

                    // This is a version of the render loop, done here to find the
                    // amount to scroll.
                    // If we ever have a third place we do this, then combine them

                    let mut inventory_render_index = 0;

                    let mut at = inventory_inner_rect.xy();

                    while inventory_render_index < self.inventory.cells.len() {
                        // draw selectrum
                        if inventory_render_index == self.inventory.index {
                            let selected_at = unscaled::Rect {
                                x: at.x,
                                y: at.y,
                                w: inventory_cell_wh.w,
                                h: inventory_cell_wh.h,
                            } + self.inventory_scroll;

                            // If the top of the card is above the clip rect, adjust scroll so that it is in view, at the top
                            if selected_at.y < inventory_inner_rect.y {
                                self.inventory_scroll.yd += inventory_cell_wh.h.into();
                            }

                            // If the bottom of the card is below the clip rect, adjust scroll so that it is in view, at the bottom
                            if selected_at.y + selected_at.h > inventory_y_max {
                                self.inventory_scroll.yd -= inventory_cell_wh.h.into();
                            }

                            break
                        }

                        at.x += inventory_cell_wh.w;
                        if at.x + inventory_cell_wh.w >= inventory_x_max {
                            at.y += inventory_cell_wh.h;
                            at.x = inventory_inner_rect.x;
                        }
                        inventory_render_index += 1;
                    }
                },
                UiSection::AboveMap => { self.ui_section = UiSection::AboveInventory },
                UiSection::AboveInventory => { self.ui_section = UiSection::AboveMap },
            }
        } else if input.pressed_this_frame(Button::A) {
            match self.ui_section {
                UiSection::Map => {},
                UiSection::Inventory => {
                    if self.animations.lock.is_none() {
                        let mut reward = None;

                        if let (Some(_), Some(lock)) = (
                            self.inventory.cells.get(self.inventory.index),
                            self.locks.locks.get_mut(self.locks.index),
                        ) {
                            if lock.reward.is_some() {
                                if lock.is_open() {
                                    reward = Some(lock.reward.take().expect("We just checked it was Some"));
                                } else {
                                    self.animations.lock = Some(
                                        LockAnimation{
                                            state: <_>::default(),
                                            inventory_index: self.inventory.index,
                                            lock_index: self.locks.index,
                                        }
                                    );
                                }
                            }
                        }

                        if let Some(reward) = reward {
                            self.apply_reward(reward);
                        }
                    }
                },
                UiSection::AboveMap => { self.ui_section = UiSection::Map },
                UiSection::AboveInventory => { self.ui_section = UiSection::Inventory },
            }
        } else if input.pressed_this_frame(Button::B) {
            match self.ui_section {
                UiSection::AboveMap | UiSection::AboveInventory => {},
                UiSection::Map => { self.ui_section = UiSection::AboveMap },
                UiSection::Inventory => { self.ui_section = UiSection::AboveInventory },
            }
        }

        if input.pressed_this_frame(Button::START) {
            self.restart(specs);
        }

        self.tick();

        //
        // Render
        //

        use gfx::nine_slice;

        const PALETTE: [ARGB; 8] = [
            0xFF3352E1, // Blue
            0xFF30B06E, // Green
            0xFFDE4949, // Red
            0xFFFFB937, // Yellow
            0xFF533354, // Purple
            0xFF5A7D8B, // Cyan/Grey
            0xFFEEEEEE, // White
            0xFF222222, // Black
        ];

        const SELECTRUM_COLOUR: ARGB = PALETTE[3];
        const INDICATOR_COLOUR: ARGB = PALETTE[0];

        let letters_wh = specs.keycard_shuffle_letters.tile();
        let lights_wh = specs.keycard_shuffle_lights.tile();

        let slot_sprite_xy = specs.keycard_shuffle_slot.xy_from_tile_sprite(0u16);

        let card_y = unscaled::Y(0) + unscaled::H::new(command::HEIGHT_SIGNED / 6);

        let slot_xy = unscaled::XY {
            x: unscaled::X(0) + unscaled::W::new((command::WIDTH_SIGNED / 8) * 7),
            y: card_y - unscaled::H::new(4),
        };

        let slot_rect = specs.keycard_shuffle_slot.rect(slot_xy);

        let card_x_max = slot_rect.x + unscaled::W::new(2);

        macro_rules! draw_pip_at {
            ($xy: expr) => {
                draw_pip_at!(@commands: &mut commands, $xy);
            };
            (@commands: $commands: expr, $xy: expr) => {
                $commands.sspr_override(
                    specs.keycard_shuffle_lights.xy_from_tile_sprite(2u16),
                    specs.keycard_shuffle_lights.rect($xy),
                    PALETTE[6]
                );
            }
        }

        macro_rules! draw_card {
            ($xy: expr, $kind: expr) => ({
                draw_card!($xy, $kind, unscaled::X(command::WIDTH_SIGNED))
            });
            ($xy: expr, $kind: expr, $cutoff_x: expr) => ({
                draw_card!(@commands: &mut commands, $xy, $kind, $cutoff_x)
            });
            (@commands: $commands: expr, $xy: expr, $kind: expr) => ({
                draw_card!(@commands: $commands, $xy, $kind, unscaled::X(command::WIDTH_SIGNED))
            });
            (@commands: $commands: expr, $xy: expr, $kind: expr, $cutoff_x: expr) => ({
                let xy = $xy;
                let kind = $kind;
                let clip_rect = unscaled::Rect {
                    x: unscaled::X(0),
                    y: unscaled::Y(0),
                    w: $cutoff_x - unscaled::X(0),
                    h: unscaled::H::new(command::HEIGHT_SIGNED),
                };

                let colour_index: u16 = kind.colour.index();

                let mut cmds = $commands.clipped(clip_rect);

                // Card Back
                cmds.sspr_override(
                    specs.keycard_shuffle_cards.xy_from_tile_sprite(0u16),
                    specs.keycard_shuffle_cards.rect(xy),
                    PALETTE[usize::from(colour_index)],
                );

                // Card Stripe
                cmds.sspr(
                    specs.keycard_shuffle_cards.xy_from_tile_sprite(1u16),
                    specs.keycard_shuffle_cards.rect(xy),
                );

                // Colour Label
                let label_base_xy = xy + unscaled::H::new(20);

                cmds.sspr_override(
                    specs.keycard_shuffle_letters.xy_from_tile_sprite(colour_index),
                    specs.keycard_shuffle_letters.rect(label_base_xy),
                    PALETTE[6]
                );

                // Symbol (if any)
                match kind.symbol {
                    CardSymbol::None => {},
                    CardSymbol::OnePip => {
                        draw_pip_at!(
                            @commands: cmds,
                            label_base_xy
                                + letters_wh.w + lights_wh.w.halve()
                                + letters_wh.h.halve() - lights_wh.h.halve()
                        );
                    },
                    CardSymbol::TwoPips => {
                        let one_pip_xy = label_base_xy
                            + letters_wh.w + lights_wh.w.halve()
                            + letters_wh.h.halve() - lights_wh.h.halve();

                        draw_pip_at!(@commands: cmds, one_pip_xy);

                        draw_pip_at!(
                            @commands: cmds,
                            one_pip_xy + lights_wh.w + lights_wh.w.halve()
                        );
                    },
                };
            });
        }

        // Render lock scene

        commands.nine_slice_overridable(
            if self.ui_section == UiSection::AboveMap {
                nine_slice::Kind::CustomOutline(SELECTRUM_COLOUR)
            } else {
                nine_slice::Kind::CustomOutline(INDICATOR_COLOUR)
            },
            lock_scene_outline_rect
        );

        let mut clipped_commands = commands.clipped(lock_scene_inner_rect);

        // Render coloured splotches for background

        for splotch in &self.splotches {
            for w_xy in world::sprial_iter(splotch.radius, splotch.xy) {
                let xy = world_to_unscaled(w_xy, self.world_scroll);

                // draw splotch
                clipped_commands.sspr_override(
                    specs.keycard_shuffle_lights.xy_from_tile_sprite(2u16),
                    specs.keycard_shuffle_lights.rect(xy),
                    PALETTE[usize::from(splotch.colour.index())]
                );
            }
        }

        // Render flags

        for i in 0..self.locks.locks.len() {
            let lock = &self.locks.locks[i];

            let xy = world_to_unscaled(lock.xy, self.world_scroll);

            // Render flag
            let sprite_offset = match self.flag_state {
                FlagState::Zero(_) => 0,
                FlagState::One(_) | FlagState::Three(_) => 1,
                FlagState::Two(_) => 2,
            };

            clipped_commands.sspr_override(
                specs.keycard_shuffle_lights.xy_from_tile_sprite(4u16 + sprite_offset),
                specs.keycard_shuffle_lights.rect(xy),
                // Need to be visible on all backgrounds. Could have a complicated colour
                // swtiching scheme, but instead I'll wave the white flag.
                PALETTE[6]
            );

            // Render either selectrum or selection indicator
            if i == self.locks.index {
                clipped_commands.sspr_override(
                    specs.keycard_shuffle_lights.xy_from_tile_sprite(3u16),
                    specs.keycard_shuffle_lights.rect(xy),
                    if self.ui_section == UiSection::Map { SELECTRUM_COLOUR } else { INDICATOR_COLOUR }
                );
            }
        }

        // Render lock lights

        let lock = &self.locks.locks[self.locks.index];

        let light_base_x = slot_xy.x - unscaled::W::new(15);
        let light_y = slot_xy.y - unscaled::H::new(16);

        if lock.lights.is_empty() {
            match &lock.reward {
                Some(Reward::Item(card)) => {
                    draw_card!(
                        unscaled::XY {
                            x: CARD_X_MIN,
                            y: card_y,
                        },
                        card
                    );
                }
                Some(Reward::Win) => {
                    // draw nothing
                    // This case is not expected, but if it happens, no biggie
                }
                None => {
                    // draw nothing
                    // TODO? Render an empty card outline?
                }
            }
        } else {
            for (i, light) in lock.lights.iter().enumerate() {
                // Outer ring

                let xy = unscaled::XY {
                    x: light_base_x + unscaled::W::new(i as unscaled::Inner * 16),
                    y: light_y,
                };

                commands.sspr_override(
                    specs.keycard_shuffle_lights.xy_from_tile_sprite(0u16),
                    specs.keycard_shuffle_lights.rect(xy),
                    PALETTE[0]
                );

                match light.state {
                    LockLightState::Off => {},
                    LockLightState::Correct => {
                        commands.sspr_override(
                            specs.keycard_shuffle_lights.xy_from_tile_sprite(1u16),
                            specs.keycard_shuffle_lights.rect(xy),
                            PALETTE[1]
                        );
                    },
                    LockLightState::Wrong => {
                        commands.sspr_override(
                            specs.keycard_shuffle_lights.xy_from_tile_sprite(1u16),
                            specs.keycard_shuffle_lights.rect(xy),
                            PALETTE[2]
                        );
                    },
                }
            }

            // Render card slot back

            commands.sspr(slot_sprite_xy, slot_rect);

            let slot_overlay_x_shift = slot_rect.w.halve().inc();

            let mut slot_overlay_sprite_xy = slot_sprite_xy;
            slot_overlay_sprite_xy.x += slot_overlay_x_shift;

            let mut slot_overlay_rect = slot_rect;
            slot_overlay_rect.x += slot_overlay_x_shift;
            slot_overlay_rect.w -= slot_overlay_x_shift;

            match self.animations.lock {
                Some(LockAnimation{ ref state, inventory_index, .. }) => {
                    if let Some(item) = self.inventory.cells.get(inventory_index) {
                        let x = match state {
                            LockAnimationState::Insert(frame_count) => {
                                let fraction = (*frame_count) as f32 / MAX_INSERT_FRAME as f32;
                                unscaled::X(unscaled::lerp(CARD_X_MIN.0, fraction, card_x_max.0))
                            },
                            LockAnimationState::Inside(_) => card_x_max,
                            LockAnimationState::Remove(frame_count) => {
                                let fraction = (MAX_REMOVE_FRAME - (*frame_count)) as f32 / MAX_REMOVE_FRAME as f32;
                                unscaled::X(unscaled::lerp(CARD_X_MIN.0, fraction, card_x_max.0))
                            },
                        };

                        let xy = unscaled::XY {
                            x,
                            y: card_y,
                        };

                        draw_card!(xy, item, slot_overlay_rect.x);
                    }
                }
                None => {}
            }

            // Render card slot overlay
            commands.sspr(slot_overlay_sprite_xy, slot_overlay_rect);
        }

        // Render inventory

        commands.nine_slice_overridable(
            if self.ui_section == UiSection::AboveInventory {
                nine_slice::Kind::CustomOutline(SELECTRUM_COLOUR)
            } else {
                nine_slice::Kind::Inventory
            },
            INVENTORY_OUTER_RECT
        );

        let mut inventory_render_index = 0;

        let mut at = inventory_inner_rect.xy();

        let mut clipped_commands = commands.clipped(inventory_inner_rect);

        while inventory_render_index < self.inventory.cells.len() {
            // Render either selectrum or selection indicator
            if inventory_render_index == self.inventory.index {
                clipped_commands.nine_slice_overridable(
                    if self.ui_section == UiSection::Inventory {
                        nine_slice::Kind::CustomOutline(SELECTRUM_COLOUR)
                    } else {
                        nine_slice::Kind::CustomOutline(INDICATOR_COLOUR)
                    },
                    unscaled::Rect {
                        x: at.x,
                        y: at.y,
                        w: inventory_cell_wh.w,
                        h: inventory_cell_wh.h,
                    } + self.inventory_scroll,
                );
            }

            if let Some(card_kind) = self.inventory.cells.get(inventory_render_index) {
                draw_card!(@commands: &mut clipped_commands, at + edge_wh + self.inventory_scroll, card_kind);
            };

            at.x += inventory_cell_wh.w;
            if at.x + inventory_cell_wh.w >= inventory_x_max {
                at.y += inventory_cell_wh.h;
                at.x = inventory_inner_rect.x;
            }
            inventory_render_index += 1;
        }
    }
}