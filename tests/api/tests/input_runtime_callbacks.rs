// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Public callback lifecycle regressions without a display, device, window, or Vulkan.

#[path = "support/joystick_notifications.rs"]
mod joystick_notifications;

use joystick_notifications::NullJoystickNotifications;
use std::{
    cell::{Cell, RefCell},
    panic::{catch_unwind, AssertUnwindSafe},
    rc::Rc,
};
use vmnl::{Event, EventKind, InputRuntime, JoystickId};

fn assert_connection_pair(events: &[Event]) {
    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0].kind(),
        EventKind::JoystickConnected {
            id: JoystickId::Joystick1
        }
    ));
    assert!(matches!(
        events[1].kind(),
        EventKind::JoystickDisconnected {
            id: JoystickId::Joystick1
        }
    ));
}

#[test]
fn callback_can_unset_itself_without_losing_polled_notifications() {
    let notifications = NullJoystickNotifications::new();
    let owner = Rc::new(RefCell::new(
        InputRuntime::acquire().expect("callback owner"),
    ));
    assert!(owner.borrow_mut().poll_events().is_empty());
    let owner_callback = Rc::downgrade(&owner);
    let calls = Rc::new(Cell::new(0));
    let callback_calls = Rc::clone(&calls);
    owner.borrow_mut().set_joystick_callback(move |_, _| {
        callback_calls.set(callback_calls.get() + 1);
        owner_callback
            .upgrade()
            .expect("owner remains alive")
            .borrow_mut()
            .unset_joystick_callback();
    });

    notifications.connect();
    notifications.disconnect();

    assert_connection_pair(&owner.borrow_mut().poll_events());
    assert_eq!(calls.get(), 1, "removed callback must not run again");
}

#[test]
fn callback_can_replace_itself_for_the_next_notification() {
    let notifications = NullJoystickNotifications::new();
    let owner = Rc::new(RefCell::new(
        InputRuntime::acquire().expect("callback owner"),
    ));
    assert!(owner.borrow_mut().poll_events().is_empty());
    let owner_callback = Rc::downgrade(&owner);
    let original_calls = Rc::new(Cell::new(0));
    let callback_calls = Rc::clone(&original_calls);
    let replacement_calls = Rc::new(Cell::new(0));
    let replacement_observed = Rc::clone(&replacement_calls);
    owner.borrow_mut().set_joystick_callback(move |_, _| {
        callback_calls.set(callback_calls.get() + 1);
        let replacement_observed = Rc::clone(&replacement_observed);
        owner_callback
            .upgrade()
            .expect("owner remains alive")
            .borrow_mut()
            .set_joystick_callback(move |event, _| {
                assert!(matches!(
                    event.kind(),
                    EventKind::JoystickDisconnected {
                        id: JoystickId::Joystick1
                    }
                ));
                replacement_observed.set(replacement_observed.get() + 1);
            });
    });

    notifications.connect();
    notifications.disconnect();

    assert_connection_pair(&owner.borrow_mut().poll_events());
    assert_eq!(
        (original_calls.get(), replacement_calls.get()),
        (1, 1),
        "the replacement must handle the next notification"
    );
}

#[test]
fn last_callback_can_unsubscribe_itself_without_a_deferred_internal_panic() {
    let notifications = NullJoystickNotifications::new();
    let owner = Rc::new(RefCell::new(
        InputRuntime::acquire().expect("callback owner"),
    ));
    let owner_callback = Rc::downgrade(&owner);
    let completed = Rc::new(Cell::new(false));
    let callback_completed = Rc::clone(&completed);
    // Do not poll this handle: removing its callback must remove the last subscription.
    owner.borrow_mut().set_joystick_callback(move |_, _| {
        owner_callback
            .upgrade()
            .expect("owner remains alive")
            .borrow_mut()
            .unset_joystick_callback();
        callback_completed.set(true);
    });

    notifications.disconnect();
    let sample = catch_unwind(AssertUnwindSafe(|| {
        owner.borrow().sample_joystick(JoystickId::Joystick1)
    }));

    assert!(
        sample.is_ok(),
        "unsubscribing inside the callback must not defer an internal panic"
    );
    assert!(sample
        .expect("no deferred panic")
        .expect("Null slot query should succeed")
        .is_none());
    assert!(completed.get(), "unset must return to the user's callback");
    notifications.assert_unregistered();
}

#[test]
fn last_callback_can_unsubscribe_and_resubscribe_during_its_invocation() {
    let notifications = NullJoystickNotifications::new();
    let owner = Rc::new(RefCell::new(
        InputRuntime::acquire().expect("callback owner"),
    ));
    let owner_callback = Rc::downgrade(&owner);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let callback_calls = Rc::clone(&calls);
    owner.borrow_mut().set_joystick_callback(move |_, _| {
        callback_calls.borrow_mut().push("original");
        let owner = owner_callback.upgrade().expect("owner remains alive");
        let mut owner = owner.borrow_mut();
        owner.unset_joystick_callback();
        let callback_calls = Rc::clone(&callback_calls);
        owner.set_joystick_callback(move |_, _| {
            callback_calls.borrow_mut().push("replacement");
        });
    });

    notifications.connect();
    notifications.disconnect();

    let events = owner.borrow_mut().poll_events();
    assert_eq!(&*calls.borrow(), &["original", "replacement"]);
    assert_eq!(
        events.len(),
        1,
        "resubscription must not replay the first event"
    );
    assert!(matches!(
        events[0].kind(),
        EventKind::JoystickDisconnected {
            id: JoystickId::Joystick1
        }
    ));
}

#[test]
fn last_callback_unsubscription_is_cleaned_up_before_resuming_a_user_panic() {
    let notifications = NullJoystickNotifications::new();
    let owner = Rc::new(RefCell::new(
        InputRuntime::acquire().expect("callback owner"),
    ));
    let owner_callback = Rc::downgrade(&owner);
    let completed = Rc::new(Cell::new(false));
    let callback_completed = Rc::clone(&completed);
    owner.borrow_mut().set_joystick_callback(move |_, _| {
        owner_callback
            .upgrade()
            .expect("owner remains alive")
            .borrow_mut()
            .unset_joystick_callback();
        callback_completed.set(true);
        panic!("user callback panic after unset");
    });

    notifications.disconnect();
    let panic = catch_unwind(AssertUnwindSafe(|| {
        owner.borrow().sample_joystick(JoystickId::Joystick1)
    }))
    .expect_err("the user's panic must resume after returning from GLFW");

    assert!(completed.get(), "unset must return before the user's panic");
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"user callback panic after unset")
    );
    notifications.assert_unregistered();
    assert!(owner.borrow_mut().poll_events().is_empty());
}
