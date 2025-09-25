use serde::{Deserialize, Serialize};

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
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
