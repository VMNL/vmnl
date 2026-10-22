// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Fixed public stick observations; native axis injection stays in the driver.

use super::{all_snapshots, invalid, require};
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use vmnl::{
    JoystickState, StickAngleConvention as Angle, StickConfig, StickState, VMNLResult, Window,
};

#[derive(Clone, Copy, Debug)]
pub(super) enum Case {
    Builder,
    Left,
    Right,
    Both,
    Boundary,
    Diagonal,
    Reset,
}

impl Case {
    pub(super) fn parse(value: &str) -> VMNLResult<Self> {
        match value {
            "builder" => Ok(Self::Builder),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "both" => Ok(Self::Both),
            "boundary" => Ok(Self::Boundary),
            "diagonal" => Ok(Self::Diagonal),
            "reset" => Ok(Self::Reset),
            _ => Err(invalid("unknown stick observation")),
        }
    }

    pub(super) const fn axes(self) -> [f32; 6] {
        match self {
            Self::Builder | Self::Left | Self::Right | Self::Both => {
                [0.5, 0.0, 0.0, 0.75, -1.0, 1.0]
            }
            Self::Boundary => [0.0, -0.25, -0.5, 0.0, -1.0, 1.0],
            Self::Diagonal => [0.75, -1.0, 0.3, 0.4, -1.0, 1.0],
            Self::Reset => [0.0; 6],
        }
    }

    pub(super) fn configure(self, windows: &mut [Window]) -> VMNLResult<()> {
        require(
            windows.len() == 4,
            "stick observations require four windows",
        )?;
        let before = all_snapshots(windows);
        match self {
            Self::Left => windows[0].set_left_stick_config(StickConfig::new(0.0, Angle::Heading)?),
            Self::Right => {
                windows[0].set_right_stick_config(StickConfig::new(0.75, Angle::Math2D)?)
            }
            Self::Both => windows[0].set_stick_configs(
                StickConfig::new(0.25, Angle::Math2D)?,
                StickConfig::new(0.0, Angle::Heading)?,
            ),
            Self::Reset => {
                for window in &mut windows[..2] {
                    window.set_stick_configs(StickConfig::default(), StickConfig::default());
                }
            }
            _ => {}
        }
        for (index, (old, new)) in before.iter().zip(all_snapshots(windows)).enumerate() {
            require(
                old.iter().zip(&new).all(|(old, new)| {
                    old.status == new.status
                        && old.sample == new.sample
                        && old.buttons == new.buttons
                }),
                "stick setter changed raw samples, presence or button transitions",
            )?;
            let changed = match self {
                Self::Left | Self::Right | Self::Both => index == 0,
                Self::Reset => index < 2,
                _ => false,
            };
            require(
                changed || *old == new,
                "stick setter changed another window",
            )?;
        }
        Ok(())
    }

    // Cardinal angles and worked 3-4-5 triangles are independent of VMNL's angle implementation.
    fn expected(self, index: usize) -> [Expected; 2] {
        let center = Expected(0.0, 0.0, 0.0, None);
        let half_right = |angle| Expected(0.5, 0.0, 0.5, Some(angle));
        let down = |angle| Expected(0.0, 0.75, 0.75, Some(angle));
        match (self, index) {
            (Self::Builder, 0) => [center, down(FRAC_PI_2)],
            (Self::Left, 0) => [half_right(FRAC_PI_2), down(FRAC_PI_2)],
            (Self::Right, 0) => [half_right(FRAC_PI_2), center],
            (Self::Both, 0) => [half_right(0.0), down(PI)],
            (Self::Builder | Self::Left | Self::Right | Self::Both, 1) => {
                [half_right(FRAC_PI_2), center]
            }
            (Self::Builder | Self::Left | Self::Right | Self::Both, 2) => {
                [half_right(0.0), down(3.0 * FRAC_PI_2)]
            }
            (Self::Boundary, 0) => [center, Expected(-0.5, 0.0, 0.5, Some(3.0 * FRAC_PI_2))],
            (Self::Boundary, 1) => [Expected(0.0, -0.25, 0.25, Some(0.0)), center],
            (Self::Boundary, 2) => [
                Expected(0.0, -0.25, 0.25, Some(FRAC_PI_2)),
                Expected(-0.5, 0.0, 0.5, Some(PI)),
            ],
            (Self::Diagonal, 0) => [
                Expected(0.75, -1.0, 1.25, Some(0.927_295_2)),
                Expected(0.3, 0.4, 0.5, Some(2.498_091_5)),
            ],
            (Self::Diagonal, 1) => [Expected(0.75, -1.0, 1.25, Some(0.643_501_1)), center],
            (Self::Diagonal, 2) => [
                Expected(0.75, -1.0, 1.25, Some(0.927_295_2)),
                Expected(0.3, 0.4, 0.5, Some(5.355_890_3)),
            ],
            _ => [center; 2],
        }
    }
}

pub(super) fn builder_configs(index: usize) -> VMNLResult<(StickConfig, StickConfig)> {
    match index {
        0 => Ok((
            StickConfig::new(0.5, Angle::Math2D)?,
            StickConfig::new(0.5, Angle::Screen2D)?,
        )),
        1 => Ok((
            StickConfig::new(0.0, Angle::Heading)?,
            StickConfig::new(0.75, Angle::Math2D)?,
        )),
        _ => Ok((StickConfig::default(), StickConfig::default())),
    }
}

pub(super) fn check(
    case: Option<Case>,
    index: usize,
    state: &JoystickState,
    mapped: bool,
) -> VMNLResult<()> {
    let actual = [state.left_stick(), state.right_stick()];
    if !mapped {
        return require(
            actual == [None; 2],
            "unmapped/untracked/absent slot retained processed sticks",
        );
    }
    require(
        actual.iter().all(Option::is_some),
        "mapped slot has no processed sticks",
    )?;
    if let Some(case) = case {
        require(
            case.expected(index).into_iter().zip(actual).all(|(expected, actual)| expected.matches(actual)),
            &format!("wrong processed sticks after polling: case={case:?}, window={index}, actual={actual:?}"),
        )?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Expected(f32, f32, f32, Option<f32>);

impl Expected {
    fn matches(self, actual: Option<StickState>) -> bool {
        let Some(actual) = actual else {
            return false;
        };
        let near =
            |actual: f32, expected: f32| actual.is_finite() && (actual - expected).abs() <= 0.0001;
        near(actual.x(), self.0)
            && near(actual.y(), self.1)
            && near(actual.magnitude(), self.2)
            && match (actual.angle(), self.3) {
                (None, None) => true,
                (Some(actual), Some(expected)) => {
                    (0.0..TAU).contains(&actual) && near(actual, expected)
                }
                _ => false,
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stick_oracle_rejects_wrong_zone_convention_rescaling_and_missing_values() -> VMNLResult<()> {
        let [left, right] = Case::Both.expected(0);
        assert!(left.matches(Some(
            StickConfig::new(0.25, Angle::Math2D)?.process(0.5, 0.0)
        )));
        assert!(right.matches(Some(
            StickConfig::new(0.0, Angle::Heading)?.process(0.0, 0.75)
        )));
        assert!(!left.matches(Some(
            StickConfig::new(0.5, Angle::Math2D)?.process(0.5, 0.0)
        )));
        assert!(!left.matches(Some(
            StickConfig::new(0.0, Angle::Heading)?.process(0.5, 0.0)
        )));
        assert!(!left.matches(Some(StickConfig::default().process(1.0, 0.0))));
        assert!(!left.matches(None));
        assert!(!left.matches(Some(StickConfig::default().process(f32::NAN, 0.0))));
        Ok(())
    }

    #[test]
    fn stick_oracle_covers_builder_boundaries_and_diagonal_worked_values() -> VMNLResult<()> {
        for (case, left, right) in [
            (Case::Builder, (0.5, Angle::Math2D), (0.5, Angle::Screen2D)),
            (Case::Left, (0.0, Angle::Heading), (0.5, Angle::Screen2D)),
            (Case::Right, (0.0, Angle::Heading), (0.75, Angle::Math2D)),
            (Case::Both, (0.25, Angle::Math2D), (0.0, Angle::Heading)),
            (Case::Boundary, (0.25, Angle::Math2D), (0.0, Angle::Heading)),
            (Case::Diagonal, (0.25, Angle::Math2D), (0.0, Angle::Heading)),
            (Case::Reset, (0.0, Angle::Math2D), (0.0, Angle::Math2D)),
        ] {
            let axes = case.axes();
            for index in 0..3 {
                let (left, right) = if index == 0 {
                    (
                        StickConfig::new(left.0, left.1)?,
                        StickConfig::new(right.0, right.1)?,
                    )
                } else if matches!(case, Case::Reset) {
                    (StickConfig::default(), StickConfig::default())
                } else {
                    builder_configs(index)?
                };
                for (expected, actual) in case.expected(index).into_iter().zip([
                    left.process(axes[0], axes[1]),
                    right.process(axes[2], axes[3]),
                ]) {
                    assert!(
                        expected.matches(Some(actual)),
                        "case={case:?}, window={index}"
                    );
                }
            }
        }
        Ok(())
    }
}
