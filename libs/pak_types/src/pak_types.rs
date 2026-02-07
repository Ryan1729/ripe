use gfx_sizes::*;

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

    impl WH {
        /// Halves both components
        pub const fn halve(self) -> Self {
            Self {
                w: self.w.halve(),
                h: self.h.halve(),
            }
        }
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

pub mod sprite {
    pub use super::unscaled::{self, W, H, WH};
    use std::marker::PhantomData;

    /// Marker
    /// The rendering commands store only allow `sprite::XY<Renderable>`
    /// so all other types must be converted to `sprite::XY<Renderable>`
    /// via a `sprite::Spec<A>` for the appropriate `A`.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Renderable;

    /// Marker
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct BaseFont;

    /// Marker
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct BaseUI;

    /// Marker
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct BaseTiles;
    
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

    impl <Marker> From<X<Marker>> for u16 {
        fn from(x: X<Marker>) -> Self {
            x.0.into()
        }
    }

    impl <Marker> From<Y<Marker>> for u16 {
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

    #[derive(Clone, Debug)]
    pub struct Spec<Marker> {
        offset: WH,
        tile: WH,
        marker: PhantomData<Marker>,
    }

    impl <Marker> Spec<Marker> {
        pub fn tile(&self) -> WH {
            self.tile
        }

        pub fn tile_center_offset(&self) -> WH {
            self.tile.halve()
        }

        /// Return a tile sized rect
        pub fn rect(&self, unscaled::XY{ x, y }: unscaled::XY) -> unscaled::Rect {
            unscaled::Rect {
                x,
                y,
                w: self.tile.w,
                h: self.tile.h,
            }
        }

        /// Take an unscaled::XY representing the center of a tile, and return the min corner of the tile.
        pub fn center_to_min_corner(&self, xy: unscaled::XY) -> unscaled::XY {
            xy - self.tile_center_offset()
        }

        /// Take an unscaled::XY and return a tile sized rect, with the offset applied to it.
        pub fn offset_rect(
            &self,
            offset: offset::XY,
            base_corner: unscaled::XY
        ) -> unscaled::Rect {
            let tile = self.tile;
            let mut output = self.rect(base_corner);
        
            if offset.x > offset::X::ZERO {
                output.x += unscaled::W::from(
                    offset::Inner::from(offset.x) * offset::Inner::from(tile.w)
                );
            } else if offset.x < offset::X::ZERO {
                output.x -= unscaled::W::from(
                    offset::Inner::from(offset.x).abs() * offset::Inner::from(tile.w)
                );
            } else {
                // do nothing for zeroes or other weird values.
            }
        
            if offset.y > offset::Y::ZERO {
                output.y += unscaled::H::from(
                    offset::Inner::from(offset.y) * offset::Inner::from(tile.h)
                );
            } else if offset.y < offset::Y::ZERO {
                output.y -= unscaled::H::from(
                    offset::Inner::from(offset.y).abs() * offset::Inner::from(tile.h)
                );
            } else {
                // do nothing for zeroes or other weird values.
            }
        
            output
        }

        /// Not advised for general use, but only when initally constructing the specs
        /// while retaining the default values from Specs.
        // TODO? Is there a clean way to allow that to work, and avoid exposing this?
        // Exposing this is not the biggest deal of course. Keeping the rune stuff in 
        // one crate seems more important than this project's code "relying too much"
        // on itself.
        pub fn pieces(&self) -> SpecPieces {
            SpecPieces {
                offset: self.offset,
                tile: self.tile,
            }
        }
    }

    pub struct SpecPieces {
        pub offset: WH,
        pub tile: WH,
    }

    pub fn spec<Marker>(SpecPieces { offset, tile }: SpecPieces) -> Spec<Marker> {
        Spec::<Marker> {
            offset,
            tile,
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

    #[derive(Clone, Debug)]
    pub struct Specs {
        pub base_font: Spec<BaseFont>,
        pub base_tiles: Spec<BaseTiles>,
        pub base_ui: Spec<BaseUI>,
        pub ice_puzzles: Spec<IcePuzzles>,
        pub sword: Spec<SWORD>,
    }
    
    impl Default for Specs {
        fn default() -> Self {
            Self {
                base_font: spec::<BaseFont>(SpecPieces{
                    offset: WH{ w: W(0), h: H(128) },
                    tile: WH{ w: W(8), h: H(8) }
                }),
                base_tiles: spec::<BaseTiles>(SpecPieces{
                    offset: WH{ w: W(0), h: H(0) },
                    tile: WH{ w: W(16), h: H(16) }
                }),
                base_ui: spec::<BaseUI>(SpecPieces{
                    offset: WH{ w: W(0), h: H(0) },
                    tile: WH{ w: W(8), h: H(8) }
                }),
                ice_puzzles: spec::<IcePuzzles>(SpecPieces{
                    offset: WH{ w: W(128), h: H(0) },
                    tile: WH{ w: W(20), h: H(20) }
                }),
                sword: spec::<SWORD>(SpecPieces{
                    offset: WH{ w: W(128), h: H(48) },
                    tile: WH{ w: W(16), h: H(16) }
                }),
            }
        }
    }
}
pub use sprite::Specs;

/// 64k entity definitions ought to be enough for anybody!
pub type DefId = u16;
// TODO? allow large enough deltas to represent going from DefId::MIN to DefId::MAX?
//     I suspect no one will
pub type DefIdDelta = i16;

pub type DefIdNextLargerSigned = i32;

pub type SegmentWidth = usize;

/// 64k world segments ought to be enough for anybody!
pub type SegmentId = u16;

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

pub type TileSprite = u8;

pub mod config {
    use vec1::{Vec1};
    use crate::{
        consts::{EntityDefFlags, TileFlags},
        DefId, OnCollect, SegmentWidth, Specs, Speech, TileSprite
    };
    use std::path::PathBuf;

    /// A configuration WorldSegment that can be used to contruct game::WorldSegments later.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct WorldSegment {
        pub width: SegmentWidth,
        // TODO Since usize is u32 on wasm, let's make a Vec32 type that makes that restriction clear, so we
        // can't have like PC only worlds that break in weird ways online. Probably no one will ever need that
        // many tiles per segment. Plus, then xs conversions go away.
        pub tiles: Vec1<TileFlags>,
    }

    #[derive(Clone, Debug)]
    pub struct Config {
        pub segments: Vec1<WorldSegment>,
        pub entities: Vec1<EntityDef>,
        pub hallways: Vec1<HallwaySpec>,
    }

    #[derive(Clone, Debug)]
    pub struct Manifest {
        pub name: String,
        pub config_path: PathBuf,
        pub spritesheet_path: PathBuf,
        pub specs: Specs,
    }

    impl Manifest {
        pub fn paths(&self) -> impl Iterator<Item = &std::path::Path> {
            [
                self.config_path.as_path(),
                self.spritesheet_path.as_path()
            ].into_iter()
        }
    }

    pub type SpeechesList = Vec<Vec1<Speech>>;

    #[derive(Clone, Debug)]
    pub struct EntityDef {
        pub speeches: SpeechesList,
        pub inventory_description: SpeechesList,
        pub id: DefId,
        pub flags: EntityDefFlags,
        pub tile_sprite: TileSprite,
        pub wants: Vec<DefId>,
        pub on_collect: OnCollect,
    }

    #[derive(Clone, Debug, Default)]
    pub enum HallwaySpec {
        #[default]
        None,
        IcePuzzle,
        SWORD,
    }
}
pub use config::{Config, EntityDef, SpeechesList};

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
    
    pub type CollectActionKind = u8;
    
    consts_def!{
        ALL_COLLECT_ACTION_KINDS: CollectActionKind;
        TRANSFORM = 1,
    }

    pub type EntityDefIdRefKind = u8;
    
    consts_def!{
        ALL_ENTITY_ID_REFERENCE_KINDS: EntityDefIdRefKind;
        RELATIVE = 1,
        ABSOLUTE = 2,
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

    pub type HallwayKind = u8;
    
    consts_def!{
        ALL_HALLWAY_KINDS: CollectActionKind;
        NONE = 0,
        ICE_PUZZLE = 1,
        SWORD = 2,
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
}

pub type EntityFlags = u8;

pub const COLLECTABLE: EntityFlags = 1 << 0;
pub const STEPPABLE: EntityFlags = 1 << 1;
pub const VICTORY: EntityFlags = 1 << 2;
pub const DOOR: EntityFlags = 1 << 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transform {
    pub from: DefId, 
    pub to: DefId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollectAction {
    Transform(Transform),
}

pub type OnCollect = Vec<CollectAction>;

pub struct Spritesheet {
    pub pixels: Vec<ARGB>,
    pub width: usize,
}

impl Spritesheet {
    pub fn slice(&self) -> (&[ARGB], usize) {
        (&self.pixels, self.width)
    }
}

pub struct Pak {
    pub config: Config,
    pub spritesheet: Spritesheet,
    pub specs: Specs,
}
