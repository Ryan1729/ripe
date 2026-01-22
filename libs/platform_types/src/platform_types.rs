pub use gfx_sizes::*;

pub mod unscaled {
    ///! Values are in pixels.

    pub type Inner = u16;

    pub const fn inner_from_u8(byte: u8) -> Inner {
        byte as Inner
    }

    pub type SignedInner = i16;

    macro_rules! def {
        ($($name: ident, $inner_name: ident = $inner_type: ident)+) => {
            $(
                pub type $inner_name = $inner_type;
                #[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
                pub struct $name(pub $inner_name);
    
                impl $name {
                    pub const fn get(self) -> $inner_name {
                        self.0
                    }
                }
    
                impl From<$name> for $inner_name {
                    fn from(to_convert: $name) -> $inner_name {
                        $inner_name::from(to_convert.0)
                    }
                }
            )*
        }
    }

    def!{
        X, XInner = Inner
        Y, YInner = Inner
        W, WInner = Inner
        H, HInner = Inner
        XD, XDInner = SignedInner
        YD, YDInner = SignedInner
    }

    pub const fn w_to_usize(w: W) -> usize {
        w.0 as usize
    }

    impl From<W> for usize {
        fn from(w: W) -> Self {
            w_to_usize(w)
        }
    }

    pub const fn h_to_usize(h: H) -> usize {
        h.0 as usize
    }

    impl From<H> for usize {
        fn from(h: H) -> Self {
            h_to_usize(h)
        }
    }

    pub const fn w_const_add(a: W, b: W) -> W {
        W(a.0 + b.0)
    }

    pub const fn w_const_sub(a: W, b: W) -> W {
        W(a.0 - b.0)
    }

    pub const fn w_const_mul(a: W, b: Inner) -> W {
        W(a.0 * b)
    }

    pub const fn w_const_div(a: W, b: Inner) -> W {
        W(a.0 / b)
    }

    pub const fn h_const_add(a: H, b: H) -> H {
        H(a.0 + b.0)
    }

    pub const fn h_const_sub(a: H, b: H) -> H {
        H(a.0 - b.0)
    }

    pub const fn h_const_mul(a: H, b: Inner) -> H {
        H(a.0 * b)
    }

    pub const fn h_const_div(a: H, b: Inner) -> H {
        H(a.0 / b)
    }

    pub const fn x_const_add_w(x: X, w: W) -> X {
        X(x.0 + w.0)
    }

    pub const fn y_const_add_h(y: Y, h: H) -> Y {
        Y(y.0 + h.0)
    }

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
        )+}
    }

    unsigned_paired_impls!{
        X, W
        Y, H
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
        X, XD, XInner
        Y, YD, YInner
    }

    impl core::ops::Sub<X> for X {
        type Output = W;

        fn sub(self, other: X) -> Self::Output {
            W(self.0 - other.0)
        }
    }

    impl core::ops::Sub<Y> for Y {
        type Output = H;

        fn sub(self, other: Y) -> Self::Output {
            H(self.0 - other.0)
        }
    }

    impl X {
        pub const fn saturating_add_w(self, w: W) -> X {
            X(self.0.saturating_add(w.0))
        }
        pub const fn saturating_sub_w(self, w: W) -> X {
            X(self.0.saturating_sub(w.0))
        }
        pub const fn saturating_point_sub_w(self, x: X) -> W {
            W(self.0.saturating_sub(x.0))
        }
    }

    impl Y {
        pub const fn saturating_add_h(self, h: H) -> Y {
            Y(self.0.saturating_add(h.0))
        }
        pub const fn saturating_sub_h(self, h: H) -> Y {
            Y(self.0.saturating_sub(h.0))
        }
        pub const fn saturating_point_sub_h(self, x: Y) -> H {
            H(self.0.saturating_sub(x.0))
        }
    }

    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
    pub struct XY {
        pub x: X,
        pub y: Y,
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

    impl core::ops::SubAssign<W> for XY {
        fn sub_assign(&mut self, other: W) {
            self.x -= other;
        }
    }

    impl core::ops::Sub<W> for XY {
        type Output = Self;

        fn sub(mut self, other: W) -> Self::Output {
            self -= other;
            self
        }
    }

    impl core::ops::SubAssign<H> for XY {
        fn sub_assign(&mut self, other: H) {
            self.y -= other;
        }
    }

    impl core::ops::Sub<H> for XY {
        type Output = Self;

        fn sub(mut self, other: H) -> Self::Output {
            self -= other;
            self
        }
    }

    impl core::ops::Sub for XY {
        type Output = WH;

        fn sub(self, other: XY) -> Self::Output {
            WH {
                w: W(self.x.0 - other.x.0),
                h: H(self.y.0 - other.y.0),
            }
        }
    }

    macro_rules! shared_displacement_impl {
        ($($name: ident, $inner_name: ident)+) => {
            $(
                impl core::ops::AddAssign for $name {
                    fn add_assign(&mut self, other: Self) {
                        self.0 += other.0;
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
                    }
                }
    
                impl core::ops::Sub for $name {
                    type Output = Self;
    
                    fn sub(mut self, other: Self) -> Self::Output {
                        self -= other;
                        self
                    }
                }
    
                impl $name {
                    pub fn saturating_sub(self, other: Self) -> Self {
                        Self(self.0.saturating_sub(other.0))
                    }
                }
    
                impl core::ops::MulAssign<$inner_name> for $name {
                    fn mul_assign(&mut self, inner: $inner_name) {
                        self.0 *= inner;
                    }
                }
            
                impl core::ops::Mul<$inner_name> for $name {
                    type Output = Self;
            
                    fn mul(mut self, inner: $inner_name) -> Self::Output {
                        self *= inner;
                        self
                    }
                }
            
                impl core::ops::Mul<$name> for $inner_name {
                    type Output = $name;
            
                    fn mul(self, mut w: $name) -> Self::Output {
                        w *= self;
                        w
                    }
                }
            
                impl core::ops::DivAssign<$inner_name> for $name {
                    fn div_assign(&mut self, inner: $inner_name) {
                        self.0 /= inner;
                    }
                }
            
                impl core::ops::Div<$inner_name> for $name {
                    type Output = Self;
            
                    fn div(mut self, inner: $inner_name) -> Self::Output {
                        self /= inner;
                        self
                    }
                }
            )*
        };
    }
    shared_displacement_impl!{
        W, WInner
        H, HInner
        XD, XDInner
        YD, YDInner
    }

    macro_rules! shared_impl {
        ($($name: ident, $inner_name: ident)+) => {
            $(
                impl $name {
                    pub const MIN: Self = Self($inner_name::MIN);
                    pub const MAX: Self = Self($inner_name::MAX);

                    pub const ZERO: Self = Self(0);
                    pub const ONE: Self = Self(1);
                    pub const TWO: Self = Self(2);

                    pub fn dec(self) -> Self {
                        Self(self.0.saturating_sub(1))
                    }

                    pub fn inc(self) -> Self {
                        Self(self.0.saturating_add(1))
                    }
                }

                impl From<$name> for f32 {
                    fn from(value: $name) -> Self {
                        Self::from(value.get())
                    }
                }

                impl From<f32> for $name {
                    fn from(value: f32) -> Self {
                        // The as cast has the behaviour we want in the cases we know we care about.
                        // https://doc.rust-lang.org/reference/expressions/operator-expr.html#r-expr.as.numeric.float-as-int
                        Self(value as $inner_name)
                    }
                }
            )+
        }
    }

    shared_impl!{
        X, XInner
        Y, YInner
        W, WInner
        H, HInner
        XD, XDInner
        YD, YDInner
    }

    macro_rules! shared_unsigned_impl {
        ($($name: ident)+) => {
            $(
                impl $name {
                    pub fn usize(self) -> usize {
                        self.0.into()
                    }

                    // Note: this won't work for signed types: -1 >> 1 == -1, not 0
                    pub const fn halve(self) -> Self {
                        Self(self.0 >> 1)
                    }
                }
            )+
        }
    }
    shared_unsigned_impl!{
        X Y W H
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct WH {
        pub w: W,
        pub h: H,
    }

    impl core::ops::AddAssign for WH {
        fn add_assign(&mut self, other: WH) {
            self.w += other.w;
            self.h += other.h;
        }
    }

    impl core::ops::Add for WH {
        type Output = Self;

        fn add(mut self, other: WH) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::SubAssign for WH {
        fn sub_assign(&mut self, other: WH) {
            self.w -= other.w;
            self.h -= other.h;
        }
    }

    impl core::ops::Sub for WH {
        type Output = Self;

        fn sub(mut self, other: WH) -> Self::Output {
            self -= other;
            self
        }
    }

    impl core::ops::MulAssign<Inner> for WH {
        fn mul_assign(&mut self, inner: Inner) {
            self.w *= inner;
            self.h *= inner;
        }
    }

    impl core::ops::Mul<Inner> for WH {
        type Output = Self;

        fn mul(mut self, inner: Inner) -> Self::Output {
            self *= inner;
            self
        }
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

    impl core::ops::SubAssign<WH> for XY {
        fn sub_assign(&mut self, other: WH) {
            self.x -= other.w;
            self.y -= other.h;
        }
    }

    impl core::ops::Sub<WH> for XY {
        type Output = Self;

        fn sub(mut self, other: WH) -> Self::Output {
            self -= other;
            self
        }
    }

    impl core::ops::AddAssign<W> for WH {
        fn add_assign(&mut self, other: W) {
            self.w += other;
        }
    }

    impl core::ops::Add<W> for WH {
        type Output = Self;

        fn add(mut self, other: W) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::AddAssign<H> for WH {
        fn add_assign(&mut self, other: H) {
            self.h += other;
        }
    }

    impl core::ops::Add<H> for WH {
        type Output = Self;

        fn add(mut self, other: H) -> Self::Output {
            self += other;
            self
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Rect {
        pub x: X,
        pub y: Y,
        pub w: W,
        pub h: H,
    }

    impl Rect {
        pub fn xy(self) -> XY {
            XY {
                x: self.x,
                y: self.y,
            }
        }

        pub fn max_xy(self) -> XY {
            XY {
                x: self.x + self.w,
                y: self.y + self.h,
            }
        }

        pub fn wh(self) -> WH {
            WH {
                w: self.w,
                h: self.h,
            }
        }

        pub const fn xy_wh(xy: XY, wh: WH) -> Rect {
            Rect {
                x: xy.x,
                y: xy.y,
                w: wh.w,
                h: wh.h,
            }
        }
    }
}

pub type PaletteIndex = u8;

pub mod sprite {
    pub use super::unscaled::{W, H, WH};
    use std::marker::PhantomData;

    /// Marker
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Renderable;

    /// Marker
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Rooms;
    
    /// Marker
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct IcePuzzles;
    
    /// Marker
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct SWORD;

    pub type Inner = u16;
    #[derive(Debug, PartialEq, Eq)]
    pub struct X<Marker>(Inner, PhantomData<Marker>);

    impl<Marker> Clone for X<Marker> {
        fn clone(&self) -> Self {
            *self
        }
    }
    
    impl<Marker> Copy for X<Marker> {}

    impl <Marker> Default for X<Marker> {
        fn default() -> Self {
            Self(<_>::default(), PhantomData)
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Y<Marker>(Inner, PhantomData<Marker>);

    impl<Marker> Clone for Y<Marker> {
        fn clone(&self) -> Self {
            *self
        }
    }
    
    impl<Marker> Copy for Y<Marker> {}

    impl <Marker> Default for Y<Marker> {
        fn default() -> Self {
            Self(<_>::default(), PhantomData)
        }
    }

    pub const fn x<Marker>(inner: Inner) -> X<Marker> {
        X(
            inner,
            PhantomData,
        )
    }

    pub const fn y<Marker>(inner: Inner) -> Y<Marker> {
        Y(
            inner,
            PhantomData,
        )
    }

    impl <Marker> From<X<Marker>> for usize {
        fn from(x: X<Marker>) -> Self {
            x.0.into()
        }
    }

    impl <Marker> From<Y<Marker>> for usize {
        fn from(y: Y<Marker>) -> Self {
            y.0.into()
        }
    }

    impl <Marker> core::ops::AddAssign<W> for X<Marker> {
        fn add_assign(&mut self, other: W) {
            self.0 += other.0;
        }
    }

    impl <Marker> core::ops::Add<W> for X<Marker> {
        type Output = Self;

        fn add(mut self, other: W) -> Self::Output {
            self += other;
            self
        }
    }

    pub const fn x_const_add_w<Marker>(x: X<Marker>, w: W) -> X<Marker> {
        X(x.0 + w.0, PhantomData)
    }

    impl <Marker> core::ops::AddAssign<H> for Y<Marker> {
        fn add_assign(&mut self, other: H) {
            self.0 += other.0;
        }
    }

    impl <Marker> core::ops::Add<H> for Y<Marker> {
        type Output = Self;

        fn add(mut self, other: H) -> Self::Output {
            self += other;
            self
        }
    }

    pub const fn y_const_add_h<Marker>(y: Y<Marker>, h: H) -> Y<Marker> {
        Y(y.0 + h.0, PhantomData)
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct XY<Marker> {
        pub x: X<Marker>,
        pub y: Y<Marker>,
    }

    impl <Marker> core::ops::AddAssign<W> for XY<Marker> {
        fn add_assign(&mut self, other: W) {
            self.x += other;
        }
    }

    impl <Marker> core::ops::Add<W> for XY<Marker> {
        type Output = Self;

        fn add(mut self, other: W) -> Self::Output {
            self += other;
            self
        }
    }

    impl <Marker> core::ops::AddAssign<H> for XY<Marker> {
        fn add_assign(&mut self, other: H) {
            self.y += other;
        }
    }

    impl <Marker> core::ops::Add<H> for XY<Marker> {
        type Output = Self;

        fn add(mut self, other: H) -> Self::Output {
            self += other;
            self
        }
    }

    pub struct Spec<Marker> {
        offset: WH,
        marker: PhantomData<Marker>,
    }

    pub fn spec<Marker>(offset: WH) -> Spec<Marker> {
        Spec::<Marker> {
            offset,
            marker: PhantomData,
        }
    }

    impl <Marker> X<Marker> {
        pub fn apply(self, spec: &Spec<Marker>) -> X<Renderable> {
            X((self + spec.offset.w).0, PhantomData)
        }
    }

    impl <Marker> Y<Marker> {
        pub fn apply(self, spec: &Spec<Marker>) -> Y<Renderable> {
            Y((self + spec.offset.h).0, PhantomData)
        }
    }

    impl <Marker> XY<Marker> {
        pub fn apply(self, spec: &Spec<Marker>) -> XY<Renderable> {
            XY::<Renderable>{
                x: self.x.apply(spec),
                y: self.y.apply(spec),
            }
        }
    }
}

pub mod command {
    use xs::Xs;
    use super::{ARGB, sprite, unscaled::{self, XD, YD}};

    pub type Inner = unscaled::Inner;
    pub type SignedInner = unscaled::SignedInner;

    // Small enough to fit on pretty much any reasonable device, at an aspect ratio
    // of 3:2 (1.5), which is a compromise between 4:3 (1.33...) and 16:9 (1.788...).
    pub const WIDTH: Inner = 480;
    pub const HEIGHT: Inner = 320;

    pub const LENGTH: usize = WIDTH as usize * HEIGHT as usize;

    pub const WIDTH_W: unscaled::W = unscaled::W(WIDTH);
    pub const HEIGHT_H: unscaled::H = unscaled::H(HEIGHT);

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct X(unscaled::X);

    impl X {
        pub const MAX: X = X(unscaled::X(WIDTH - 1));

        pub const fn get(self) -> unscaled::X {
            self.0
        }

        pub const fn clipped(x: unscaled::X) -> X {
            if x.0 < X::MAX.0.0 {
                X(x)
            } else {
                X::MAX
            }
        }

        pub const fn clipped_inner(x: Inner) -> X {
            X::clipped(unscaled::X(x))
        }

        pub fn gen(rng: &mut Xs) -> X {
            X::clipped(unscaled::X(xs::range(rng, 0..WIDTH as _) as Inner))
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Y(unscaled::Y);

    impl Y {
        pub const MAX: Y = Y(unscaled::Y(HEIGHT - 1));

        pub const fn get(self) -> unscaled::Y {
            self.0
        }

        pub const fn clipped(y: unscaled::Y) -> Y {
            if y.0 < Y::MAX.0.0 {
                Y(y)
            } else {
                Y::MAX
            }
        }

        pub const fn clipped_inner(y: Inner) -> Y {
            Y::clipped(unscaled::Y(y))
        }

        pub fn gen(rng: &mut Xs) -> Y {
            Y::clipped(unscaled::Y(xs::range(rng, 0..WIDTH as _) as Inner))
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct W(unscaled::W);

    impl W {
        pub const MAX: W = W(unscaled::W(WIDTH - 1));

        pub const fn get(self) -> unscaled::W {
            self.0
        }

        pub const fn clipped(w: unscaled::W) -> W {
            if w.0 < W::MAX.0.0 {
                W(w)
            } else {
                W::MAX
            }
        }

        pub const fn clipped_inner(w: Inner) -> W {
            W::clipped(unscaled::W(w))
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct H(unscaled::H);

    impl H {
        pub const MAX: H = H(unscaled::H(WIDTH - 1));

        pub const fn get(self) -> unscaled::H {
            self.0
        }

        pub const fn clipped(h: unscaled::H) -> H {
            if h.0 < H::MAX.0.0 {
                H(h)
            } else {
                H::MAX
            }
        }

        pub const fn clipped_inner(h: Inner) -> H {
            H::clipped(unscaled::H(h))
        }
    }
    /*
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct XD(unscaled::XD);

    impl XD {
        pub const MAX: XD = XD(unscaled::XD(WIDTH as SignedInner - 1));

        pub const fn get(self) -> unscaled::XD {
            self.0
        }

        pub const fn clipped(xd: unscaled::XD) -> XD {
            if xd.0 < XD::MAX.0.0 {
                XD(xd)
            } else {
                XD::MAX
            }
        }

        pub const fn clipped_inner(xd: SignedInner) -> XD {
            XD::clipped(unscaled::XD(xd))
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct YD(unscaled::YD);

    impl YD {
        pub const MAX: YD = YD(unscaled::YD(WIDTH as SignedInner - 1));

        pub const fn get(self) -> unscaled::YD {
            self.0
        }

        pub const fn clipped(yd: unscaled::YD) -> YD {
            if yd.0 < YD::MAX.0.0 {
                YD(yd)
            } else {
                YD::MAX
            }
        }

        pub const fn clipped_inner(yd: SignedInner) -> YD {
            YD::clipped(unscaled::YD(yd))
        }
    }
    */

    pub const fn w_to_usize(w: W) -> usize {
        w.0.0 as usize
    }

    pub const fn h_to_usize(h: H) -> usize {
        h.0.0 as usize
    }

    pub const fn w_const_add(a: W, b: W) -> W {
        W::clipped_inner(a.0.0 + b.0.0)
    }

    pub const fn w_const_sub(a: W, b: W) -> W {
        W::clipped_inner(a.0.0 - b.0.0)
    }

    pub const fn w_const_mul(a: W, b: Inner) -> W {
        W::clipped_inner(a.0.0 * b)
    }

    pub const fn w_const_div(a: W, b: Inner) -> W {
        W::clipped_inner(a.0.0 / b)
    }

    pub const fn h_const_add(a: H, b: H) -> H {
        H::clipped_inner(a.0.0 + b.0.0)
    }

    pub const fn h_const_sub(a: H, b: H) -> H {
        H::clipped_inner(a.0.0 - b.0.0)
    }

    pub const fn h_const_mul(a: H, b: Inner) -> H {
        H::clipped_inner(a.0.0 * b)
    }

    pub const fn h_const_div(a: H, b: Inner) -> H {
        H::clipped_inner(a.0.0 / b)
    }

    impl From<X> for usize {
        fn from(x: X) -> Self {
            x.0.0.into()
        }
    }

    impl From<Y> for usize {
        fn from(y: Y) -> Self {
            y.0.0.into()
        }
    }

    impl From<X> for Inner {
        fn from(to_convert: X) -> Inner {
            to_convert.0.0
        }
    }

    impl From<Y> for Inner {
        fn from(to_convert: Y) -> Inner {
            to_convert.0.0
        }
    }

    impl From<W> for Inner {
        fn from(to_convert: W) -> Inner {
            to_convert.0.0
        }
    }

    impl From<H> for Inner {
        fn from(to_convert: H) -> Inner {
            to_convert.0.0
        }
    }

    impl core::ops::AddAssign<W> for X {
        fn add_assign(&mut self, other: W) {
            *self = Self::clipped(self.0 + other.0);
        }
    }

    impl core::ops::Add<W> for X {
        type Output = Self;

        fn add(mut self, other: W) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::AddAssign<H> for Y {
        fn add_assign(&mut self, other: H) {
            *self = Self::clipped(self.0 + other.0);
        }
    }

    impl core::ops::Add<H> for Y {
        type Output = Self;

        fn add(mut self, other: H) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::MulAssign<Inner> for W {
        fn mul_assign(&mut self, inner: Inner) {
            *self = Self::clipped(self.0 * inner);
        }
    }

    impl core::ops::Mul<Inner> for W {
        type Output = Self;

        fn mul(mut self, inner: Inner) -> Self::Output {
            self *= inner;
            self
        }
    }

    impl core::ops::Mul<W> for Inner {
        type Output = W;

        fn mul(self, mut w: W) -> Self::Output {
            w *= self;
            w
        }
    }

    impl core::ops::MulAssign<Inner> for H {
        fn mul_assign(&mut self, inner: Inner) {
            *self = Self::clipped(self.0 * inner);
        }
    }

    impl core::ops::Mul<Inner> for H {
        type Output = Self;

        fn mul(mut self, inner: Inner) -> Self::Output {
            self *= inner;
            self
        }
    }

    impl core::ops::Mul<H> for Inner {
        type Output = H;

        fn mul(self, mut h: H) -> Self::Output {
            h *= self;
            h
        }
    }

    impl core::ops::AddAssign<XD> for X {
        fn add_assign(&mut self, other: XD) {
            *self = Self::clipped(self.0 + other);
        }
    }


    impl core::ops::Add<XD> for X {
        type Output = Self;

        fn add(mut self, other: XD) -> Self::Output {
            self += other;
            self
        }
    }

    impl core::ops::AddAssign<YD> for Y {
        fn add_assign(&mut self, other: YD) {
            *self = Self::clipped(self.0 + other);
        }
    }

    impl core::ops::Add<YD> for Y {
        type Output = Self;

        fn add(mut self, other: YD) -> Self::Output {
            self += other;
            self
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Rect {
        pub x_min: X,
        pub y_min: Y,
        pub x_max: X,
        pub y_max: Y,
    }

    impl Rect {
        pub fn from_unscaled(
            unscaled::Rect {
                x,
                y,
                w,
                h,
            }: unscaled::Rect
        ) -> Rect {
            Rect {
                x_min: X::clipped(x),
                y_min: Y::clipped(y),
                x_max: X::clipped_inner((x + w).get() - 1),
                y_max: Y::clipped_inner((y + h).get() - 1),
            }
        }

        pub fn unscaled(self) -> unscaled::Rect {
            let Rect {
                x_min,
                y_min,
                x_max,
                y_max,
            }: Rect = self;

            unscaled::Rect {
                x: x_min.get(),
                y: y_min.get(),
                w: x_max.get() - x_min.get() + unscaled::W(1),
                h: y_max.get() - y_min.get() + unscaled::H(1),
            }
        }
    }

    #[test]
    fn from_unscaled_then_unscaled_is_identity_on_this_example() {
        let expected = Rect {
            x_min: X::clipped_inner(2),
            y_min: Y::clipped_inner(3),
            x_max: X::clipped_inner(5),
            y_max: Y::clipped_inner(7),
        };

        let actual = Rect::from_unscaled(expected.unscaled());

        assert_eq!(expected, actual);
    }

    #[test]
    fn unscaled_then_from_unscaled_is_identity_on_this_example() {
        let expected = unscaled::Rect {
            x: unscaled::X(7),
            y: unscaled::Y(5),
            w: unscaled::W(3),
            h: unscaled::H(2),
        };

        let actual = Rect::from_unscaled(expected).unscaled();

        assert_eq!(expected, actual);
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Command {
        pub rect: Rect,
        pub sprite_xy: sprite::XY<sprite::Renderable>,
        pub colour_override: ARGB,
    }    
}
pub use command::Command;

#[derive(Clone, Copy, Default, Debug)]
pub struct Input {
    pub gamepad: Button,
    pub previous_gamepad: Button,
}

impl Input {
    #[allow(unused)]
    pub fn pressed_this_frame(&self, buttons: Button) -> bool {
        !self.previous_gamepad.contains(buttons) && self.gamepad.contains(buttons)
    }

    #[allow(unused)]
    pub fn released_this_frame(&self, buttons: Button) -> bool {
        self.previous_gamepad.contains(buttons) && !self.gamepad.contains(buttons)
    }

    pub fn dir_pressed_this_frame(&self) -> Option<Dir> {
        if self.pressed_this_frame(Button::UP) {
            Some(Dir::Up)
        } else if self.pressed_this_frame(Button::DOWN) {
            Some(Dir::Down)
        } else if self.pressed_this_frame(Button::LEFT) {
            Some(Dir::Left)
        } else if self.pressed_this_frame(Button::RIGHT) {
            Some(Dir::Right)
        } else {
            None
        }
    }

    pub fn contains_dir(&self) -> Option<Dir> {
        if self.gamepad.contains(Button::UP) {
            Some(Dir::Up)
        } else if self.gamepad.contains(Button::DOWN) {
            Some(Dir::Down)
        } else if self.gamepad.contains(Button::LEFT) {
            Some(Dir::Left)
        } else if self.gamepad.contains(Button::RIGHT) {
            Some(Dir::Right)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SFX {
    CardPlace,
    CardSlide,
    ButtonPress,
}

pub struct Speaker {
    requests: Vec<SFX>,
}

impl Default for Speaker {
    fn default() -> Self {
        Speaker {
            requests: Vec::with_capacity(8),
        }
    }
}

impl Speaker {
    pub fn clear(&mut self) {
        self.requests.clear();
    }

    pub fn request_sfx(&mut self, sfx: SFX) {
        self.requests.push(sfx);
    }

    pub fn slice(&self) -> &[SFX] {
        &self.requests
    }
}

// These values are deliberately picked to be the same as the ones in NES' input registers.
pub mod button {
    #[cfg(not(feature = "refresh"))]
    type Inner = u8;

    #[cfg(feature = "refresh")]
    type Inner = u16;

    #[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
    pub struct Button(Inner);

    impl Button {
        pub const A     : Self = Self(1 << 0);
        pub const B     : Self = Self(1 << 1);
        pub const SELECT: Self = Self(1 << 2);
        pub const START : Self = Self(1 << 3);
        pub const UP    : Self = Self(1 << 4);
        pub const DOWN  : Self = Self(1 << 5);
        pub const LEFT  : Self = Self(1 << 6);
        pub const RIGHT : Self = Self(1 << 7);

        #[cfg(feature = "refresh")]
        pub const RESET : Self = Self(1 << 8);

        #[cfg(not(feature = "refresh"))]
        pub const ALL : [Self; 8] = [
            Self::A,
            Self::B,
            Self::SELECT,
            Self::START,
            Self::UP,
            Self::DOWN,
            Self::LEFT,
            Self::RIGHT,
        ];

        #[cfg(feature = "refresh")]
        pub const ALL : [Self; 9] = [
            Self::A,
            Self::B,
            Self::SELECT,
            Self::START,
            Self::UP,
            Self::DOWN,
            Self::LEFT,
            Self::RIGHT,
            Self::RESET,
        ];

        pub const fn contains(&self, other: Self) -> bool {
            self.0 & other.0 == other.0
        }

        pub fn insert(&mut self, other: Self) {
            self.0 |= other.0;
        }

        pub fn remove(&mut self, other: Self) {
            self.0 &= !other.0;
        }
    }
}
pub use button::Button;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Left,
    Right,
    Up,
    Down,
}

pub type Logger = Option<fn(&str) -> ()>;

pub trait PakReader
where 
    Self: std::io::Read + std::io::Seek
{}

impl<T: ?Sized> PakReader for T
where
    Self: std::io::Read + std::io::Seek
{}

pub type PakLoader = Option<fn() -> Option<Box<dyn PakReader>>>;

#[derive(Clone, Copy)]
pub struct StateParams {
    pub seed: [u8; 16], 
    pub logger: Logger,
    pub error_logger: Logger, 
    pub pak_loader: PakLoader,
}

// reportedly colourblind friendly colours
// https://twitter.com/ea_accessible/status/968595073184092160
pub mod colours {
    use super::ARGB;

    pub const BLUE: ARGB = 0xFF3352E1;
    pub const GREEN: ARGB = 0xFF30B06E;
    pub const RED: ARGB = 0xFFDE4949;
    pub const YELLOW: ARGB = 0xFFFFB937;
    pub const PURPLE: ARGB = 0xFF533354;
    #[allow(unused)]
    pub const GREY: ARGB = 0xFF5A7D8B;
    #[allow(unused)]
    pub const GRAY: ARGB = GREY;
    pub const WHITE: ARGB = 0xFFEEEEEE;
    pub const BLACK: ARGB = 0xFF222222;
}

pub use colours::*;

pub const PALETTE: [ARGB; 8] = [
    BLUE,
    GREEN,
    RED,
    YELLOW,
    PURPLE,
    GREY,
    WHITE,
    BLACK,
];

pub mod arrow_timer {
    use crate::unscaled::{self, W, H};

    /// 64k arrow frames ought to be enough for anybody!
    pub type ArrowTimer = u16;
    
    const MAX: ArrowTimer = 128;
    
    pub fn tick(timer: &mut ArrowTimer) {
        if *timer == 0 {
            *timer = MAX;
        } else {
            *timer = timer.saturating_sub(1);
        }
    }

    /// The max W value that will be returned from `offset`.
    pub const MAX_W: unscaled::W = unscaled::W(0);
    /// The max H value that will be returned from `offset`.
    pub const MAX_H: unscaled::H = unscaled::H::TWO;

    pub fn offset(timer: ArrowTimer) -> unscaled::WH {
        if timer < 32 {
            unscaled::WH{ w: W::ZERO, h: H::TWO }
        } else if timer < 64 {
            unscaled::WH{ w: W::ZERO, h: H::ONE }
        } else if timer < 96 {
            unscaled::WH{ w: W::ZERO, h: H::ZERO }
        } else {
            unscaled::WH{ w: W::ZERO, h: H::ONE }
        }
    }
}

/// This is true for the default spritesheet.
pub const TILES_PER_ROW: u8 = 8;