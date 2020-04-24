use core::cmp::Ordering;
use core::fmt;
use core::num::FpCategory;

/// Represents a two-word floating point type, represented as the sum of two
/// non-overlapping f64 values.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TwoFloat {
    pub(crate) hi: f64,
    pub(crate) lo: f64,
}

#[inline]
fn exponent(x: f64) -> u32 {
    ((x.to_bits() >> 52) & 0x7ff) as u32
}

/// Checks if two `f64` values do not overlap, with the first value being the
/// more significant.
///
/// # Examples
///
/// ```
/// # use twofloat::no_overlap;
/// let a = no_overlap(1.0, -1e-200);
/// let b = no_overlap(1e-200, 1.0);
/// let c = no_overlap(1.0, 0.25);
///
/// assert!(a);
/// assert!(!b);
/// assert!(!c);
pub fn no_overlap(a: f64, b: f64) -> bool {
    match (a.classify(), b.classify()) {
        (FpCategory::Normal, FpCategory::Normal) => {
            exponent(a) >= exponent(b) + f64::MANTISSA_DIGITS
        }
        (FpCategory::Normal, FpCategory::Subnormal) => {
            let a_exponent = exponent(a);
            println!("a_exponent = {}", a_exponent);
            if a_exponent >= f64::MANTISSA_DIGITS {
                true
            } else {
                let b_mantissa = b.to_bits() & ((1 << 52) - 1);
                a_exponent >= 65 - b_mantissa.leading_zeros()
            }
        }
        (FpCategory::Normal, FpCategory::Zero) => true,
        (FpCategory::Subnormal, FpCategory::Zero) => true,
        (FpCategory::Zero, FpCategory::Zero) => true,
        _ => false,
    }
}

impl TwoFloat {
    /// Returns the high word of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use twofloat::TwoFloat;
    /// let value = TwoFloat::new_add(1.0, -1.0e-200);
    /// assert_eq!(value.hi(), 1.0);
    pub fn hi(&self) -> f64 {
        self.hi
    }

    /// Returns the low word of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use twofloat::TwoFloat;
    /// let value = TwoFloat::new_add(1.0, -1.0e-200);
    /// assert_eq!(value.lo(), -1.0e-200);
    pub fn lo(&self) -> f64 {
        self.lo
    }

    /// Returns `true` if `self` is a valid value, where both components are
    /// finite (not infinity or `NAN`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use twofloat::TwoFloat;
    /// let a = TwoFloat::new_add(1.0, 1.0e-300).is_valid();
    /// let b = TwoFloat::new_mul(1.0e300, 1.0e300).is_valid();
    ///
    /// assert!(a);
    /// assert!(!b);
    pub fn is_valid(&self) -> bool {
        self.hi.is_finite() && self.lo.is_finite() && no_overlap(self.hi, self.lo)
    }

    /// Returns the minimum of two numbers. If one of the arguments is `NAN`,
    /// the other is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use twofloat::TwoFloat;
    /// let a = TwoFloat::new_add(35.2, 1e-84);
    /// let b = TwoFloat::new_add(35.2, -1e-93);
    ///
    /// assert_eq!(a.min(b), b);
    pub fn min(self, other: TwoFloat) -> TwoFloat {
        if !self.is_valid() {
            other.clone()
        } else if !other.is_valid() || self <= other {
            self.clone()
        } else {
            other.clone()
        }
    }

    /// Returns the maximum of two numbers. If one of the arguments is `NAN`,
    /// the other is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use twofloat::TwoFloat;
    /// let a = TwoFloat::new_add(35.2, 1e-84);
    /// let b = TwoFloat::new_add(35.2, -1e-93);
    ///
    /// assert_eq!(a.max(b), a);
    pub fn max(self, other: TwoFloat) -> TwoFloat {
        if !self.is_valid() {
            other.clone()
        } else if !other.is_valid() || self >= other {
            self.clone()
        } else {
            other.clone()
        }
    }

    /// Represents an error value equivalent to `f64::NAN`.
    pub const NAN: Self = Self {
        hi: f64::NAN,
        lo: f64::NAN,
    };
}

impl fmt::Display for TwoFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} ({:+})]", self.hi, self.lo)
    }
}

impl PartialEq<f64> for TwoFloat {
    fn eq(&self, other: &f64) -> bool {
        self.hi.eq(other) && self.lo == 0.0
    }
}

impl PartialEq<TwoFloat> for f64 {
    fn eq(&self, other: &TwoFloat) -> bool {
        self.eq(&other.hi) && other.lo == 0.0
    }
}

impl PartialOrd<f64> for TwoFloat {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        let hi_cmp = self.hi.partial_cmp(other);
        if hi_cmp == Some(Ordering::Equal) {
            self.lo.partial_cmp(&0.0)
        } else {
            hi_cmp
        }
    }
}

impl PartialOrd<TwoFloat> for f64 {
    fn partial_cmp(&self, other: &TwoFloat) -> Option<Ordering> {
        let hi_cmp = self.partial_cmp(&other.hi);
        if hi_cmp == Some(Ordering::Equal) {
            0.0.partial_cmp(&other.lo)
        } else {
            hi_cmp
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_overlap_test() {
        assert!(!no_overlap(1.0, (-52f64).exp2()));
        assert!(!no_overlap(-1.0, -(-52f64).exp2()));
        assert!(no_overlap(1.0, (-53f64).exp2()));
        assert!(no_overlap(-1.0, -(-53f64).exp2()));
        assert!(no_overlap(1.0, (-1023f64).exp2()));
        assert!(no_overlap(1.0, -(-1023f64).exp2()));
        assert!(no_overlap(1.0, 0.0));
        assert!(no_overlap(-1.0, -0.0));

        assert!(!no_overlap((-970f64).exp2(), (-1022f64).exp2()));
        assert!(no_overlap((-970f64).exp2(), (-1023f64).exp2()));
        assert!(!no_overlap((-971f64).exp2(), (-1023f64).exp2()));
        assert!(no_overlap((-971f64).exp2(), (-1024f64).exp2()));

        assert!(no_overlap((-1023f64).exp2(), 0.0));
        assert!(!no_overlap((-1023f64).exp2(), f64::MIN));

        assert!(!no_overlap(f64::INFINITY, 1.0));
        assert!(!no_overlap(f64::NAN, 1.0));

        assert!(!no_overlap(0.0, 1.0));
        assert!(!no_overlap(0.0, f64::MIN));
        assert!(no_overlap(0.0, 0.0));
    }
}
