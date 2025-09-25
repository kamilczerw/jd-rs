use std::fmt;

use serde::{Deserialize, Serialize};

use crate::OptionsError;

/// Controls how arrays are interpreted during equality and diff operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrayMode {
    /// Arrays behave as ordered lists (default).
    List,
    /// Arrays behave as mathematical sets (order-insensitive, unique elements).
    Set,
    /// Arrays behave as multisets (order-insensitive, duplicate-aware).
    MultiSet,
}

impl Default for ArrayMode {
    fn default() -> Self {
        Self::List
    }
}

/// Configuration knobs passed to equality and diff operations.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiffOptions {
    array_mode: ArrayMode,
    precision: f64,
    set_keys: Option<Vec<String>>,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self { array_mode: ArrayMode::List, precision: 0.0, set_keys: None }
    }
}

impl DiffOptions {
    /// Returns the configured array interpretation mode.
    ///
    /// ```
    /// # use jd_core::{ArrayMode, DiffOptions};
    /// let opts = DiffOptions::default()
    ///     .with_array_mode(ArrayMode::MultiSet)
    ///     .expect("set array mode");
    /// assert_eq!(opts.array_mode(), ArrayMode::MultiSet);
    /// ```
    #[must_use]
    pub fn array_mode(&self) -> ArrayMode {
        self.array_mode
    }

    /// Returns the numeric equality tolerance.
    ///
    /// ```
    /// # use jd_core::DiffOptions;
    /// let opts = DiffOptions::default()
    ///     .with_precision(0.1)
    ///     .expect("set precision");
    /// assert!((opts.precision() - 0.1).abs() < f64::EPSILON);
    /// ```
    #[must_use]
    pub fn precision(&self) -> f64 {
        self.precision
    }

    /// Returns the keys used to identify objects within set semantics.
    ///
    /// ```
    /// # use jd_core::DiffOptions;
    /// let opts = DiffOptions::default()
    ///     .with_set_keys(["id"])
    ///     .expect("set keys");
    /// assert_eq!(opts.set_keys().unwrap(), ["id"]);
    /// ```
    #[must_use]
    pub fn set_keys(&self) -> Option<&[String]> {
        self.set_keys.as_deref()
    }

    /// Sets the array interpretation mode.
    ///
    /// ```
    /// # use jd_core::{ArrayMode, DiffOptions};
    /// let opts = DiffOptions::default()
    ///     .with_array_mode(ArrayMode::Set)
    ///     .expect("set array mode");
    /// assert_eq!(opts.array_mode(), ArrayMode::Set);
    /// ```
    pub fn with_array_mode(mut self, mode: ArrayMode) -> Result<Self, OptionsError> {
        self.array_mode = mode;
        self.validate()?;
        Ok(self)
    }

    /// Sets the numeric precision tolerance.
    ///
    /// ```
    /// # use jd_core::DiffOptions;
    /// let opts = DiffOptions::default()
    ///     .with_precision(0.5)
    ///     .expect("set precision");
    /// assert!((opts.precision() - 0.5).abs() < f64::EPSILON);
    /// ```
    pub fn with_precision(mut self, precision: f64) -> Result<Self, OptionsError> {
        self.precision = precision;
        self.validate()?;
        Ok(self)
    }

    /// Sets the object identity keys used when arrays behave as sets.
    ///
    /// ```
    /// # use jd_core::DiffOptions;
    /// let opts = DiffOptions::default()
    ///     .with_set_keys(["name", "id"])
    ///     .expect("set keys");
    /// assert_eq!(opts.set_keys().unwrap(), ["id", "name"]);
    /// ```
    pub fn with_set_keys<I, S>(mut self, keys: I) -> Result<Self, OptionsError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut collected = Vec::new();
        for key in keys {
            let key = key.into();
            if key.trim().is_empty() {
                return Err(OptionsError::EmptySetKey);
            }
            collected.push(key);
        }
        if collected.is_empty() {
            return Err(OptionsError::EmptySetKey);
        }
        collected.sort();
        collected.dedup();
        self.set_keys = Some(collected);
        self.array_mode = ArrayMode::Set;
        self.validate()?;
        Ok(self)
    }

    fn validate(&self) -> Result<(), OptionsError> {
        if !matches!(self.array_mode, ArrayMode::List) && self.precision > 0.0 {
            return Err(OptionsError::PrecisionIncompatible);
        }
        if self.set_keys.is_some() && !matches!(self.array_mode, ArrayMode::Set) {
            return Err(OptionsError::SetKeysRequireSetMode);
        }
        Ok(())
    }
}

impl fmt::Display for ArrayMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrayMode::List => f.write_str("list"),
            ArrayMode::Set => f.write_str("set"),
            ArrayMode::MultiSet => f.write_str("multiset"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precision_and_set_mode_conflict() {
        let err = DiffOptions::default()
            .with_array_mode(ArrayMode::Set)
            .and_then(|opts| opts.with_precision(0.1))
            .unwrap_err();
        assert_eq!(err, OptionsError::PrecisionIncompatible);
    }

    #[test]
    fn set_keys_require_non_empty_strings() {
        let err = DiffOptions::default().with_set_keys([" "]).unwrap_err();
        assert_eq!(err, OptionsError::EmptySetKey);
    }

    #[test]
    fn set_keys_force_set_mode() {
        let opts = DiffOptions::default().with_set_keys(["id"]).unwrap();
        assert_eq!(opts.array_mode(), ArrayMode::Set);
        assert_eq!(opts.set_keys().unwrap(), ["id"]);
    }
}
