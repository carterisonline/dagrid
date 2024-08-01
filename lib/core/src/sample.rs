use std::{
    fmt::Display,
    simd::{f64x2, num::SimdFloat, StdFloat},
};

use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

use crate::*;

macro_rules! impl_unary_ops {
    ($type: ident, [$([$trait: ident, $fn: ident]),+]) => {
        $(
            impl std::ops::$trait for $type {
                type Output = Self;

                fn $fn(mut self) -> Self::Output {
                    self.0 = std::ops::$trait::$fn(self.0);

                    self
                }
            }
        )+
    };
}

macro_rules! impl_parallel {
    ($type: ident, [$($fn: ident),+]) => {
        impl $type {
            $(
                pub fn $fn(mut self) -> Self {
                    self.0 = f64x2::$fn(self.0);

                    self
                }
            )+
        }
    };
}

macro_rules! impl_f64 {
    ($type: ident, [$($fn: ident),+]) => {
        impl $type {
            $(
                pub fn $fn(mut self) -> Self {
                    *self.l_mut() = f64::$fn(self.l());
                    *self.r_mut() = f64::$fn(self.r());

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
                    self.0 = f64x2::$fn(self.0, rhs.0);

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
                    self.0 = f64x2::$fn(self.0, f64x2::splat(n));

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
                    self.0 = std::ops::$trait::$fn(self.0, rhs.0);

                    self
                }
            }

            impl std::ops::$trait<f64> for $type {
                type Output = Self;

                fn $fn(mut self, rhs: f64) -> Self::Output {
                    self.0 = std::ops::$trait::$fn(self.0, f64x2::splat(rhs));

                    self
                }
            }

            impl std::ops::$trait<$type> for f64 {
                type Output = $type;

                fn $fn(self, mut rhs: $type) -> Self::Output {
                    rhs.0 = std::ops::$trait::$fn(f64x2::splat(self), rhs.0);

                    rhs
                }
            }
        )+
    };
}

newtype!([cc, d, e] pub Sample = f64x2);

impl Serialize for Sample {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for e in self.0.to_array() {
            seq.serialize_element(&e)?;
        }
        seq.end()
    }
}

struct SIMDVisitor;

impl<'de> Visitor<'de> for SIMDVisitor {
    type Value = f64x2;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a vector of floats with a length of 2")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut x = [0.0; 2];
        for (i, y) in x.iter_mut().enumerate() {
            *y = seq
                .next_element::<f64>()?
                .ok_or(serde::de::Error::invalid_length(
                    i,
                    &"f64x2 with 2 elements",
                ))?;
        }

        Ok(f64x2::from_array(x))
    }
}

impl<'de> Deserialize<'de> for Sample {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(deserializer.deserialize_seq(SIMDVisitor)?))
    }
}

impl Sample {
    /// Creates a sample with one (mono) channel.
    pub const fn mono(val: f64) -> Self {
        Self(f64x2::from_array([val, val]))
    }

    /// Creates a sample with two (stereo) channels.
    pub const fn stereo(l: f64, r: f64) -> Self {
        Self(f64x2::from_array([l, r]))
    }

    /// Returns the value of the left channel.
    pub const fn l(&self) -> f64 {
        self.0.as_array()[0]
    }

    /// Returns the value of the right channel.
    pub const fn r(&self) -> f64 {
        self.0.as_array()[1]
    }

    /// Returns a mutable reference to the right channel.
    pub fn l_mut(&mut self) -> &mut f64 {
        &mut self.0.as_mut_array()[0]
    }

    /// Returns a mutable reference to the left channel.
    pub fn r_mut(&mut self) -> &mut f64 {
        &mut self.0.as_mut_array()[1]
    }

    pub fn powi(mut self, n: i32) -> Self {
        *self.l_mut() = self.l().powi(n);
        *self.r_mut() = self.r().powi(n);

        self
    }

    pub fn clamp(mut self, min: f64, max: f64) -> Self {
        self.0 = self.0.simd_clamp(f64x2::splat(min), f64x2::splat(max));

        self
    }

    pub fn mul_add(mut self, a: f64, b: f64) -> Self {
        *self.l_mut() = self.l().mul_add(a, b);
        *self.r_mut() = self.r().mul_add(a, b);

        self
    }

    pub fn powf(mut self, n: f64) -> Self {
        *self.l_mut() = self.l().powf(n);
        *self.r_mut() = self.r().powf(n);

        self
    }

    pub fn sin_cos(self) -> (Self, Self) {
        let (lsin, lcos) = self.l().sin_cos();
        let (rsin, rcos) = self.r().sin_cos();

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
        write!(f, "({}, {})", self.l(), self.r())
    }
}

impl_ops!(Sample, [[Add, add], [Sub, sub], [Mul, mul], [Div, div]]);
impl_unary_ops!(Sample, [[Neg, neg]]);
impl_parallel!(
    Sample,
    [
        floor, ceil, round, trunc, fract, abs, signum, sqrt, exp, exp2, ln, log2, log10, sin, cos,
        recip, to_degrees, to_radians
    ]
);

impl_f64!(
    Sample,
    [
        round_ties_even,
        cbrt,
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
        atanh
    ]
);

impl_parallel2!(
    Sample,
    [copysign, /*div_euclid, rem_euclid, hypot, atan2,*/ simd_max, simd_min]
);

impl_parallel_f64!(Sample, [log]);
