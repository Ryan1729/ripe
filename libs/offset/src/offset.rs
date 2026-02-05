///! Offsets from a tile, for visual purposes only.

pub type Inner = f32;

/// Distinct from f32::signum in that it returns 0.0 for 0.0, -0.0, NaNs, etc.
fn sign(x: Inner) -> Inner {
    if x > 0.0{
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

const MAX: Inner = 1.;
const MIN: Inner = -MAX;

fn normalize(inner: Inner) -> Inner {
    use std::num::FpCategory;

    match inner.classify() {
        FpCategory::Nan
        | FpCategory::Subnormal
        | FpCategory::Zero => 0.,
        FpCategory::Normal => if inner > MAX {
            MAX
        } else if inner < MIN {
            MIN
        } else {
            inner
        },
        FpCategory::Infinite => MAX.copysign(inner),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct X(Inner);

pub fn x(inner: Inner) -> X {
    X(normalize(inner))
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Y(Inner);

pub fn y(inner: Inner) -> Y {
    Y(normalize(inner))
}

macro_rules! shared_impl {
    ($($name: ident)+) => {
        $(
            impl Ord for $name {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    // Valid because we clamp NaN values away
                    if self.0 < other.0 {
                        std::cmp::Ordering::Less
                    } else if self.0 > other.0 {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }
            }
            
            impl PartialOrd for $name {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }
            
            impl PartialEq for $name {
                fn eq(&self, other: &Self) -> bool {
                    self.0 == other.0
                }
            }
            
            impl Eq for $name {}

            impl $name {
                pub const ZERO: Self = Self(0.);
                pub const ONE: Self = Self(1.);

                pub fn get(self) -> Inner {
                    self.0
                }

                pub fn abs(self) -> Self {
                    Self(self.0.abs())
                }

                pub fn sign(self) -> Self {
                    Self(sign(self.0))
                }

                pub fn decay(&mut self) {
                    const DECAY_RATE: f32 = 1./8.;

                    *self -= self.sign() * DECAY_RATE;
                }
            }

            impl From<$name> for Inner {
                fn from(value: $name) -> Self {
                    Self::from(value.get())
                }
            }

            impl From<Inner> for $name {
                fn from(inner: Inner) -> Self {
                    Self(normalize(inner))
                }
            }

            impl core::ops::AddAssign for $name {
                fn add_assign(&mut self, other: Self) {
                    self.0 += other.0;
                    self.0 = normalize(self.0);
                }
            }

            impl core::ops::Add for $name {
                type Output = Self;

                fn add(mut self, other: Self) -> Self::Output {
                    self += other;
                    self
                }
            }

            impl core::ops::SubAssign for $name {
                fn sub_assign(&mut self, other: Self) {
                    self.0 -= other.0;
                    self.0 = normalize(self.0);
                }
            }

            impl core::ops::Sub for $name {
                type Output = Self;

                fn sub(mut self, other: Self) -> Self::Output {
                    self -= other;
                    self
                }
            }

            impl core::ops::MulAssign<Inner> for $name {
                fn mul_assign(&mut self, inner: Inner) {
                    self.0 *= inner;
                    self.0 = normalize(self.0);
                }
            }
        
            impl core::ops::Mul<Inner> for $name {
                type Output = Self;
        
                fn mul(mut self, inner: Inner) -> Self::Output {
                    self *= inner;
                    self
                }
            }
        
            impl core::ops::Mul<$name> for Inner {
                type Output = $name;
        
                fn mul(self, mut w: $name) -> Self::Output {
                    w *= self;
                    w
                }
            }

            impl core::ops::MulAssign<$name> for $name {
                fn mul_assign(&mut self, other: $name) {
                    self.0 *= other.0;
                    self.0 = normalize(self.0);
                }
            }

            impl core::ops::Mul<$name> for $name {
                type Output = $name;
        
                fn mul(self, mut w: $name) -> Self::Output {
                    w *= self;
                    w
                }
            }
        
            impl core::ops::DivAssign<Inner> for $name {
                fn div_assign(&mut self, inner: Inner) {
                    self.0 /= inner;
                    self.0 = normalize(self.0);
                }
            }
        
            impl core::ops::Div<Inner> for $name {
                type Output = Self;
        
                fn div(mut self, inner: Inner) -> Self::Output {
                    self /= inner;
                    self
                }
            }
        )+
    }
}

shared_impl!{
    X Y
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct XY {
    pub x: X,
    pub y: Y,
}

pub fn xy(x: X, y: Y) -> XY {
    XY { x, y }
}

impl XY {
    pub const ZERO: Self = Self { x: X::ZERO, y: Y::ZERO };
    pub const ONE: Self = Self { x: X::ONE, y: Y::ONE };

    pub fn decay(&mut self) {
        self.x.decay();
        self.y.decay();
    }
}

impl core::ops::AddAssign for XY {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl core::ops::Add for XY {
    type Output = Self;

    fn add(mut self, other: Self) -> Self::Output {
        self += other;
        self
    }
}

impl core::ops::SubAssign for XY {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl core::ops::Sub for XY {
    type Output = Self;

    fn sub(mut self, other: Self) -> Self::Output {
        self -= other;
        self
    }
}

