// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public bus behaviour. Creating a bus through a device requires a real CPAL backend.

mod common_imports {
    pub use crate::common::{device_or_skip, lock_backend_tests};
}

use common_imports::{device_or_skip, lock_backend_tests};

#[test]
fn bus_clones_share_volume_mute_pause_and_effective_gain() {
    let _guard = lock_backend_tests();
    let Some(device) = device_or_skip(true) else {
        return;
    };

    let bus = device.sfx_bus();
    let clone = bus.clone();

    bus.set_volume(0.4).expect("finite gain in range should be accepted");
    assert_eq!(clone.volume(), 0.4);
    assert_eq!(clone.gain(), 0.4);

    assert!(clone.set_volume(f32::NAN).is_err());
    assert_eq!(bus.volume(), 0.4, "rejected gain must not replace stored gain");

    clone.mute();
    assert!(bus.is_muted());
    assert_eq!(bus.volume(), 0.4, "mute must not change configured volume");
    assert_eq!(bus.gain(), 0.0, "muted bus has zero effective gain");

    bus.unmute();
    assert!(!clone.is_muted());
    assert_eq!(bus.gain(), 0.4);

    bus.pause();
    assert!(clone.is_paused());
    assert_eq!(clone.gain(), 0.0, "paused bus has zero effective gain");

    clone.resume();
    assert!(!bus.is_paused());
    assert_eq!(bus.gain(), 0.4);
}
