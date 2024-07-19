use std::fmt::Display;

use serde::{Deserialize, Serialize};

macro_rules! impl_unary_ops {
    ($type: ident, [$([$trait: ident, $fn: ident]),+]) => {
        $(
            impl std::ops::$trait for $type {
                type Output = Self;

                fn $fn(mut self) -> Self::Output {
                    *self.l_mut() = std::ops::$trait::$fn(self.l);
                    *self.r_mut() = std::ops::$trait::$fn(self.r);

                    self
                }
            }
        )+
    };
}

macro_rules! impl_parallel {
    ($type: ident, [$($fn: ident),+,]) => {
        impl $type {
            $(
                pub fn $fn(mut self) -> Self {
                    *self.l_mut() = f64::$fn(self.l);
                    *self.r_mut() = f64::$fn(self.r);

                    self
                }
            )+
        }
    };
}

macro_rules! impl_parallel2 {
    ($type: ident, [$($fn: ident),+]) => {
        impl $type {
            $(
                pub fn $fn(mut self, rhs: Self) -> Self {
                    *self.l_mut() = f64::$fn(self.l, rhs.l);
                    *self.r_mut() = f64::$fn(self.r, rhs.r);

                    self
                }
            )+
        }
    };
}

macro_rules! impl_parallel_f64 {
    ($type: ident, [$($fn: ident),+]) => {
        impl $type {
            $(
                pub fn $fn(mut self, n: f64) -> Self {
                    *self.l_mut() = f64::$fn(self.l, n);
                    *self.r_mut() = f64::$fn(self.r, n);

                    self
                }
            )+
        }
    };
}

macro_rules! impl_ops {
    ($type: ident, [$([$trait: ident, $fn: ident]),+]) => {
        $(
            impl std::ops::$trait<$type> for $type {
                type Output = Self;

                fn $fn(mut self, rhs: Self) -> Self::Output {
                    *self.l_mut() = std::ops::$trait::$fn(self.l, rhs.l);
                    *self.r_mut() = std::ops::$trait::$fn(self.r, rhs.r);

                    self
                }
            }

            impl std::ops::$trait<f64> for $type {
                type Output = Self;

                fn $fn(mut self, rhs: f64) -> Self::Output {
                    *self.l_mut() = std::ops::$trait::$fn(self.l, rhs);
                    *self.r_mut() = std::ops::$trait::$fn(self.r, rhs);

                    self
                }
            }

            impl std::ops::$trait<$type> for f64 {
                type Output = $type;

                fn $fn(self, mut rhs: $type) -> Self::Output {
                    *rhs.l_mut() = std::ops::$trait::$fn(self, rhs.l);
                    *rhs.r_mut() = std::ops::$trait::$fn(self, rhs.r);

                    rhs
                }
            }
        )+
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    l: f64,
    r: f64,
}

impl Sample {
    /// Creates a sample with one (mono) channel.
    pub const fn mono(val: f64) -> Self {
        Self { l: val, r: val }
    }

    /// Creates a sample with two (stereo) channels.
    pub const fn stereo(l: f64, r: f64) -> Self {
        Self { l, r }
    }

    /// Returns the value of the left channel.
    pub const fn l(&self) -> f64 {
        self.l
    }

    /// Returns the value of the right channel.
    pub const fn r(&self) -> f64 {
        self.r
    }

    /// Returns a mutable reference to the right channel.
    pub fn l_mut(&mut self) -> &mut f64 {
        &mut self.l
    }

    /// Returns a mutable reference to the left channel.
    pub fn r_mut(&mut self) -> &mut f64 {
        &mut self.r
    }

    pub fn powi(mut self, n: i32) -> Self {
        *self.l_mut() = self.l.powi(n);
        *self.r_mut() = self.r.powi(n);

        self
    }

    pub fn clamp(mut self, min: f64, max: f64) -> Self {
        *self.l_mut() = self.l.clamp(min, max);
        *self.r_mut() = self.r.clamp(min, max);

        self
    }

    pub fn mul_add(mut self, a: f64, b: f64) -> Self {
        *self.l_mut() = self.l.mul_add(a, b);
        *self.r_mut() = self.r.mul_add(a, b);

        self
    }

    pub fn sin_cos(self) -> (Self, Self) {
        let (lsin, lcos) = self.l.sin_cos();
        let (rsin, rcos) = self.r.sin_cos();

        (Self::stereo(lsin, rsin), Self::stereo(lcos, rcos))
    }
}

impl From<f64> for Sample {
    fn from(value: f64) -> Self {
        Self::mono(value)
    }
}

impl Default for Sample {
    fn default() -> Self {
        Self::mono(f64::NAN)
    }
}

impl Display for Sample {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.l, self.r)
    }
}

impl_ops!(Sample, [[Add, add], [Sub, sub], [Mul, mul], [Div, div]]);
impl_unary_ops!(Sample, [[Neg, neg]]);
impl_parallel!(
    Sample,
    [
        floor,
        ceil,
        round,
        round_ties_even,
        trunc,
        fract,
        abs,
        signum,
        sqrt,
        exp,
        exp2,
        ln,
        log2,
        log10,
        cbrt,
        sin,
        cos,
        tan,
        asin,
        acos,
        atan,
        exp_m1,
        ln_1p,
        sinh,
        cosh,
        tanh,
        asinh,
        acosh,
        atanh,
        recip,
        to_degrees,
        to_radians,
    ]
);

impl_parallel2!(
    Sample,
    [copysign, div_euclid, rem_euclid, hypot, atan2, max, min]
);

impl_parallel_f64!(Sample, [powf, log]);
