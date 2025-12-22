pub type SegmentWidth = usize;

/// 64k world segments ought to be enough for anybody!
pub type SegmentId = u16;

pub type TileSprite = u8;

pub const WALL_SPRITE: TileSprite = 32;
pub const FLOOR_SPRITE: TileSprite = 33;
pub const PLAYER_SPRITE: TileSprite = 34;
pub const DOOR_ANIMATION_FRAME_1: TileSprite = 42;
pub const DOOR_ANIMATION_FRAME_2: TileSprite = DOOR_ANIMATION_FRAME_1 + 8;
pub const DOOR_ANIMATION_FRAME_3: TileSprite = DOOR_ANIMATION_FRAME_2 + 8;


/// An amount of screenshake to render with.
pub type ShakeAmount = u8;

/// Offsets from a tile, for visual purposes only.
pub mod offset {
    // TODO? Worth clamping these to the range [-1.0, 1.0], possibly removing subnormals too?
    //     Would be able to make them Eq in that case
    pub type X = f32;
    pub type Y = f32;
}

/// 64k entity definitions ought to be enough for anybody!
pub type DefId = u16;
// TODO? allow large enough deltas to represent going from DefId::MIN to DefId::MAX?
//     I suspect no one will
pub type DefIdDelta = i16;

pub type DefIdNextLargerSigned = i32;

/// Higher overrides lower.
pub type Precedence = u8;

#[derive(Clone, Default, Debug)]
pub struct SpeechSelection {
    pub speeches_state: speeches::State,
    pub precedence: Precedence,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum DesireState {
    #[default]
    Unsatisfiable,
    Unsatisfied,
    SatisfactionInSight,
    Satisfied,
}

#[derive(Clone, Default, Debug)]
pub struct Desire {
    pub state: DesireState,
    pub def_id: DefId,
}

impl Desire {
    pub fn new(def_id: DefId) -> Self {
        Self {
            def_id,
            state: DesireState::Unsatisfied,
        }
    }

    pub fn speech_selection(&self) -> SpeechSelection {
        use DesireState::*;
        match self.state {
            Unsatisfiable => SpeechSelection::default(),
            Unsatisfied => SpeechSelection { 
                speeches_state: 0,
                precedence: 0x02, // NPCs should mention what they want to the player.
            },
            SatisfactionInSight => SpeechSelection { 
                speeches_state: 1,
                precedence: 0x10, // NPCs should ask for what they want if they see it.
            },
            Satisfied => SpeechSelection { 
                speeches_state: 2,
                precedence: 0x01, // Saying thank you can be easily overriden by other concerns.
            },
        }
    }
}

pub type Desires = Vec<Desire>;

#[derive(Clone, Debug, Default)]
pub struct MiniEntityDef {
    pub id: DefId,
    pub flags: consts::EntityDefFlags,
    pub tile_sprite: TileSprite,
    pub on_collect: OnCollect,
    pub wants: Vec<DefId>,
}

#[derive(Clone, Debug, Default)]
pub struct EntityTransformable {
    pub id: DefId,
    pub flags: consts::EntityDefFlags,
    pub tile_sprite: TileSprite,    
    pub on_collect: OnCollect,
    pub wants: Desires,
}

impl From<&MiniEntityDef> for EntityTransformable {
    fn from(def: &MiniEntityDef) -> Self {
        Self {
            id: def.id,
            // This relies on the entity flags being a subset of the entity def flags.
            flags: def.flags,
            tile_sprite: def.tile_sprite,
            on_collect: def.on_collect.clone(),
            wants: def.wants.iter().map(|&id| Desire::new(id)).collect::<Vec<_>>(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub from: DefId, 
    pub to: DefId,
}

#[derive(Clone, Debug)]
pub enum CollectAction {
    Transform(Transform),
}

pub type OnCollect = Vec<CollectAction>;

pub type Inventory = Vec<Entity>;

pub mod consts {
    macro_rules! consts_def {
        (
            $all_name: ident : $type: ty;
            $($name: ident = $value: expr),+ $(,)?
        ) => {
    
    
            pub const $all_name: [(&str, $type); const {
                let mut count = 0;
    
                $(
                    // Use the repetition for something so we can take the count
                    const _: $type = $value;
                    count += 1;
                )+
    
                count
            }] = [
                $(
                    (stringify!($name), $value),
                )+
            ];
    
            $(
                pub const $name: $type = $value;
            )+
        };
    }
    
    pub type TileFlags = u32;
    
    consts_def!{
        ALL_TILE_FLAGS: TileFlags;
        // Can't be anything but a blocker
        WALL = 0,
        FLOOR = 1 << 0,
        PLAYER_START = 1 << 2,
        ITEM_START = 1 << 3,
        NPC_START = 1 << 4,
        DOOR_START = 1 << 5,
    }
    
    pub type EntityDefFlags = u8;
    
    consts_def!{
        ALL_ENTITY_FLAGS: EntityDefFlags;
        COLLECTABLE = super::COLLECTABLE,
        STEPPABLE = super::STEPPABLE,
        VICTORY = super::VICTORY,
        DOOR = super::DOOR,
        NOT_SPAWNED_AT_START = 1 << 4,
    }
    
    pub type EntityDefIdRefKind = u8;
    
    consts_def!{
        ALL_ENTITY_ID_REFERENCE_KINDS: EntityDefIdRefKind;
        RELATIVE = 1,
        ABSOLUTE = 2,
    }
    
    pub type CollectActionKind = u8;
    
    consts_def!{
        ALL_COLLECT_ACTION_KINDS: CollectActionKind;
        TRANSFORM = 1,
    }
}

pub type EntityFlags = u8;

pub const COLLECTABLE: EntityFlags = 1 << 0;
pub const STEPPABLE: EntityFlags = 1 << 1;
pub const VICTORY: EntityFlags = 1 << 2;
pub const DOOR: EntityFlags = 1 << 3;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub id: SegmentId,
    pub xy: XY,
}

impl Location {
    pub fn xy(&self) -> XY {
        self.xy
    }
}

// Fat-struct for entities! Fat-struct for entities!
#[derive(Clone, Default, Debug)]
pub struct Entity {
    pub x: X,
    pub y: Y,
    pub offset_x: offset::X,
    pub offset_y: offset::Y,
    pub transformable: EntityTransformable,
    pub inventory: Inventory,
    // TODO? Have a goal where it's a journey to discover that the path was inside you all along?
    pub door_target: Location,
}

impl Entity {
    pub fn new(
        x: X,
        y: Y,
        transformable: EntityTransformable,
    ) -> Self {
        Self {
            x,
            y,
            transformable,
            ..<_>::default()
        }
    }

    pub fn xy(&self) -> XY {
        XY { x: self.x, y: self.y }
    }

    pub fn def_id(&self) -> DefId {
        self.transformable.id
    }

    pub fn speeches_key(&self) -> speeches::Key {
        let mut current_speeches_state = <_>::default(); 
        let mut current_precedence = 0;

        for desire in &self.transformable.wants {
            let SpeechSelection{ speeches_state, precedence } = desire.speech_selection();

            if precedence > current_precedence {
                current_speeches_state = speeches_state;
                current_precedence = precedence;
            }
        }

        speeches::Key {
            def_id: self.def_id(),
            state: current_speeches_state,
        }
    }

    pub fn is_collectable(&self) -> bool {
        self.transformable.flags & COLLECTABLE == COLLECTABLE
    }

    pub fn is_steppable(&self) -> bool {
        self.transformable.flags & STEPPABLE == STEPPABLE
    }

    pub fn is_victory(&self) -> bool {
        self.transformable.flags & VICTORY == VICTORY
    }

    pub fn is_door(&self) -> bool {
        self.transformable.flags & DOOR == DOOR
    }
}

/// Returns a phrase like "a thing" or "an entity".
pub fn entity_article_phrase(entity: &Entity) -> &str {
    match entity.transformable.tile_sprite {
        WALL_SPRITE => "a wall",
        FLOOR_SPRITE => "a floor",
        PLAYER_SPRITE => "a me(?!)",
        _ => "a whatever-this-is",
    }
}

pub mod xy {
    pub type Inner = u16;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct X(Inner);

    /// Use this in case we want to clamp the coords, or something.
    pub const fn x(x: Inner) -> X {
        X(x)
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct W(Inner);

    /// Use this in case we want to clamp the coords, or something.
    pub const fn w(w: Inner) -> W {
        W(w)
    }

    impl core::ops::SubAssign<W> for W {
        fn sub_assign(&mut self, other: W) {
            self.0 = self.0.saturating_sub(other.0);
        }
    }

    impl core::ops::Sub<W> for W {
        type Output = Self;

        fn sub(mut self, other: W) -> Self::Output {
            self -= other;
            self
        }
    }

    pub const fn const_add_assign_w(x: &mut X, w: W) {
        x.0 = x.0.saturating_add(w.0);
    }

    impl core::ops::AddAssign<W> for X {
        fn add_assign(&mut self, w: W) {
            const_add_assign_w(self, w)
        }
    }

    pub const fn const_add_w(mut x: X, w: W) -> X {
        const_add_assign_w(&mut x, w);
        x
    }

    impl core::ops::Add<W> for X {
        type Output = Self;

        fn add(mut self, other: W) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::SubAssign<W> for X {
        fn sub_assign(&mut self, other: W) {
            self.0 = self.0.saturating_sub(other.0);
        }
    }

    impl core::ops::Sub<W> for X {
        type Output = Self;

        fn sub(mut self, other: W) -> Self::Output {
            self -= other;
            self
        }
    }

    impl core::ops::Sub<X> for X {
        type Output = W;

        fn sub(self, other: X) -> Self::Output {
            W(self.0.saturating_sub(other.0))
        }
    }


    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Y(Inner);

    /// Use this in case we want to clamp the coords, or something.
    pub const fn y(y: Inner) -> Y {
        Y(y)
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct H(Inner);

    /// Use this in case we want to clamp the coords, or something.
    pub const fn h(h: Inner) -> H {
        H(h)
    }

    impl core::ops::SubAssign<H> for H {
        fn sub_assign(&mut self, other: H) {
            self.0 = self.0.saturating_sub(other.0);
        }
    }

    impl core::ops::Sub<H> for H {
        type Output = Self;

        fn sub(mut self, other: H) -> Self::Output {
            self -= other;
            self
        }
    }

    pub const fn const_add_assign_h(y: &mut Y, h: H) {
        y.0 = y.0.saturating_add(h.0);
    }

    impl core::ops::AddAssign<H> for Y {
        fn add_assign(&mut self, h: H) {
            const_add_assign_h(self, h)
        }
    }

    pub const fn const_add_h(mut y: Y, h: H) -> Y {
        const_add_assign_h(&mut y, h);
        y
    }

    impl core::ops::Add<H> for Y {
        type Output = Self;

        fn add(mut self, other: H) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::SubAssign<H> for Y {
        fn sub_assign(&mut self, other: H) {
            self.0 = self.0.saturating_sub(other.0);
        }
    }

    impl core::ops::Sub<H> for Y {
        type Output = Self;

        fn sub(mut self, other: H) -> Self::Output {
            self -= other;
            self
        }
    }

    impl core::ops::Sub<Y> for Y {
        type Output = H;

        fn sub(self, other: Y) -> Self::Output {
            H(self.0.saturating_sub(other.0))
        }
    }

    macro_rules! shared_impl {
        ($($name: ident)+) => {
            $(
                impl $name {
                    pub const MIN: Self = Self(Inner::MIN);
                    pub const MAX: Self = Self(Inner::MAX);

                    pub const ZERO: Self = Self(0);
                    pub const ONE: Self = Self(1);

                    pub fn get(self) -> Inner {
                        self.0
                    }

                    pub fn dec(self) -> Self {
                        Self(self.0.saturating_sub(1))
                    }

                    pub fn inc(self) -> Self {
                        Self(self.0.saturating_add(1))
                    }

                    pub fn usize(self) -> usize {
                        self.0.into()
                    }

                    pub fn halve(self) -> Self {
                        Self(self.0 >> 1)
                    }
                }

                impl From<$name> for f32 {
                    fn from(value: $name) -> Self {
                        Self::from(value.get())
                    }
                }
            )+
        }
    }

    shared_impl!{
        X Y W H
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct XY {
        pub x: X,
        pub y: Y,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct WH {
        pub w: W,
        pub h: H,
    }

    pub const fn const_add_assign_wh(xy: &mut XY, wh: WH) {
        const_add_assign_w(&mut xy.x, wh.w);
        const_add_assign_h(&mut xy.y, wh.h);
    }

    impl core::ops::AddAssign<WH> for XY {
        fn add_assign(&mut self, wh: WH) {
            const_add_assign_wh(self, wh)
        }
    }

    pub const fn const_add_wh(mut xy: XY, wh: WH) -> XY {
        const_add_assign_wh(&mut xy, wh);
        xy
    }

    impl core::ops::Add<WH> for XY {
        type Output = Self;

        fn add(mut self, other: WH) -> Self::Output {
            self += other;
            self
        }
    }

    pub struct Rect {
        pub min_x: X,
        pub min_y: Y,
        pub max_x: X,
        pub max_y: Y,
    }

    pub fn eight_neighbors(x: X, y: Y) -> [(X, Y); 8] {
        let mut output: [(X, Y); 8] = <_>::default();

        for i in 0..8 {
            output[i] = match i {
                1 => (x + W::ONE, y - H::ONE),
                2 => (x, y - H::ONE),
                3 => (x - W::ONE, y - H::ONE),
                4 => (x - W::ONE, y),
                5 => (x - W::ONE, y + H::ONE),
                6 => (x, y + H::ONE),
                7 => (x + W::ONE, y + H::ONE),
                _ => (x + W::ONE, y),
            };
        }

        output
    }

}
pub use xy::{X, Y, W, H, Rect, XY};

#[derive(Clone, Debug, Default)]
pub struct Tile {
    pub sprite: TileSprite,
}

pub fn is_passable(tile: &Tile) -> bool {
    tile.sprite == FLOOR_SPRITE
}

#[derive(Clone, Debug, Default)]
pub struct WorldSegment {
    pub width: SegmentWidth,
    // TODO? Nonempty Vec?
    // TODO Since usize is u32 on wasm, let's make a Vec32 type that makes that restriction clear, so we
    // can't have like PC only worlds that break in weird ways online. Probably no one will ever need that
    // many tiles per segment. Plus, then xs conversions go away.
    pub tiles: Vec<Tile>,
}

pub type Index = usize;

pub enum XYToIError {
    XPastWidth
}

pub fn xy_to_i(segment: &WorldSegment, x: X, y: Y) -> Result<Index, XYToIError> {
    let x_usize = x.usize();
    if x_usize >= segment.width {
        return Err(XYToIError::XPastWidth);
    }

    Ok(y.usize() * segment.width + x_usize)
}

pub fn i_to_xy(segment_width: SegmentWidth, index: Index) -> XY {
    XY {
        x: xy::x((index % segment_width) as _),
        y: xy::y((index / segment_width) as _),
    }
}

pub mod config {
    use vec1::{Vec1};
    use crate::{
        consts::{EntityDefFlags, TileFlags}, DefId, OnCollect, SegmentWidth, Speech, TileSprite
    };

    /// A configuration WorldSegment that can be used to contruct game::WorldSegments later.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct WorldSegment {
        pub width: SegmentWidth,
        // TODO? Nonempty Vec?
        // TODO Since usize is u32 on wasm, let's make a Vec32 type that makes that rsstriction clear, so we
        // can't have like PC only worlds that break in weird ways online. Probably no one will ever need that
        // many tiles per segment. Plus, then xs conversions go away.
        pub tiles: Vec<TileFlags>,
    }

    #[derive(Clone)]
    pub struct Config {
        pub segments: Vec1<WorldSegment>,
        pub entities: Vec1<EntityDef>,
    }

    #[derive(Clone, Debug)]
    pub struct EntityDef {
        pub speeches: Vec<Vec<Speech>>,
        pub inventory_description: Vec<Vec<Speech>>,
        pub id: DefId,
        pub flags: EntityDefFlags,
        pub tile_sprite: TileSprite,
        pub wants: Vec<DefId>,
        pub on_collect: OnCollect,
    }
}
pub use config::{Config, EntityDef};

impl From<&EntityDef> for MiniEntityDef {
    fn from(def: &EntityDef) -> Self {
        Self {
            id: def.id,
            flags: def.flags,
            tile_sprite: def.tile_sprite,
            wants: def.wants.clone(),
            on_collect: def.on_collect.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Speech {
    pub text: String,
}

impl From<String> for Speech {
    fn from(text: String) -> Self {
        Self::from(text.as_str())
    }
}

impl From<&String> for Speech {
    fn from(text: &String) -> Self {
        Self::from(text.as_str())
    }
}

impl From<&str> for Speech {
    fn from(raw_text: &str) -> Self {
        Self {
            text: text::string::reflow(&raw_text.to_lowercase(), 54),
        }
    }
}

pub mod speeches {
    use crate::{DefId, Speech};
    use std::collections::BTreeMap;

    /// The state of the entity in so far as it relates to which speech
    /// should be used.
    pub type State = u8;

    #[derive(Clone, Copy, Debug)]
    pub struct Key {
        pub state: State,
        pub def_id: DefId,
    }

    type SparseState = std::num::NonZeroU8;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct SparseKey {
        state: SparseState,
        def_id: DefId,
    }
    
    impl TryFrom<Key> for SparseKey {
        type Error = <SparseState as TryFrom<State>>::Error;

        fn try_from(value: Key) -> Result<Self, Self::Error> {
            // Ideally this should be able to compile to almost nothing, since the representation is the same.
            // TODO? Check on that?
            let state = SparseState::try_from(value.state)?;
            Ok(SparseKey{
                state,
                def_id: value.def_id,
            })
        }
    }

    type SparseSpeeches = BTreeMap<SparseKey, Vec<Speech>>;

    #[derive(Clone, Debug, Default)]
    pub struct Speeches {
        // We expect that many entities will have a first speech of each category, so dense seems appropriate.
        // For now, it seems reasonable to assume we can force Def IDs to be dense, and start at 0.
        first_speeches: Vec<Vec<Speech>>,
        // We expect that many entities will only have a first speech though, so for the rest of them, 
        // sparse makes sense.
        // TODO? Use non-empty Vec here?
        sparse_speeches: SparseSpeeches,
    }
    
    #[derive(Clone, Debug)]
    pub enum PushError {
        TooManySpeechStates,
        TooManyDefs,
    }

    impl Speeches {
        /// In terms of entity defs to hold.
        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                first_speeches: Vec::with_capacity(capacity),
                sparse_speeches: SparseSpeeches::new(),
            }
        }

        #[must_use]
        pub fn push(&mut self, speeches: &mut [Vec<Speech>]) -> Result<(), PushError> {
            if speeches.is_empty() {
                self.first_speeches.push(Vec::new());
                return Ok(())
            }

            let state_len: SparseState = std::num::NonZeroUsize::try_from(speeches.len())
                .and_then(SparseState::try_from)
                .map_err(|_| PushError::TooManySpeechStates)?;
            let def_id = DefId::try_from(self.first_speeches.len()).map_err(|_| PushError::TooManyDefs)?;

            let first_speech = std::mem::replace(&mut speeches[0], Vec::new());

            self.first_speeches.push(first_speech);

            let mut state = SparseState::MIN;
            while state < state_len {
                self.sparse_speeches.insert(
                    SparseKey{
                        state,
                        def_id,
                    },
                    std::mem::replace(&mut speeches[state.get() as usize], Vec::new()),
                );

                state = state.saturating_add(1);
            }

            Ok(())
        }

        pub fn get(&self, key: Key) -> Option<&[Speech]> {
            match SparseKey::try_from(key) {
                Ok(s_key) => self.sparse_speeches.get(&s_key).map(|v| &**v),
                Err(_) => self.first_speeches.get(key.def_id as usize).map(|v| &**v),
            }
        }
    }
}
pub use speeches::{Speeches};