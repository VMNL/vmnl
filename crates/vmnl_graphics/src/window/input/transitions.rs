// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Fixed-size transition tracking shared by keyboard and mouse snapshots.

pub(super) struct TransitionState<const N: usize> {
    down: [bool; N],
    pressed: [bool; N],
    released: [bool; N],
}

impl<const N: usize> TransitionState<N> {
    pub(super) const fn new() -> Self {
        Self {
            down: [false; N],
            pressed: [false; N],
            released: [false; N],
        }
    }

    pub(super) const fn begin_batch(&mut self) {
        self.clear_transitions();
    }

    pub(super) const fn press(&mut self, index: usize) {
        self.pressed[index] = true;
        self.down[index] = true;
    }

    pub(super) const fn repeat(&mut self, index: usize) {
        self.down[index] = true;
    }

    pub(super) const fn release(&mut self, index: usize) {
        self.released[index] = true;
        self.down[index] = false;
    }

    pub(super) const fn is_down(&self, index: usize) -> bool {
        self.down[index]
    }

    pub(super) const fn is_pressed(&self, index: usize) -> bool {
        self.pressed[index]
    }

    pub(super) const fn is_released(&self, index: usize) -> bool {
        self.released[index]
    }

    pub(super) const fn clear_transitions(&mut self) {
        self.pressed = [false; N];
        self.released = [false; N];
    }
}

#[cfg(test)]
mod tests {
    use super::TransitionState;

    #[test]
    fn retains_every_transition_in_a_batch() {
        let mut state = TransitionState::<1>::new();

        state.begin_batch();
        state.press(0);
        state.release(0);

        assert!(!state.is_down(0));
        assert!(state.is_pressed(0));
        assert!(state.is_released(0));
    }

    #[test]
    fn next_batch_clears_transitions_but_retains_down_state() {
        let mut state = TransitionState::<1>::new();
        state.press(0);

        state.begin_batch();

        assert!(state.is_down(0));
        assert!(!state.is_pressed(0));
        assert!(!state.is_released(0));
    }

    #[test]
    fn repeated_queries_do_not_consume_transitions() {
        let mut state = TransitionState::<1>::new();
        state.press(0);

        assert!(state.is_pressed(0));
        assert!(state.is_pressed(0));
    }

    #[test]
    fn explicit_clear_retains_down_state() {
        let mut state = TransitionState::<1>::new();
        state.press(0);

        state.clear_transitions();

        assert!(state.is_down(0));
        assert!(!state.is_pressed(0));
        assert!(!state.is_released(0));
    }

    #[test]
    fn repeat_keeps_down_without_creating_a_press_transition() {
        let mut state = TransitionState::<1>::new();

        state.repeat(0);

        assert!(state.is_down(0));
        assert!(!state.is_pressed(0));
    }

    #[test]
    fn release_is_observable_without_a_locally_observed_press() {
        let mut state = TransitionState::<1>::new();

        state.release(0);

        assert!(!state.is_down(0));
        assert!(state.is_released(0));
    }
}
