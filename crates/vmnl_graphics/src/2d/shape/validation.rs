// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Shared numeric validation for built-in 2D shape parameters.

use crate::{VMNLError, VMNLErrorKind, VMNLResult};

pub(super) fn validate_finite(values: &[f32], subject: &str) -> VMNLResult<()> {
    if values.iter().any(|value| value.is_nan()) {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "{subject} must not be NaN"
        ))));
    }

    if values.iter().any(|value| value.is_infinite()) {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "{subject} must be finite"
        ))));
    }

    Ok(())
}

pub(super) fn validate_positive_finite(values: &[f32], subject: &str) -> VMNLResult<()> {
    validate_finite(values, subject)?;

    if values.iter().any(|value| *value <= 0.0) {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "{subject} must be strictly positive"
        ))));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_invalid_state(result: VMNLResult<()>, expected: &str) {
        assert!(matches!(
            result,
            Err(error)
                if matches!(
                    error.kind(),
                    VMNLErrorKind::InvalidState(message) if message == expected
                )
        ));
    }

    #[test]
    fn validate_finite_accepts_finite_values() {
        assert!(validate_finite(&[-1.0, 0.0, 1.0], "values").is_ok());
    }

    #[test]
    fn validate_finite_rejects_nan_before_infinity() {
        assert_invalid_state(
            validate_finite(&[f32::INFINITY, f32::NAN], "values"),
            "values must not be NaN",
        );
    }

    #[test]
    fn validate_finite_rejects_infinity() {
        assert_invalid_state(
            validate_finite(&[f32::NEG_INFINITY], "values"),
            "values must be finite",
        );
    }

    #[test]
    fn validate_positive_finite_accepts_positive_values() {
        assert!(validate_positive_finite(&[f32::MIN_POSITIVE, 1.0], "values").is_ok());
    }

    #[test]
    fn validate_positive_finite_rejects_zero_and_negative_values() {
        assert_invalid_state(
            validate_positive_finite(&[0.0], "values"),
            "values must be strictly positive",
        );
        assert_invalid_state(
            validate_positive_finite(&[-1.0], "values"),
            "values must be strictly positive",
        );
    }
}
