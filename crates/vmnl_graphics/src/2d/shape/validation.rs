// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Shared numeric validation for built-in 2D shape parameters.

use super::Vertex2D;
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

pub(super) fn validate_finite_positions(vertices: &[Vertex2D], subject: &str) -> VMNLResult<()> {
    if vertices
        .iter()
        .any(|vertex| vertex.position.x.is_nan() || vertex.position.y.is_nan())
    {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "{subject} vertex positions must not be NaN"
        ))));
    }

    if vertices
        .iter()
        .any(|vertex| vertex.position.x.is_infinite() || vertex.position.y.is_infinite())
    {
        return Err(VMNLError::new(VMNLErrorKind::InvalidState(format!(
            "{subject} vertex positions must be finite"
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
    use crate::{
        common::Rgba,
        d2::{Vector2f, Vertex2D},
    };

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

    fn vertex(x: f32, y: f32) -> Vertex2D {
        Vertex2D {
            position: Vector2f { x, y },
            color: Rgba::WHITE,
        }
    }

    #[test]
    fn validate_finite_positions_accepts_finite_extremes_negative_coordinates_and_signed_zero() {
        assert!(validate_finite_positions(
            &[
                vertex(f32::MAX, f32::MIN),
                vertex(-0.0, 0.0),
                vertex(-17.0, 23.0),
            ],
            "indexed shape",
        )
        .is_ok());
    }

    #[test]
    fn validate_finite_positions_rejects_non_finite_values_in_each_component() {
        for (x, y, expected) in [
            (
                f32::NAN,
                0.0,
                "indexed shape vertex positions must not be NaN",
            ),
            (
                0.0,
                f32::NAN,
                "indexed shape vertex positions must not be NaN",
            ),
            (
                f32::INFINITY,
                0.0,
                "indexed shape vertex positions must be finite",
            ),
            (
                0.0,
                f32::INFINITY,
                "indexed shape vertex positions must be finite",
            ),
            (
                f32::NEG_INFINITY,
                0.0,
                "indexed shape vertex positions must be finite",
            ),
            (
                0.0,
                f32::NEG_INFINITY,
                "indexed shape vertex positions must be finite",
            ),
        ] {
            assert_invalid_state(
                validate_finite_positions(&[vertex(x, y)], "indexed shape"),
                expected,
            );
        }
    }

    #[test]
    fn validate_finite_positions_rejects_nan_before_an_earlier_infinity() {
        assert_invalid_state(
            validate_finite_positions(
                &[vertex(f32::INFINITY, 0.0), vertex(1.0, f32::NAN)],
                "indexed shape",
            ),
            "indexed shape vertex positions must not be NaN",
        );
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
