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
    /// let num = Number::new(42.0).expect("finite");
    /// assert_eq!(num.get(), 42.0);
    /// ```
    pub fn new(value: f64) -> Result<Self, CanonicalizeError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(CanonicalizeError::NotFinite { value })
        }
    }

    /// Returns the raw floating-point value.
    ///
    /// ```
    /// # use jd_core::Number;
    /// let num = Number::new(1.5).expect("finite");
    /// assert_eq!(num.get(), 1.5);
    /// ```
    #[must_use]
    pub fn get(self) -> f64 {
        self.0
    }

    /// Compares two numbers using the provided absolute tolerance.
    ///
    /// ```
    /// # use jd_core::Number;
    /// let lhs = Number::new(10.0).expect("finite");
    /// let rhs = Number::new(10.4).expect("finite");
    /// assert!(lhs.equals_with_precision(rhs, 0.5));
    /// ```
    #[must_use]
    pub fn equals_with_precision(self, other: Self, precision: f64) -> bool {
        (self.0 - other.0).abs() <= precision
    }

    /// Computes the hash code following the Go implementation's strategy.
    ///
    /// ```
    /// # use jd_core::{DiffOptions, Node, Number};
    /// let node = Node::Number(Number::new(3.14).expect("finite"));
    /// let hash = node.hash_code(&DiffOptions::default());
    /// assert_eq!(hash.len(), 8);
    /// ```
    #[must_use]
    pub fn hash_code(self) -> crate::hash::HashCode {
        hash_bytes(&self.0.to_le_bytes())
    }

    /// Converts the number into a `serde_json::Number` using minimal integer representation when possible.
    ///
    /// ```
    /// # use jd_core::Number;
    /// let as_int = Number::new(5.0).expect("finite").to_json_number();
    /// assert_eq!(as_int.as_i64().unwrap(), 5);
    /// let as_float = Number::new(5.25).expect("finite").to_json_number();
    /// assert!(as_float.as_f64().unwrap() > 5.0);
    /// ```
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
