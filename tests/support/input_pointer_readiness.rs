// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Bounded native pointer preparation across asynchronous window reconfiguration.

use std::time::Duration;

pub(crate) const POINTER_SETTLE_TIME: Duration = Duration::from_millis(100);

#[derive(Clone, Copy, Debug)]
pub(crate) struct PointerSnapshot {
    pub geometry: (i16, i16, u16, u16),
    pub focused: bool,
    pub hovered: bool,
    pub position: (i16, i16),
}

impl PointerSnapshot {
    fn target(self) -> Result<(i16, i16), String> {
        let (x, y, width, height) = self.geometry;
        if width == 0 || height == 0 {
            return Err("pointer preparation requires a nonempty window".into());
        }
        // Use a fresh interior point, distinct from the client's former (120, 100) warp.
        let offset_x = i16::try_from(width / 3).map_err(|e| e.to_string())?;
        let offset_y = i16::try_from(height / 3).map_err(|e| e.to_string())?;
        Ok((
            x.checked_add(offset_x).ok_or("pointer target X overflow")?,
            y.checked_add(offset_y).ok_or("pointer target Y overflow")?,
        ))
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum PointerPreparation {
    Wait,
    Position(i16, i16),
    Ready,
}

pub(crate) struct PointerReadiness {
    timeout: Duration,
    geometry: Option<(i16, i16, u16, u16)>,
    positioned: bool,
    stable_since: Option<Duration>,
}

impl PointerReadiness {
    pub(crate) const fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            geometry: None,
            positioned: false,
            stable_since: None,
        }
    }

    pub(crate) fn observe(
        &mut self,
        elapsed: Duration,
        snapshot: PointerSnapshot,
    ) -> Result<PointerPreparation, String> {
        if elapsed >= self.timeout {
            return Err(format!(
                "native pointer preparation timed out: {snapshot:?}"
            ));
        }
        let changed = self.geometry.replace(snapshot.geometry) != Some(snapshot.geometry);
        let (x, y) = snapshot.target()?;
        if !snapshot.focused {
            self.positioned = false;
            self.stable_since = None;
            return Ok(PointerPreparation::Wait);
        }
        if changed || !self.positioned || !snapshot.hovered || snapshot.position != (x, y) {
            self.positioned = true;
            self.stable_since = None;
            return Ok(PointerPreparation::Position(x, y));
        }
        let since = self.stable_since.get_or_insert(elapsed);
        Ok(if elapsed.saturating_sub(*since) >= POINTER_SETTLE_TIME {
            PointerPreparation::Ready
        } else {
            PointerPreparation::Wait
        })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn snapshot() -> PointerSnapshot {
        PointerSnapshot {
            geometry: (15, 66, 1416, 1719),
            focused: true,
            hovered: true,
            position: (487, 639),
        }
    }

    #[test]
    fn reconfiguration_after_positioning_requires_new_injection_and_full_settling() {
        let mut readiness = PointerReadiness::new(Duration::from_secs(5));
        let mut state = snapshot();
        let observe = |readiness: &mut PointerReadiness, ms, state| {
            readiness
                .observe(Duration::from_millis(ms), state)
                .expect("valid sample")
        };
        assert_eq!(
            observe(&mut readiness, 0, state),
            PointerPreparation::Position(487, 639)
        );
        assert_eq!(observe(&mut readiness, 1, state), PointerPreparation::Wait);
        state.geometry = (1449, 66, 1416, 851);
        state.hovered = false;
        assert_eq!(
            observe(&mut readiness, 3, state),
            PointerPreparation::Position(1921, 349)
        );
        state.geometry = snapshot().geometry;
        state.position = (2296, 620);
        assert_eq!(
            observe(&mut readiness, 10, state),
            PointerPreparation::Position(487, 639)
        );
        assert_eq!(
            observe(&mut readiness, 200, state),
            PointerPreparation::Position(487, 639)
        );
        state = snapshot();
        assert_eq!(
            observe(&mut readiness, 201, state),
            PointerPreparation::Wait
        );
        assert_eq!(
            observe(&mut readiness, 300, state),
            PointerPreparation::Wait
        );
        assert_eq!(
            observe(&mut readiness, 301, state),
            PointerPreparation::Ready
        );
    }

    #[test]
    fn lost_focus_hover_or_geometry_restarts_stability() {
        for change in 0..3 {
            let mut readiness = PointerReadiness::new(Duration::from_secs(5));
            let mut state = snapshot();
            readiness.observe(Duration::ZERO, state).expect("position");
            readiness
                .observe(Duration::from_millis(1), state)
                .expect("settle");
            match change {
                0 => state.focused = false,
                1 => state.hovered = false,
                _ => state.geometry.2 += 1,
            }
            assert_ne!(
                readiness
                    .observe(Duration::from_millis(100), state)
                    .expect("changed"),
                PointerPreparation::Ready
            );
            assert_ne!(
                readiness
                    .observe(Duration::from_millis(101), snapshot())
                    .expect("restored"),
                PointerPreparation::Ready
            );
        }
    }

    #[test]
    fn missing_motion_never_qualifies_and_deadline_is_fixed() {
        let mut readiness = PointerReadiness::new(Duration::from_millis(200));
        let mut state = snapshot();
        state.position = (135, 166);
        for ms in [0, 100, 199] {
            assert_eq!(
                readiness
                    .observe(Duration::from_millis(ms), state)
                    .expect("position missing"),
                PointerPreparation::Position(487, 639)
            );
        }
        assert!(readiness
            .observe(Duration::from_millis(200), snapshot())
            .is_err());
        assert!(readiness
            .observe(Duration::from_millis(201), snapshot())
            .is_err());
    }

    #[test]
    fn targets_reject_empty_windows_and_coordinate_overflow() {
        let mut state = snapshot();
        state.geometry.2 = 0;
        assert!(state.target().is_err());
        state.geometry = (i16::MAX, 0, 3, 3);
        assert!(state.target().is_err());
        state.geometry = (0, i16::MAX, 3, 3);
        assert!(state.target().is_err());
    }
}
