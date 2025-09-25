use serde::{Deserialize, Serialize};
use serde_json::Number as JsonNumber;

use crate::{hash::hash_bytes, CanonicalizeError};

/// Represents a JSON number using IEEE-754 double precision, mirroring Go's `float64`.
#[derive(Clone, Copy, Debug, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Number(f64);

impl Number {
    /// Creates a new [`Number`] after validating finiteness.
    ///
    /// ```
    /// # use jd_core::Number;
    /// let num = Number::new(42.0)?;
    /// assert_eq!(num.get(), 42.0);
    /// # Ok::<(), jd_core::CanonicalizeError>(())
    /// ```
    pub fn new(value: f64) -> Result<Self, CanonicalizeError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(CanonicalizeError::NotFinite { value })
        }
    }

    /// Returns the raw floating-point value.
    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }

    /// Compares two numbers using the provided absolute tolerance.
    #[must_use]
    pub fn equals_with_precision(self, other: Self, precision: f64) -> bool {
        (self.0 - other.0).abs() <= precision
    }

    /// Computes the hash code following the Go implementation's strategy.
    #[must_use]
    pub fn hash_code(self) -> crate::hash::HashCode {
        hash_bytes(&self.0.to_le_bytes())
    }

    /// Converts the number into a `serde_json::Number` using minimal integer representation when possible.
    pub fn to_json_number(self) -> JsonNumber {
        if self.0.fract() == 0.0 && !(self.0 == 0.0 && self.0.is_sign_negative()) {
            if (i64::MIN as f64) <= self.0 && self.0 <= (i64::MAX as f64) {
                return JsonNumber::from(self.0 as i64);
            }
            if self.0 >= 0.0 && self.0 <= (u64::MAX as f64) {
                return JsonNumber::from(self.0 as u64);
            }
        }
        JsonNumber::from_f64(self.0).expect("finite number")
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
