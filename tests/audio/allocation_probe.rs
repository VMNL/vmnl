// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Thread-local heap instrumentation, including the first measured call.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Counts {
    pub allocations: usize,
    pub deallocations: usize,
    pub reallocations: usize,
}

thread_local! {
    static COUNTS: Cell<Option<Counts>> = const { Cell::new(None) };
}

fn record(update: impl FnOnce(&mut Counts)) {
    let _ = COUNTS.try_with(|cell| {
        if let Some(mut counts) = cell.get() {
            update(&mut counts);
            cell.set(Some(counts));
        }
    });
}

struct ObservedSystem;

#[global_allocator]
static ALLOCATOR: ObservedSystem = ObservedSystem;

// SAFETY: Every operation delegates to System with the caller's original
// pointer/layout. Instrumentation uses only a const-initialized Cell, never heap.
unsafe impl GlobalAlloc for ObservedSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(|counts| counts.allocations += 1);
        // SAFETY: GlobalAlloc's caller supplies the valid allocation layout.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record(|counts| counts.allocations += 1);
        // SAFETY: Forward the caller's valid layout without changing its size.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        record(|counts| counts.deallocations += 1);
        // SAFETY: The caller owns this allocation from System with this layout.
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record(|counts| counts.reallocations += 1);
        // SAFETY: The caller supplies the live allocation and valid new size.
        unsafe { System.realloc(pointer, layout, size) }
    }
}

struct Reset;

impl Drop for Reset {
    fn drop(&mut self) {
        COUNTS.set(None);
    }
}

pub(crate) fn measure<T>(operation: impl FnOnce() -> T) -> (T, Counts) {
    assert!(COUNTS.get().is_none(), "heap probes must not nest");
    COUNTS.set(Some(Counts::default()));
    let reset = Reset;
    let result = operation();
    let counts = COUNTS.get().expect("active probe");
    drop(reset);
    (result, counts)
}
