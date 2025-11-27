pub type SegmentWidth = usize;

/// 64k world segments ought to be enough for anybody!
pub type SegmentId = u16;

pub type TileSprite = u8;

pub const WALL_SPRITE: TileSprite = 0;
pub const FLOOR_SPRITE: TileSprite = 1;
pub const PLAYER_SPRITE: TileSprite = 2;
pub const NPC_SPRITE: TileSprite = 3;
pub const ITEM_SPRITE: TileSprite = 4;

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


// Fat-struct for entities! Fat-struct for entities!
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Entity {
    pub x: X,
    pub y: Y,
    pub offset_x: offset::X,
    pub offset_y: offset::Y,
    pub sprite: TileSprite,
    pub def_id: DefId,
    pub speeches_state: speeches::State,
}

impl Entity {
    pub fn new(
        x: X,
        y: Y,
        sprite: TileSprite,
        def_id: DefId,
    ) -> Self {
        Self {
            x,
            y,
            sprite,
            def_id,    
            ..<_>::default()
        }
    }

    pub fn xy(&self) -> XY {
        XY { x: self.x, y: self.y }
    }

    pub fn speeches_key(&self) -> speeches::Key {
        speeches::Key {
            def_id: self.def_id,
            state: self.speeches_state,
        }
    }
}

/// Returns a phrase like "a thing" or "an entity".
pub fn entity_article_phrase(entity: &Entity) -> &str {
    match entity.sprite {
        WALL_SPRITE => "a wall",
        FLOOR_SPRITE => "a floor",
        PLAYER_SPRITE => "a me(?!)",
        NPC_SPRITE => "a person",
        ITEM_SPRITE => "an item",
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

#[derive(Clone, Default)]
pub struct Tile {
    pub sprite: TileSprite,
}

pub fn is_passable(tile: &Tile) -> bool {
    tile.sprite == FLOOR_SPRITE
}

#[derive(Clone, Default)]
pub struct WorldSegment {
    pub id: SegmentId,
    pub width: SegmentWidth,
    // TODO? Nonempty Vec?
    // TODO Since usize is u32 on wasm, let's make a Vec32 type that makes that rsstriction clear, so we
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
            let def_id = DefId::try_from(self.first_speeches.len() + 1).map_err(|_| PushError::TooManyDefs)?;

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