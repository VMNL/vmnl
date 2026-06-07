////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Nathan Flachat
/// SPDX-License-Identifier: MIT
///
////////////////////////////////////////////////////////////////////////////////


use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BusKind {
    Master,
    Music,
    Sfx,
}

#[derive(Debug)]
struct AudioBusState {
    volume_bits: AtomicU32,
    muted: AtomicBool,
    paused: AtomicBool,
}

#[derive(Clone, Debug)]
pub struct AudioBus {
    kind: BusKind,
    state: Arc<AudioBusState>,
}

impl AudioBus {
    pub fn new(kind: BusKind) -> Self {
        Self {
            kind,
            state: Arc::new(AudioBusState {
                volume_bits: AtomicU32::new(1.0f32.to_bits()),
                muted: AtomicBool::new(false),
                paused: AtomicBool::new(false),
            }),
        }
    }

    pub fn kind(&self) -> BusKind {
        self.kind
    }

    pub fn set_volume(&self, volume: f32) {
        self.state
            .volume_bits
            .store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn volume(&self) -> f32 {
        f32::from_bits(self.state.volume_bits.load(Ordering::Relaxed))
    }

    pub fn mute(&self) {
        self.state.muted.store(true, Ordering::Relaxed);
    }

    pub fn unmute(&self) {
        self.state.muted.store(false, Ordering::Relaxed);
    }

    pub fn is_muted(&self) -> bool {
        self.state.muted.load(Ordering::Relaxed)
    }

    pub fn pause(&self) {
        self.state.paused.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.state.paused.store(false, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.state.paused.load(Ordering::Relaxed)
    }

    pub fn gain(&self) -> f32 {
        if self.is_muted() || self.is_paused() {
            0.0
        } else {
            self.volume()
        }
    }
}
