// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Reader ownership, bounded retention, and concurrent snapshot publication.

use crate::allocation_probe::{measure, Counts};
use crate::audio::snapshot::Snapshot;
use std::sync::{Arc, Barrier};

#[test]
fn unread_publications_coalesce_without_releasing_the_held_view() {
    let snapshot = Snapshot::new();
    let first = Arc::new(0);
    let first_weak = Arc::downgrade(&first);
    snapshot.publish(vec![first]);
    let held = snapshot.read().expect("exclusive reader");
    let mut unread = Vec::new();
    for version in 1..=64 {
        let value = Arc::new(version);
        unread.push(Arc::downgrade(&value));
        snapshot.publish(vec![value]);
        snapshot.reclaim();
        assert_eq!(*held[0], 0, "publication must not mutate the held view");
    }
    assert!(unread[..63].iter().all(|value| value.upgrade().is_none()));
    assert!(first_weak.upgrade().is_some());
    let ((), counts) = measure(|| drop(held));
    assert_eq!(counts, Counts::default());
    let (latest, counts) = measure(|| snapshot.read().expect("next reader"));
    assert_eq!(counts, Counts::default());
    assert_eq!(*latest[0], 64);
    drop(latest);
    snapshot.reclaim();
    assert!(
        first_weak.upgrade().is_none(),
        "control side reclaims the old view"
    );
    assert!(unread[63].upgrade().is_some());
    snapshot.publish(Vec::new());
    assert!(snapshot.read().expect("adopt empty view").is_empty());
    snapshot.reclaim();
    assert!(unread[63].upgrade().is_none());
}

#[test]
fn overlapping_readers_are_rejected_without_heap_work() {
    let snapshot = Snapshot::new();
    snapshot.publish(vec![Arc::new(17)]);
    let held = snapshot.read().expect("first reader");
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let (overlap, counts) = measure(|| snapshot.read());
            assert!(overlap.is_none());
            assert_eq!(counts, Counts::default());
        });
    });
    assert_eq!(*held[0], 17);
    drop(held);
    assert_eq!(*snapshot.read().expect("reader after release")[0], 17);
}

#[test]
fn concurrent_publishers_preserve_whole_views_without_reader_heap_work() {
    const ITERATIONS: usize = 8_000;
    let snapshot = Snapshot::new();
    let start = Barrier::new(3);
    std::thread::scope(|scope| {
        for publisher in 0..2 {
            let snapshot = &snapshot;
            let start = &start;
            scope.spawn(move || {
                start.wait();
                for version in 1..=ITERATIONS {
                    let value = Arc::new(publisher * ITERATIONS + version);
                    snapshot.publish(vec![value; 8]);
                    snapshot.reclaim();
                    std::thread::yield_now();
                }
            });
        }
        start.wait();
        let (consistent, counts) = measure(|| {
            let mut consistent = true;
            for _ in 0..ITERATIONS * 2 {
                let view = snapshot.read().expect("single reader");
                if let Some(first) = view.first() {
                    consistent &= view.len() == 8 && view.iter().all(|value| **value == **first);
                }
                drop(view);
                std::thread::yield_now();
            }
            consistent
        });
        assert!(consistent, "a snapshot must not contain mixed publications");
        assert_eq!(counts, Counts::default());
    });
    assert_eq!(snapshot.read().expect("last view").len(), 8);
}

#[test]
fn dropping_the_owner_releases_current_and_unread_views() {
    let snapshot = Snapshot::new();
    let current = Arc::new(1);
    let current_weak = Arc::downgrade(&current);
    snapshot.publish(vec![current]);
    assert_eq!(*snapshot.read().expect("adopt current")[0], 1);
    let unread = Arc::new(2);
    let unread_weak = Arc::downgrade(&unread);
    snapshot.publish(vec![unread]);
    assert!(current_weak.upgrade().is_some());
    assert!(unread_weak.upgrade().is_some());
    drop(snapshot);
    assert!(current_weak.upgrade().is_none());
    assert!(unread_weak.upgrade().is_none());
}
