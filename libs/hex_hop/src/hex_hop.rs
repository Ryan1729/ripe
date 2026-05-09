use gfx::{Commands};
use gfx_sizes::ARGB;
use platform_types::{command, sprite, unscaled, Button, Dir, DirFlag, Input, Speaker};
use vec1::{Grid1, Grid1Spec, vec1, Vec1};
use xs::{Seed, Xs};

use std::collections::BTreeMap;
use std::num::TryFromIntError;

type Index = usize;

type TileSprite = u16;

pub type TilesWidthInner = xy::Inner;
pub type TilesWidth = std::num::NonZeroU16;

mod fixed {
    type Inner = i32;

    /// signed 16.16 fixed point
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Fixed(Inner);

    const SCALE: Inner = 16;

    pub const fn add_assign(a: &mut Fixed, b: Fixed) {
        a.0 += b.0
    }

    pub const fn add(mut a: Fixed, b: Fixed) -> Fixed {
        add_assign(&mut a, b);
        a
    }

    impl core::ops::AddAssign for Fixed {
        fn add_assign(&mut self, other: Fixed) {
            add_assign(self, other);
        }
    }

    impl core::ops::Add for Fixed {
        type Output = Self;

        fn add(mut self, other: Fixed) -> Self::Output {
            add(self, other)
        }
    }

    pub const fn sub_assign(a: &mut Fixed, b: Fixed) {
        a.0 -= b.0
    }

    pub const fn sub(mut a: Fixed, b: Fixed) -> Fixed {
        sub_assign(&mut a, b);
        a
    }

    impl core::ops::SubAssign for Fixed {
        fn sub_assign(&mut self, other: Fixed) {
            sub_assign(self, other);
        }
    }

    impl core::ops::Sub for Fixed {
        type Output = Self;

        fn sub(mut self, other: Fixed) -> Self::Output {
            sub(self, other)
        }
    }

    type WiderInner = i64;

    pub const fn mul_assign(a: &mut Fixed, b: Fixed) {
        a.0 = ((a.0 as WiderInner * b.0 as WiderInner) >> SCALE) as Inner
    }

    pub const fn mul(mut a: Fixed, b: Fixed) -> Fixed {
        mul_assign(&mut a, b);
        a
    }

    impl core::ops::MulAssign for Fixed {
        fn mul_assign(&mut self, other: Fixed) {
            mul_assign(self, other)
        }
    }

    impl core::ops::Mul for Fixed {
        type Output = Self;

        fn mul(mut self, other: Fixed) -> Self::Output {
            mul(self, other)
        }
    }

    pub const fn div_assign(a: &mut Fixed, b: Fixed) {
        a.0 = (((a.0 as WiderInner) << SCALE) / b.0 as WiderInner) as Inner
    }

    pub const fn div(mut a: Fixed, b: Fixed) -> Fixed {
        div_assign(&mut a, b);
        a
    }

    impl core::ops::DivAssign for Fixed {
        fn div_assign(&mut self, other: Fixed) {
            div_assign(self, other)
        }
    }

    impl core::ops::Div for Fixed {
        type Output = Self;

        fn div(mut self, other: Fixed) -> Self::Output {
            div(self, other)
        }
    }

    impl Fixed {
        pub const fn from_i16(n: i16) -> Fixed {
            Fixed((n as i32) << SCALE)
        }

        pub const fn round(self) -> i16 {
            if self.0 < 0 {
                (self.0 >> SCALE) as i16 + 1
            } else {
                (self.0 >> SCALE) as i16
            }
        }
    }

    pub fn from_i16(n: i16) -> Fixed {
        Fixed::from_i16(n)
    }

    impl From<i16> for Fixed {
        fn from(n: i16) -> Self {
            Self::from_i16(n)
        }
    }

    #[cfg(test)]
    mod from_i16_works {
        use super::*;

        #[test]
        fn on_these_basic_examples() {
            assert_eq!(from_i16(0), Fixed(0));

            let one = from_i16(1);

            assert_eq!(one, Fixed(0x1_0000));

            let minus_one = Fixed(0) - from_i16(1);

            assert_eq!(from_i16(-1), minus_one);

            assert_eq!(minus_one + one, Fixed(0));
            assert_eq!(minus_one + one + one, one);

            assert_eq!(minus_one * minus_one, one);

            assert_eq!(one * minus_one, minus_one);

            let two = from_i16(2);

            assert_eq!(two, one + one);

            assert_eq!(two, one * two);
            assert_eq!(two, two * one);

            let minus_two = from_i16(-2);

            assert_eq!(minus_two, minus_one + minus_one);

            assert_eq!(minus_two, minus_one * two);
            assert_eq!(minus_two, minus_two * one);

            let three = from_i16(3);

            assert_eq!(three, one + two);

            assert_eq!(three / two, Fixed(0b1_1000_0000_0000_0000));
        }
    }

    // f32 literal with more precision than represented here: 1.732050807568877293527446341505872367...
    pub const SQRT_3: Fixed = Fixed(0b0000_0000_0000_0001__1011_1011_0110_0111);

    #[cfg(test)]
    mod round_works {
        use super::*;

        #[test]
        fn on_these_found_examples_we_want_to_be_evenly_spaced() {
            {
                let n = 113511;
                let low = Fixed(-n);
                let middle = Fixed(0);
                let high = Fixed(n);
                
                let low_rounded = low.round();
                let middle_rounded = middle.round();
                let high_rounded = high.round();
    
                assert_eq!(middle_rounded - low_rounded, high_rounded - middle_rounded, "n = {n}");
            }
            {
                let low = Fixed(5960339);
                let middle = Fixed(7208960);
                let high = Fixed(8457581);

                assert_eq!(middle.0 - low.0, high.0 - middle.0);
                
                let low_rounded = low.round();
                let middle_rounded = middle.round();
                let high_rounded = high.round();
    
                assert_eq!(middle_rounded - low_rounded, high_rounded - middle_rounded, "({low_rounded:?}, {high_rounded:?})");
            }
        }

        #[test]
        fn by_making_every_negatable_number_below_two_round_to_the_same_value_both_ways() {
            for n in Fixed(1).0..Fixed(2).0 {
                let low = Fixed(-n);
                let middle = Fixed(0);
                let high = Fixed(n);
                
                let low_rounded = low.round();
                let middle_rounded = middle.round();
                let high_rounded = high.round();

                assert_eq!(middle_rounded - low_rounded, high_rounded - middle_rounded, "n = {n} ({low_rounded:?}, {high_rounded:?})");
            }
        }

        #[test]
        fn by_making_every_negatable_number_in_this_range_round_to_the_same_distance_apart_when_this_number_is_added() {
            const FOUND_OFFSET: Inner = 7208960 - 5960339;
            // This is the first one in the range that current works: 5963776
            // Maybe that's interesting?
            println!("{:#b} {:#b} {:#b}", 5960339, 5960339 + FOUND_OFFSET, 5960339 + FOUND_OFFSET + FOUND_OFFSET);
            let low = Fixed(5960339);
            let middle = Fixed(low.0 + FOUND_OFFSET);
            let high = Fixed(middle.0 + FOUND_OFFSET);
            
            let low_rounded = low.round();
            let middle_rounded = middle.round();
            let high_rounded = high.round();
            dbg!(low, middle, high, low_rounded, middle_rounded, high_rounded, );
            for n in Fixed(5960339).0..Fixed(5960339 + (2 << 16)).0 {
                let low = Fixed(n);
                let middle = Fixed(low.0 + FOUND_OFFSET);
                let high = Fixed(middle.0 + FOUND_OFFSET);
                
                let low_rounded = low.round();
                let middle_rounded = middle.round();
                let high_rounded = high.round();

                assert_eq!(middle_rounded - low_rounded, high_rounded - middle_rounded, "n = {n} ({low_rounded:?}, {high_rounded:?})");
            }
        }
    }
}
use fixed::Fixed;

/// Hexagonal coordinates.
/// We follow the q, r, and s naming convention used in https://www.redblobgames.com/grids/hexagons/
mod qrs {
    use crate::fixed::{self, Fixed};

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Dir {
        #[default]
        DecRIncS,
        DecRIncQ,
        DecSIncQ,
        DecSIncR,
        DecQIncR,
        DecQIncS,
    }

    impl Dir {
        pub const ALL: [Dir; 6] = [
            Dir::DecRIncS,
            Dir::DecRIncQ,
            Dir::DecSIncQ,
            Dir::DecSIncR,
            Dir::DecQIncR,
            Dir::DecQIncS,
        ];

        fn basis(self) -> QRSD {
            match self {
                Dir::DecRIncS => QRSD { qd: QD(0),  rd: RD(-1) },
                Dir::DecRIncQ => QRSD { qd: QD(1),  rd: RD(-1) },
                Dir::DecSIncQ => QRSD { qd: QD(1),  rd: RD(0)  },
                Dir::DecSIncR => QRSD { qd: QD(0),  rd: RD(1)  },
                Dir::DecQIncR => QRSD { qd: QD(-1), rd: RD(1)  },
                Dir::DecQIncS => QRSD { qd: QD(-1), rd: RD(0)  },
            }
        }
    }

    pub type Inner = i16;

    pub type Distance = u8;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Q(pub Inner);

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct R(pub Inner);

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct S(pub Inner);

    /// We can avoid storing `S` by computing it as needed based only on q and r.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct QRS {
        // We put R first so the default sorting layers the hexes from back to front.
        pub r: R,
        pub q: Q,
    }

    type NeighborError = ();

    impl QRS {
        fn neighbor(self, dir: Dir) -> Self {
            self + dir.basis()
        }
    }

    const X_Q_FACTOR: Fixed = fixed::div(Fixed::from_i16(3), Fixed::from_i16(2));
    const X_R_FACTOR: Fixed = Fixed::from_i16(0);

    const Y_Q_FACTOR: Fixed = fixed::div(fixed::SQRT_3, Fixed::from_i16(2));
    const Y_R_FACTOR: Fixed = fixed::SQRT_3;
    
    impl QRS {
        /// Converts to x and y on a conceptual infinite hex-grid. Will likely
        /// need further processing for any real use-case.
        #[allow(unused)]
        pub fn to_unit_grid(self) -> (Fixed, Fixed) {
            let q = Fixed::from_i16(self.q.0);
            let r = Fixed::from_i16(self.r.0);

            let x = X_Q_FACTOR * q + X_R_FACTOR * r;
            let y = Y_Q_FACTOR * q + Y_R_FACTOR * r;
            (x, y)
        }
    }

    /// A delta in Q space, as opposed to a Q, which is a point.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct QD(pub Inner);

    /// A delta in R space, as opposed to an R, which is a point.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RD(pub Inner);

    /// A delta in S space, as opposed to an S, which is a point.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SD(pub Inner);

    macro_rules! shared_d_def {
        ($($name: ident)+) => {
            $(
                impl $name {
                    fn scale(self, radius: Distance) -> Self {
                        Self(self.0.saturating_mul(radius.into()))
                    }
                }
            )+
        }
    }

    shared_d_def!{
        QD
        RD
        SD
    }

    impl core::ops::AddAssign<QD> for Q {
        fn add_assign(&mut self, other: QD) {
            self.0 += other.0;
        }
    }

    impl core::ops::Add<QD> for Q {
        type Output = Self;

        fn add(mut self, other: QD) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::AddAssign<RD> for R {
        fn add_assign(&mut self, other: RD) {
            self.0 += other.0;
        }
    }

    impl core::ops::Add<RD> for R {
        type Output = Self;

        fn add(mut self, other: RD) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::AddAssign<SD> for S {
        fn add_assign(&mut self, other: SD) {
            self.0 += other.0;
        }
    }

    impl core::ops::Add<SD> for S {
        type Output = Self;

        fn add(mut self, other: SD) -> Self::Output {
            self += other;
            self
        }
    }

    /// A delta in QRS space, as opposed to a QRS, which is a point.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct QRSD {
        qd: QD,
        rd: RD,
    }

    impl core::ops::AddAssign<QRSD> for QRS {
        fn add_assign(&mut self, other: QRSD) {
            self.q += other.qd;
            self.r += other.rd;
        }
    }

    impl core::ops::Add<QRSD> for QRS {
        type Output = Self;

        fn add(mut self, other: QRSD) -> Self::Output {
            self += other;
            self
        }
    }

    impl QRSD {
        fn scale(self, radius: Distance) -> Self {
            Self {
                qd: self.qd.scale(radius),
                rd: self.rd.scale(radius),
            }
        }
    }

    pub fn spiral(radius: Distance, center: QRS) -> impl Iterator<Item = QRS> {
        // See https://www.redblobgames.com/grids/hexagons/#rings for capacity formula
        let mut output = Vec::with_capacity(1 + 3 * radius as usize * (radius as usize + 1));

        output.push(center);

        for ring_i in 1..=radius {
            let mut hex = center + Dir::ALL[4].basis().scale(ring_i);

            for dir in Dir::ALL {
                for _ in 0..ring_i {
                    output.push(hex);
                    hex = hex.neighbor(dir);
                }
            }
        }

        output.into_iter()
    }

    #[cfg(test)]
    mod spiral_works {
        use super::*;

        #[test]
        fn on_the_basic_1_case() {
            let actual: Vec<_> = spiral(1, <_>::default()).collect();

            assert_eq!(actual.len(), 7);

            macro_rules! a {
                ($expected_element: expr) => {
                    assert!(
                        actual.contains(&$expected_element)
                    );
                }
            }

            a!(QRS { q: Q(0), r: R(0), });

            a!(QRS { q: Q(1), r: R(0), });
            a!(QRS { q: Q(1), r: R(-1), });
            a!(QRS { q: Q(0), r: R(-1), });
            a!(QRS { q: Q(-1), r: R(0), });
            a!(QRS { q: Q(-1), r: R(1), });
            a!(QRS { q: Q(0), r: R(1), });
        }

        #[test]
        fn on_the_basic_2_case() {
            let actual: Vec<_> = spiral(2, <_>::default()).collect();

            assert_eq!(actual.len(), 19);

            macro_rules! a {
                ($expected_element: expr) => {
                    assert!(
                        actual.contains(&$expected_element)
                    );
                }
            }

            a!(QRS { q: Q(0), r: R(0), });

            a!(QRS { q: Q(1), r: R(0), });
            a!(QRS { q: Q(1), r: R(-1), });
            a!(QRS { q: Q(0), r: R(-1), });
            a!(QRS { q: Q(-1), r: R(0), });
            a!(QRS { q: Q(-1), r: R(1), });
            a!(QRS { q: Q(0), r: R(1), });

            a!(QRS { q: Q(0), r: R(-2), });
            a!(QRS { q: Q(1), r: R(-2), });
            a!(QRS { q: Q(2), r: R(-2), });
            a!(QRS { q: Q(2), r: R(-1), });
            a!(QRS { q: Q(2), r: R(0), });
            a!(QRS { q: Q(0), r: R(-1), });
            a!(QRS { q: Q(1), r: R(1), });
            a!(QRS { q: Q(0), r: R(2), });
            a!(QRS { q: Q(-1), r: R(2), });
            a!(QRS { q: Q(-2), r: R(2), });
            a!(QRS { q: Q(-2), r: R(1), });
            a!(QRS { q: Q(-2), r: R(0), });
            a!(QRS { q: Q(-1), r: R(-1), });
        }
    }
}
use qrs::{QRS, Q, R};

pub mod xy {
    pub type Inner = u16;
    /// A signed type large enough to hold the difference between two Inner
    /// values.
    pub type Diff = i32;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct X(pub Inner);

    pub fn x(inner: Inner) -> X { X(inner) }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Y(pub Inner);

    pub fn y(inner: Inner) -> Y { Y(inner) }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct W(pub Inner);

    pub fn w(inner: Inner) -> W { W(inner) }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct H(pub Inner);

    pub fn h(inner: Inner) -> H { H(inner) }

    macro_rules! def {
        ($($name: path),+) => {
            $(
                impl $name {
                    pub const ONE: Self = Self(1);

                    pub fn dec(&self) -> Self {
                        Self(self.0.saturating_sub(1))
                    }

                    pub fn inc(&self) -> Self {
                        Self(self.0.saturating_add(1))
                    }

                    pub fn double(&self) -> Self {
                        Self(self.0.saturating_mul(2))
                    }

                    pub fn u32(self) -> u32 {
                        u32::from(self.0)
                    }

                    pub fn usize(self) -> usize {
                        usize::from(self.0)
                    }

                    pub fn diff(self) -> Diff {
                        Diff::from(self.0)
                    }
                }

                impl From<$name> for Diff {
                    fn from(value: $name) -> Diff {
                        Diff::from(value.0)
                    }
                }
            )+
        }
    }

    def!{ X, Y, W, H }

    macro_rules! unsigned_paired_impls {
        ($($a_name: ident, $b_name: ident)+) => {$(
            impl core::ops::AddAssign<$b_name> for $a_name {
                fn add_assign(&mut self, other: $b_name) {
                    self.0 += other.0;
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
                    self.0 -= other.0;
                }
            }

            impl core::ops::Sub<$b_name> for $a_name {
                type Output = Self;

                fn sub(mut self, other: $b_name) -> Self::Output {
                    self -= other;
                    self
                }
            }

            impl $a_name {
                pub fn checked_add(&self, b: $b_name) -> Option<Self> {
                    Some(Self(self.0.checked_add(b.0)?))
                }

                pub fn checked_sub(&self, b: $b_name) -> Option<Self> {
                    Some(Self(self.0.checked_sub(b.0)?))
                }
            }
        )+}
    }

    unsigned_paired_impls!{
        X, W
        Y, H
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct XY {
        pub x: X,
        pub y: Y,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct WH {
        pub w: W,
        pub h: H,
    }

    impl core::ops::AddAssign<WH> for XY {
        fn add_assign(&mut self, other: WH) {
            self.x += other.w;
            self.y += other.h;
        }
    }

    impl core::ops::Add<WH> for XY {
        type Output = Self;

        fn add(mut self, other: WH) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::AddAssign<W> for XY {
        fn add_assign(&mut self, other: W) {
            self.x += other;
        }
    }

    impl core::ops::Add<W> for XY {
        type Output = Self;

        fn add(mut self, other: W) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::AddAssign<H> for XY {
        fn add_assign(&mut self, other: H) {
            self.y += other;
        }
    }

    impl core::ops::Add<H> for XY {
        type Output = Self;

        fn add(mut self, other: H) -> Self::Output {
            self += other;
            self
        }
    }
}
#[allow(unused_imports)]
use xy::{X, Y, XY, W, H, WH};

pub fn i_to_xy(width: impl Into<TilesWidth>, index: Index) -> XY {
    let width = width.into();
    XY {
        x: xy::x((index % usize::from(width.get())) as _),
        y: xy::y((index / usize::from(width.get())) as _),
    }
}

#[derive(Debug)]
pub enum XYToIError {
    XPastWidth
}

pub fn xy_to_i(width: impl Into<TilesWidth>, xy: XY) -> Result<Index, XYToIError> {
    let width = width.into();
    let width_usize = usize::from(width.get());

    let x_usize = xy.x.usize();
    if x_usize >= width_usize {
        return Err(XYToIError::XPastWidth);
    }

    Ok(xy.y.usize() * width_usize + x_usize)
}

type HexHeight = u8;

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub height: HexHeight,
    pub colour: ARGB
}

pub type Key = QRS;

pub type Tiles = BTreeMap<Key, Tile>;

#[derive(Clone, Debug, Default)]
pub struct Entity {
}

mod mobs {
    use super::*;

    #[derive(Clone, Debug, Default)]
    pub struct Mobs {
        entities: BTreeMap<Key, Entity>,
    }

    impl Mobs {
        pub fn get(&self, key: Key) -> Option<&Entity> {
            self.entities.get(&key)
        }

        pub fn get_mut(&mut self, key: Key) -> Option<&mut Entity> {
            self.entities.get_mut(&key)
        }

        pub fn remove(&mut self, key: Key) -> Option<Entity> {
            self.entities.remove(&key)
        }

        pub fn insert(&mut self, key: Key, entity: Entity) {
            self.entities.insert(
                key,
                entity
            );
        }

        pub fn entities_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
            self.entities.values_mut()
        }

        pub fn all(&self) -> impl Iterator<Item = (&Key, &Entity)> {
            self.entities.iter()
        }

        pub fn keys(&self) -> impl Iterator<Item = &Key> {
            self.entities.keys()
        }
    }
}
use mobs::Mobs;

#[derive(Clone, Debug)]
pub struct State {
    pub seed: Seed, // For restarting
    pub rng: Xs,
    pub tiles: Tiles,
    pub mobs: Mobs,
}

impl State {
    pub fn new(rng: &mut Xs, hex_pieces_spec: &sprite::Spec::<sprite::HexPieces>) -> Self {
        let seed = xs::new_seed(rng);

        Self::init(seed, hex_pieces_spec)
    }

    fn init(seed: Seed, hex_pieces_spec: &sprite::Spec::<sprite::HexPieces>) -> Self {
        let mut rng_ = xs::from_seed(seed);
        let rng = &mut rng_;

        let mut tiles = Tiles::default();

        // TODO? Is this actually going to be useful elsewhere? And will it stick around?
        macro_rules! qr {
            ($q_inner: literal $r_inner: literal) => {
                QRS {
                    q: Q($q_inner),
                    r: R($r_inner),
                }
            }
        }
        
        // TODO Generate the layout instead.
        #[cfg(false)]
        let coords = [
            qr!(0 0),
            qr!(1 0),
            qr!(1 -1),

            qr!(-1 0),
            qr!(-1 1),
            qr!(0 1),
            qr!(1 -2),
            qr!(2 -2),
            qr!(2 -1),

            qr!(1 2),
            qr!(1 1),
            qr!(0 2),
            qr!(-1 2),
            qr!(-1 3),
            qr!(0 3),

            qr!(-2 -2),
            qr!(-2 -1),
            qr!(-2 0),
            qr!(-2 1),
            qr!(-2 2),
            qr!(-1 -1),

            qr!(2 0),
            qr!(2 1),
            qr!(2 2),

            qr!(3 -3),
            qr!(3 -2),
            qr!(3 -1),

            qr!(-3 0),
            qr!(-3 1),
            qr!(-3 2),

            qr!(0 -4),
            qr!(0 -3),
            qr!(0 -2),
            qr!(0 -1),// Above visible problem
        ];

        #[cfg(true)]
        let coords = [
            qr!(0 -1),
            qr!(0 0),
            qr!(0 1),
        ];

        let heights = [0];//[0, 0, 0, 0, 0, 0, 0, 20, 20, 20];
        let palette = [
            0xFF3352E1,
            0xFF30B06E,
            0xFFDE4949,
            0xFFFFB937,
            0xFF533354,
            0xFF5A7D8B,
            0xFFEEEEEE,
            0xFF222222,
        ];

        for i in 0..coords.len() {
            tiles.insert(
                coords[i],
                Tile {
                    height: heights[i % heights.len()],
                    colour: palette[i % palette.len()],
                },
            );
        }

        let mut mobs = Mobs::default();

        Self {
            seed,
            rng: rng_,
            tiles,
            mobs,
        }
    }

    fn restart(&mut self, hex_pieces_spec: &sprite::Spec::<sprite::HexPieces>) {
        *self = Self::init(self.seed, hex_pieces_spec);
    }

    pub fn is_complete(&self) -> bool {
        false
    }

    fn tick(&mut self) {

    }

    pub fn update_and_render(
        &mut self,
        commands: &mut Commands,
        hex_pieces_spec: &sprite::Spec::<sprite::HexPieces>,
        input: Input,
        _speaker: &mut Speaker,
    ) {
        //
        //
        // Update Section
        //
        //

        self.tick();

        //
        //
        // Render Section
        //
        //

        let tile = hex_pieces_spec.tile();
        let tile_w = tile.w;
        let tile_h = tile.h;

        //
        // Draw Hexes
        //

        // TODO move these to a `colour` crate, find a good colour space or whatever to
        // make these not fuck up the hue. Do not move the ARGB type alias there, but
        // making a copy of it is fine.
        let darken = |colour: ARGB| {
            let alpha = colour & 0xFF00_0000;

            let mut r = colour & 0xFF_0000;
            r = r.saturating_div(2);
            r &= 0xFF_0000;
            let mut g = colour & 0xFF00;
            g = g.saturating_div(2);
            g &= 0xFF00;
            let mut b = colour & 0xFF;
            b = b.saturating_div(2);

            alpha | r | g | b
        };

        let brighten = |colour: ARGB| {
            let alpha = colour & 0xFF00_0000;

            let mut r = colour & 0xFF_0000;
            r = r.saturating_mul(2);
            r &= 0xFF_0000;
            let mut g = colour & 0xFF00;
            g = g.saturating_mul(2);
            g &= 0xFF00;

            let mut b = colour & 0xFF;
            b = b.saturating_div(2);
            b &= 0xFF;

            alpha | r | g | b
        };

        const HEX_Y_SCALE: Fixed = Fixed::from_i16(12);

        const HEX_X_SCALE: Fixed = Fixed::from_i16(12);

        const HEX_X_OFFSET: Fixed = Fixed::from_i16(5);
        const HEX_Y_OFFSET: Fixed = Fixed::from_i16(10);

        const X_Q_FACTOR: Fixed = fixed::div(Fixed::from_i16(3), Fixed::from_i16(2));
        const X_R_FACTOR: Fixed = Fixed::from_i16(0);
    
        const Y_Q_FACTOR: Fixed = fixed::div(fixed::SQRT_3, Fixed::from_i16(2));
        const Y_R_FACTOR: Fixed = fixed::SQRT_3;

        let mut draw_hex = |qrs: QRS, Tile { height, colour: base_colour }| {
            // Getting weird spacing issues around 0.
            // TODO Okay, the issue seems to be just inherent in the fact that we are rounding
            // when we are deciding which pixel to render things in, and starting from 
            // the technically correct sqrt(3) based spacing. So we can instead just place 
            // things an integer distance apart, perhaps by rounding sqrt(3) at the right spot
            
            let q = Fixed::from_i16(qrs.q.0);
            let r = Fixed::from_i16(qrs.r.0);

            let x = X_Q_FACTOR * q + X_R_FACTOR * r;
            let y = Y_Q_FACTOR * q + Y_R_FACTOR * r;
            
            //dbg!((
                //((x + HEX_X_OFFSET) * HEX_X_SCALE).round(),
                //((y + HEX_Y_OFFSET) * HEX_Y_SCALE).round()
            //));
            let at: unscaled::XY = unscaled::XY {
                x: unscaled::X(((x + HEX_X_OFFSET) * HEX_X_SCALE).round().try_into().unwrap_or(0)),
                y: unscaled::Y(((y + HEX_Y_OFFSET) * HEX_Y_SCALE).round().try_into().unwrap_or(0)),
            };
            //dbg!(at);
            //dbg!(y, ((y + HEX_Y_OFFSET) * HEX_Y_SCALE), at.y);
            //dbg!(((y + HEX_Y_OFFSET) * HEX_Y_SCALE), ((y + HEX_Y_OFFSET) * HEX_Y_SCALE).round());

            let outline_colour: ARGB = 0xFF00_0000;
            // TODO? cache this across frames? It is a few cbrts.
            let colour::DarkMiddleBright{ dark, middle, bright }
                = colour::DarkMiddleBright::from(base_colour);

            let top_face_colour: ARGB = middle;
            let top_lower_edge_colour: ARGB = bright;
            let left_face_colour: ARGB = bright;
            let center_face_colour: ARGB = middle;
            let right_face_colour: ARGB = dark;

            const TOP_LINE: TileSprite = 0;
            const LEFT_RIGHT_EDGES: TileSprite = 3;
            const TOP_FACE: TileSprite = 6;
            const BOTTOM_FULL_LINE: TileSprite = 9;
            const BOTTOM_LEFT_LINE: TileSprite = 12;
            const BOTTOM_RIGHT_LINE: TileSprite = 15;
            const BOTTOM_CENTER_LINE: TileSprite = 18;

            let mut xy = at;
            xy.y -= unscaled::H(height.into());

            for _ in 0..2 {
                commands.sspr_override(
                    hex_pieces_spec.xy_from_tile_sprite(TOP_LINE),
                    command::Rect::from_unscaled(hex_pieces_spec.rect(xy)),
                    outline_colour,
                );

                xy += unscaled::H(1);
            }

            commands.sspr_override(
                hex_pieces_spec.xy_from_tile_sprite(TOP_FACE),
                command::Rect::from_unscaled(hex_pieces_spec.rect(xy)),
                top_face_colour,
            );

            xy += (tile_h / 2);
            xy -= unscaled::H(1);

            macro_rules! left_right_edges {
                () => {
                    commands.sspr_override(
                        hex_pieces_spec.xy_from_tile_sprite(LEFT_RIGHT_EDGES),
                        command::Rect::from_unscaled(hex_pieces_spec.rect(xy)),
                        outline_colour,
                    );
                }
            }

            for _ in 0..2 {
                left_right_edges!();

                xy += unscaled::H(1);
            }

            commands.sspr_override(
                hex_pieces_spec.xy_from_tile_sprite(BOTTOM_FULL_LINE),
                command::Rect::from_unscaled(hex_pieces_spec.rect(xy)),
                top_lower_edge_colour,
            );
            left_right_edges!();

            xy += unscaled::H(1);

            for _ in 0..height {
                commands.sspr_override(
                    hex_pieces_spec.xy_from_tile_sprite(BOTTOM_LEFT_LINE),
                    command::Rect::from_unscaled(hex_pieces_spec.rect(xy)),
                    left_face_colour,
                );
                commands.sspr_override(
                    hex_pieces_spec.xy_from_tile_sprite(BOTTOM_CENTER_LINE),
                    command::Rect::from_unscaled(hex_pieces_spec.rect(xy)),
                    center_face_colour,
                );
                commands.sspr_override(
                    hex_pieces_spec.xy_from_tile_sprite(BOTTOM_RIGHT_LINE),
                    command::Rect::from_unscaled(hex_pieces_spec.rect(xy)),
                    right_face_colour,
                );
                left_right_edges!();

                xy += unscaled::H(1);
            }

            left_right_edges!();
            for _ in 0..2 {
                commands.sspr_override(
                    hex_pieces_spec.xy_from_tile_sprite(BOTTOM_FULL_LINE),
                    command::Rect::from_unscaled(hex_pieces_spec.rect(xy)),
                    outline_colour,
                );
                xy += unscaled::H(1);
            }
        };

        for (&qrs, &tile) in self.tiles.iter() {
            draw_hex(
                qrs,
                tile,
            );
        }

        //
        // Draw Mobs
        //

        for (&key, mob) in self.mobs.all() {
            // TODO
        }

        //
        // Draw player
        //

        // TODO
    }
}