pub type ARGB = u32;

type Float = f32;

// TODO? Worth caching here?
fn cbrt(x: Float) -> Float {
    // TODO The documentation for this function says the precision can vary from 
    // platform to platform. Is it worth implementing our own version here?
    Float::cbrt(x)
}

#[derive(Clone, Copy)]
struct Oklab {
    l: Float,
    a: Float,
    b: Float,
}

impl Oklab {
    // Conversions based on https://github.com/colorjs/color-space

    fn from_argb(argb: ARGB) -> Self {
        let r = (((argb & 0xFF_0000) >> 16) as Float) / 255.0;
        let g = (((argb & 0xFF_0000) >> 8) as Float) / 255.0;
        let b = ((argb & 0xFF_0000) as Float) / 255.0;

        // Original RGB to LMS matrix coefficients
        let l: Float = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let m: Float = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let s: Float = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
        
        // Cube root the LMS values
        let l_: Float = cbrt(l);
        let m_: Float = cbrt(m);
        let s_: Float = cbrt(s);
        
        // Convert LMS to Oklab using direct coefficients
        Self {
            l: 0.2104542553 * l_ + 0.793617785 * m_ - 0.0040720468 * s_,  // L
            a: 1.9779984951 * l_ - 2.428592205 * m_ + 0.4505937099 * s_,  // a
            b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.808675766 * s_   // b
        }
    }

    /// Always returns the maximum value for the alpha channel
    fn to_argb(self) -> ARGB {
        // Step 1: Convert Oklab to linear LMS
        let l_: Float = self.l + 0.3963377774 * self.a + 0.2158037573 * self.b;
        let m_: Float = self.l - 0.1055613458 * self.a - 0.0638541728 * self.b;
        let s_: Float = self.l - 0.0894841775 * self.a - 1.291485548 * self.b;
        
        // Step 2: Cube the values (reverse of cube root)
        let l3: Float = l_ * l_ * l_;
        let m3: Float = m_ * m_ * m_;
        let s3: Float = s_ * s_ * s_;
        
        // Step 3: Convert LMS to RGB with CORRECTED inverse matrix
        let r = (4.0767416621 * l3 - 3.307711591 * m3 + 0.2309699292 * s3) * 255.0;
        let g = (-1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3) * 255.0;
        let b = (-0.0041960863 * l3 - 0.7034186147 * m3 + 1.707614701 * s3) * 255.0;

        0xFF00_0000 
        | (ARGB::from(r as u8) << 16)
        | (ARGB::from(g as u8) << 8)
        | (ARGB::from(b as u8))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DarkMiddleBright<A> {
    pub dark: A,
    pub middle: A,
    pub bright: A,
}

/// Always returns the maximum value for the alpha channels.
impl From<ARGB> for DarkMiddleBright<ARGB> {
    fn from(hue: ARGB) -> Self {
        let mut oklab = Oklab::from_argb(hue);

        oklab.l = 0.4;
        let dark = oklab.to_argb();
        oklab.l = 0.6;
        let middle = oklab.to_argb();
        oklab.l = 0.8;
        let bright = oklab.to_argb();

        DarkMiddleBright {
            dark,
            middle,
            bright,
        }
    }
}