#![deny(unconditional_recursion)]
///! Hexagonal coordinates.
///! We follow the q, r, and s naming convention used in https://www.redblobgames.com/grids/hexagons/

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

type RotationAmount = i8;

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

    pub fn index(self) -> u8 {
        let mut index = 0;
        for i in 0..Self::ALL.len() as u8 {
            if Self::ALL[i as usize] == self {
                index = i;
            }
        }
        index
    }

    pub fn clockwise(self, mut by: RotationAmount) -> Self {
        let mut index = self.index() as usize;

        if by < 0 {
            // Force index to be high enogh we won't hit 0 while subratcing,
            // without chainging the value modulo Self::ALL.len()
            index += Self::ALL.len() * (RotationAmount::MAX as usize);

            // There's probably a clever way to do this, while also accounting
            // for the -128 case, but this is simple, correct in all cases, and
            // in practice more than likely fast enough, since the number of 
            // loops is bounded by a small constant. Maybe the compiler is even
            // clever enough to figure out the clever way for us.
            while by < 0 {
                by += 1;
                index -= 1;
            }
        } else {
            index += usize::from(by as u8);
        }

        index %= Self::ALL.len();

        Self::ALL[index]
    }
}

pub type Inner = i16;

pub type Distance = u8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Q(pub Inner);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct R(pub Inner);

#[allow(unused)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct S(pub Inner);

/// We can avoid storing `S` by computing it as needed based only on q and r.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QRS {
    // We put R first so the default sorting layers the hexes from back to front.
    pub r: R,
    pub q: Q,
}

#[macro_export]
macro_rules! qr_ {
    ($q_inner: literal $(,)? $r_inner: literal) => {
        QRS {
            q: Q($q_inner),
            r: R($r_inner),
        }
    }
}
pub use qr_ as qr;


impl PartialEq<&QRS> for QRS {
    fn eq(&self, other: &&QRS) -> bool {
        *self == **other
    }
}

impl PartialEq<QRS> for &QRS {
    fn eq(&self, other: &QRS) -> bool {
        *self == other
    }
}

impl QRS {
    pub fn neighbor(self, dir: Dir) -> Self {
        self + dir.basis()
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
    (
        [$self: ident $other: ident]
        $($d_name: ident $base_name: ident $plus_code_qrs: block $minus_code_qrs: block $plus_code_qrsd: block $minus_code_qrsd: block)+
    ) => {
        $(
            impl $d_name {
                #[allow(unused)]
                fn scale(self, radius: Distance) -> Self {
                    Self(self.0.saturating_mul(radius.into()))
                }
            }

            // D with self section

            impl core::ops::AddAssign<$d_name> for $d_name {
                fn add_assign(&mut self, other: $d_name) {
                    self.0 += other.0;
                }
            }

            impl core::ops::Add<$d_name> for $d_name {
                type Output = Self;

                fn add(mut self, other: $d_name) -> Self::Output {
                    self += other;
                    self
                }
            }

            impl core::ops::SubAssign<$d_name> for $d_name {
                fn sub_assign(&mut self, other: $d_name) {
                    self.0 -= other.0;
                }
            }

            impl core::ops::Sub<$d_name> for $d_name {
                type Output = Self;

                fn sub(mut self, other: $d_name) -> Self::Output {
                    self -= other;
                    self
                }
            }

            // D with Base section

            impl core::ops::AddAssign<$d_name> for $base_name {
                fn add_assign(&mut self, other: $d_name) {
                    self.0 += other.0;
                }
            }

            impl core::ops::Add<$d_name> for $base_name {
                type Output = Self;

                fn add(mut self, other: $d_name) -> Self::Output {
                    self += other;
                    self
                }
            }

            impl core::ops::SubAssign<$d_name> for $base_name {
                fn sub_assign(&mut self, other: $d_name) {
                    self.0 -= other.0;
                }
            }

            impl core::ops::Sub<$d_name> for $base_name {
                type Output = Self;

                fn sub(mut self, other: $d_name) -> Self::Output {
                    self -= other;
                    self
                }
            }

            // D with QRS section

            impl core::ops::AddAssign<$d_name> for QRS {
                fn add_assign(&mut $self, $other: $d_name) {
                    $plus_code_qrs
                }
            }

            impl core::ops::Add<$d_name> for QRS {
                type Output = Self;

                fn add(mut self, other: $d_name) -> Self::Output {
                    self += other;
                    self
                }
            }

            impl core::ops::SubAssign<$d_name> for QRS {
                fn sub_assign(&mut $self, $other: $d_name) {
                    $minus_code_qrs
                }
            }

            impl core::ops::Sub<$d_name> for QRS {
                type Output = Self;

                fn sub(mut self, other: $d_name) -> Self::Output {
                    self -= other;
                    self
                }
            }

            // D with QRSD section

            impl core::ops::AddAssign<$d_name> for QRSD {
                fn add_assign(&mut $self, $other: $d_name) {
                    $plus_code_qrsd
                }
            }

            impl core::ops::Add<$d_name> for QRSD {
                type Output = Self;

                fn add(mut self, other: $d_name) -> Self::Output {
                    self += other;
                    self
                }
            }

            impl core::ops::SubAssign<$d_name> for QRSD {
                fn sub_assign(&mut $self, $other: $d_name) {
                    $minus_code_qrsd
                }
            }

            impl core::ops::Sub<$d_name> for QRSD {
                type Output = Self;

                fn sub(mut self, other: $d_name) -> Self::Output {
                    self -= other;
                    self
                }
            }
        )+
    }
}

shared_d_def!{
    [self other]
    QD Q {self.q += other;} {self.q -= other;} {self.qd += other;} {self.qd -= other;}
    RD R {self.r += other;} {self.r -= other;} {self.rd += other;} {self.rd -= other;}
    SD S {self.q.0 += other.0; self.r.0 -= other.0;} {self.q.0 -= other.0; self.r.0 += other.0;} {self.qd.0 += other.0; self.rd.0 -= other.0;} {self.qd.0 -= other.0; self.rd.0 += other.0;}
}

/// A delta in QRS space, as opposed to a QRS, which is a point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QRSD {
    qd: QD,
    rd: RD,
}

impl From<Dir> for QRSD {
    fn from(dir: Dir) -> Self {
        dir.basis()
    }
}

impl From<QD> for QRSD {
    fn from(d: QD) -> Self {
        let mut output = Self::default();
        output += d;
        output
    }
}

impl From<RD> for QRSD {
    fn from(d: RD) -> Self {
        let mut output = Self::default();
        output += d;
        output
    }
}

impl From<SD> for QRSD {
    fn from(d: SD) -> Self {
        let mut output = Self::default();
        output += d;
        output
    }
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Targeting {
    pub source: QRS,
    pub target: QRS,
}

pub fn adjacent_dir(
    Targeting { source, target }: Targeting,
) -> Option<Dir> {
    for dir in Dir::ALL {
        if source + dir.basis() == target { 
            return Some(dir)
        }
    }

    None
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

#[cfg(test)]
mod spiral_has_no_duplicates {
    use super::*;

    use std::collections::{BTreeSet};

    macro_rules! a {
        ($radius: expr, $center: expr $(,)?) => ({
            let mut seen = BTreeSet::new();

            for qrs in spiral($radius, $center) {
                assert!(
                    !seen.contains(&qrs),
                    "{qrs:?} was in {seen:?}",
                );

                seen.insert(qrs);
            }
        });
        ($radius: expr) => ({
            a!($radius, <_>::default());
        })
    }

    #[test]
    fn on_the_basic_1_case() { a!(1) }

    #[test]
    fn on_the_basic_2_case() { a!(2) }

    #[test]
    fn on_the_basic_3_case() { a!(3) }

    #[test]
    fn on_this_offset_3_case() { a!(3, QRS { q: Q(-2), r: R(2), }) }
}