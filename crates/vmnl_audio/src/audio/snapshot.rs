// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Bounded playback snapshots with control-side destruction.

use std::cell::UnsafeCell;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const UPDATED: usize = 4;
const INDEX: usize = 3;

/// Three slots have distinct owners: writer, exchange, and reader. Publication
/// coalesces unread updates; only the writer replaces or clears vector contents.
pub(crate) struct Snapshot<T> {
    slots: [UnsafeCell<Vec<Arc<T>>>; 3],
    writer: Mutex<usize>,
    middle: AtomicUsize,
    reader: AtomicUsize,
    reading: AtomicBool,
}

// SAFETY: Writer access is serialized by writer; reader access by reading.
// Their indices and middle always form a permutation of 0, 1, 2. AcqRel swaps
// transfer exclusive slot ownership and publish all prior accesses to that slot.
// A reader never mutates its vector; T: Sync permits shared access to elements.
unsafe impl<T: Send + Sync> Sync for Snapshot<T> {}

impl<T> Snapshot<T> {
    pub(crate) fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| UnsafeCell::new(Vec::new())),
            writer: Mutex::new(0),
            middle: AtomicUsize::new(1),
            reader: AtomicUsize::new(2),
            reading: AtomicBool::new(false),
        }
    }

    /// Control side only. Unread publications may be superseded by this value.
    pub(crate) fn publish(&self, value: Vec<Arc<T>>) {
        // Ownership indices remain valid even if destruction poisons this lock:
        // the writer index is updated before any retired elements are dropped.
        let mut writer = self
            .writer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // SAFETY: The locked writer index is disjoint from reader and middle.
        unsafe { *self.slots[*writer].get() = value };
        *writer = self.middle.swap(*writer | UPDATED, Ordering::AcqRel) & INDEX;
        // SAFETY: The exchange transferred this slot to the writer, acquiring
        // completion of the reader's accesses before any elements are dropped.
        unsafe { (*self.slots[*writer].get()).clear() };
    }

    /// Control side only. Releases a reader's returned view even without a new
    /// publication; a still-unread publication must remain available to it.
    pub(crate) fn reclaim(&self) {
        let mut writer = self
            .writer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.middle.load(Ordering::Acquire) & UPDATED == 0 {
            // No publication can race under writer. With UPDATED clear, the
            // sole reader will not exchange either; ownership is transferred.
            *writer = self.middle.swap(*writer, Ordering::AcqRel) & INDEX;
            // SAFETY: This index is now exclusively writer-owned, as above.
            unsafe { (*self.slots[*writer].get()).clear() };
        }
    }

    /// One attempt, with no TLS, heap, lock, or retry loop on the reader path.
    /// Reentrant/concurrent readers are rejected without touching slot contents.
    pub(crate) fn read(&self) -> Option<SnapshotRead<'_, T>> {
        self.reading
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()?;
        let mut index = self.reader.load(Ordering::Relaxed);
        if self.middle.load(Ordering::Acquire) & UPDATED != 0 {
            index = self.middle.swap(index, Ordering::AcqRel) & INDEX;
            self.reader.store(index, Ordering::Relaxed);
        }
        Some(SnapshotRead {
            snapshot: self,
            index,
        })
    }
}

pub(crate) struct SnapshotRead<'a, T> {
    snapshot: &'a Snapshot<T>,
    index: usize,
}

impl<T> Deref for SnapshotRead<'_, T> {
    type Target = [Arc<T>];

    fn deref(&self) -> &Self::Target {
        // SAFETY: reading excludes another reader until this guard is dropped.
        // The reader-owned slot is never accessed by the writer or reclaimer.
        unsafe { &*self.snapshot.slots[self.index].get() }
    }
}

impl<T> Drop for SnapshotRead<'_, T> {
    fn drop(&mut self) {
        // Keep the slot owned by the reader between calls. A subsequent read
        // returns it through middle; its elements are never dropped here.
        self.snapshot.reading.store(false, Ordering::Release);
    }
}
