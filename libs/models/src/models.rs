pub type SegmentWidth = usize;

/// 64k world segments ought to be enough for anybody!
pub type SegmentId = u16;

pub type TileSprite = u8;

pub const WALL_SPRITE: TileSprite = 0;
pub const FLOOR_SPRITE: TileSprite = 1;
pub const PLAYER_SPRITE: TileSprite = 2;
pub const NPC_SPRITE: TileSprite = 3;
pub const ITEM_SPRITE: TileSprite = 4;

// Fat-struct for entities! Fat-struct for entities!
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Entity {
    pub x: X,
    pub y: Y,
    pub sprite: TileSprite,
}

impl Entity {
    pub fn xy(&self) -> XY {
        XY { x: self.x, y: self.y }
    }
}

pub mod xy {
    use super::*;

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