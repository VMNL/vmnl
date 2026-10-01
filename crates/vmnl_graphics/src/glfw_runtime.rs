// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Shared ownership and initialization configuration for GLFW.

use crate::{VMNLError, VMNLErrorKind, VMNLResult};
use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    ffi::CString,
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    rc::{Rc, Weak},
    sync::Mutex,
    thread::{self, ThreadId},
};

/// Reserves process-wide GLFW lifecycle transitions and records its owning thread. The lock is
/// released before calling GLFW and does not guard ordinary GLFW calls, callbacks, or GPU state.
static GLFW_INIT_LOCK: Mutex<RuntimeState> = Mutex::new(RuntimeState::Inactive);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeState {
    Inactive,
    Initializing(ThreadId),
    Active(ThreadId),
}

thread_local! {
    /// GLFW is !Send; this weak registry lets handles on its owning thread share one Rc owner.
    static THREAD_GLFW_RUNTIME: RefCell<Weak<GlfwRuntime>> = const { RefCell::new(Weak::new()) };
}

/// Resolved process-wide initialization settings for the active GLFW runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputRuntimeConfig {
    hat_buttons: bool,
}

impl InputRuntimeConfig {
    /// Returns whether GLFW also exposes each joystick hat direction in its button array.
    ///
    /// Hats remain available through GLFW's separate hat API either way.
    #[inline]
    #[must_use]
    pub const fn hat_buttons(self) -> bool {
        self.hat_buttons
    }
}

impl Default for InputRuntimeConfig {
    fn default() -> Self {
        Self { hat_buttons: true }
    }
}

/// A handle to VMNL's process-wide GLFW runtime, available without creating a Vulkan context.
///
/// Acquiring this handle initializes GLFW on the calling thread if no VMNL handle is active.
/// Otherwise it shares the current runtime and its resolved initialization configuration. The
/// first active runtime uses GLFW's default settings unless an explicit builder setting is given.
/// An active runtime remains alive while any `InputRuntime`, `Context`, `Window`, or native cursor
/// owns it.
///
/// This handle does not create a window or Vulkan resources. GLFW still requires a usable backend
/// for initialization. It is intentionally not `Clone`; each `acquire` returns a distinct handle.
/// The handle is neither `Send` nor `Sync` and must be acquired and dropped on GLFW's platform
/// thread.
pub struct InputRuntime {
    runtime: Rc<GlfwRuntime>,
    joystick_events: Rc<RefCell<JoystickEventQueue>>,
    joystick_subscription: Option<JoystickEventSubscription>,
    has_polled_joystick_events: bool,
}

impl InputRuntime {
    /// Acquires the active GLFW runtime or initializes it with GLFW defaults.
    ///
    /// If a runtime is active, this handle adopts its configuration. GLFW initialization remains
    /// lazy: this call does not create a Vulkan instance or window.
    ///
    /// # Errors
    /// Returns `GlfwInitFailed` if GLFW cannot initialize, or `InvalidState` if another thread
    /// currently owns VMNL's active GLFW runtime.
    pub fn acquire() -> VMNLResult<Self> {
        Ok(Self {
            runtime: GlfwRuntime::acquire(None)?,
            joystick_events: Rc::default(),
            joystick_subscription: None,
            has_polled_joystick_events: false,
        })
    }

    /// Creates a builder for choosing GLFW initialization settings on first acquisition.
    #[must_use]
    pub const fn builder() -> InputRuntimeBuilder {
        InputRuntimeBuilder { hat_buttons: None }
    }

    /// Returns the settings resolved when the active GLFW runtime was initialized.
    #[inline]
    #[must_use]
    pub fn configuration(&self) -> InputRuntimeConfig {
        self.runtime.configuration
    }

    /// Samples one joystick slot, copying all GLFW-owned data into a standalone value.
    ///
    /// `Ok(None)` means the slot is absent. A present joystick may have no gamepad mapping;
    /// inspect [`crate::JoystickSample::gamepad`] for mapped state. This sample has no transition
    /// history and does not update any window snapshot. GLFW provides fields through successive
    /// queries, so a device change during sampling is not an atomic snapshot.
    ///
    /// GLFW joystick calls must run on the main platform thread. Sampling may allocate to own the
    /// variable-length raw arrays and device strings. The first joystick query lazily initializes
    /// GLFW's joystick subsystem; its latency is unspecified.
    ///
    /// # Errors
    /// Returns a GLFW operation error if the backend reports one or violates the documented
    /// pointer/count contract.
    pub fn sample_joystick(
        &self,
        id: crate::JoystickId,
    ) -> VMNLResult<Option<crate::JoystickSample>> {
        self.runtime.sample_joystick(id)
    }

    /// Processes pending GLFW events and returns joystick connection changes observed by this
    /// runtime handle.
    ///
    /// This handle subscribes on its first call unless `set_joystick_callback` subscribed it
    /// earlier. Changes reported before subscription are not replayed. Every independently
    /// acquired `InputRuntime` has its own queue; polling this handle does not drain another
    /// handle's events or a window's event queue. A call to
    /// [`crate::Window::poll_events`] may also invoke the GLFW callback without consuming this
    /// handle's queue.
    ///
    /// GLFW event processing must run on its main platform thread and must not be called from a
    /// GLFW callback. Panics in the native joystick callback are caught at the C boundary and
    /// resumed after GLFW returns to Rust. Every subscriber still receives the queued event;
    /// further user callbacks are skipped for the remainder of that GLFW call.
    #[must_use]
    pub fn poll_events(&mut self) -> Vec<crate::Event> {
        self.has_polled_joystick_events = true;
        self.ensure_joystick_subscription();

        let mut glfw = self.runtime.glfw.clone();
        glfw.poll_events();
        self.runtime.resume_joystick_callback_panic();
        self.joystick_events.borrow_mut().drain_events().collect()
    }

    /// Installs a synchronous callback for joystick connection changes.
    ///
    /// The callback runs on GLFW's event-processing thread while the native joystick callback is
    /// active. The supplied pointer is GLFW's per-slot user pointer; it is still raw and does not
    /// carry ownership. A disconnect callback can inspect it before GLFW clears the slot. Do not
    /// retain or dereference the pointer unless the caller-provided allocation's lifetime and Rust
    /// aliasing rules permit it. Callback code must not recursively process GLFW events. VMNL
    /// catches a panic at the C boundary and resumes it after the enclosing GLFW call returns. The
    /// event is queued for all subscribers before the panic resumes, and further user callbacks
    /// are skipped for the remainder of that GLFW call.
    pub fn set_joystick_callback(
        &mut self,
        callback: impl FnMut(&crate::Event, *mut std::ffi::c_void) + 'static,
    ) {
        self.joystick_events.borrow_mut().callback = Some(Box::new(callback));
        self.ensure_joystick_subscription();
    }

    /// Removes this handle's synchronous joystick callback.
    pub fn unset_joystick_callback(&mut self) {
        self.joystick_events.borrow_mut().callback = None;
        if !self.has_polled_joystick_events {
            self.joystick_subscription = None;
            self.joystick_events.borrow_mut().events.clear();
        }
    }

    fn ensure_joystick_subscription(&mut self) {
        if self.joystick_subscription.is_none() {
            self.joystick_subscription = Some(
                self.runtime
                    .subscribe_joystick_events(Rc::clone(&self.joystick_events)),
            );
        }
    }

    /// Adds or replaces SDL gamepad mapping database entries in GLFW.
    ///
    /// GLFW accepts ASCII mapping lines separated by newlines. Replacing a GUID updates the
    /// existing entry. The mapping database belongs to the shared GLFW runtime; windows sample
    /// the new mapping on their next input poll.
    ///
    /// # Errors
    /// Returns an error for non-ASCII or NUL-containing input, a GLFW error callback during
    /// parsing, or a failed GLFW return value. GLFW 3.4 may report an invalid mapping through its
    /// error callback while still returning `true`; VMNL treats either signal as failure.
    pub fn update_gamepad_mappings(&self, mappings: &str) -> VMNLResult<()> {
        if !mappings.is_ascii() {
            return Err(VMNLError::new(VMNLErrorKind::InvalidGamepadMapping(
                "mapping text must be ASCII".to_owned(),
            )));
        }
        let mappings = CString::new(mappings).map_err(|_| {
            VMNLError::new(VMNLErrorKind::InvalidGamepadMapping(
                "mapping text contains an interior NUL byte".to_owned(),
            ))
        })?;

        let (updated, errors) = self
            .runtime
            .capture_glfw_errors(|| crate::glfw_backend::update_gamepad_mappings(&mappings));
        if !errors.is_empty() {
            return Err(glfw_input_error("gamepad mapping update", errors));
        }
        if !updated {
            let (error, description) = glfw::get_error_string();
            let message = if description.is_empty() {
                format!("GLFW returned false without an error callback ({error:?})")
            } else {
                format!("{error:?}: {description}")
            };
            return Err(glfw_input_error("gamepad mapping update", vec![message]));
        }
        Ok(())
    }

    /// Returns GLFW's process-wide user pointer for a joystick slot.
    ///
    /// The pointer is retained by GLFW only while that slot's device remains connected. This
    /// accessor does not establish ownership or keep the pointed-to allocation alive.
    ///
    /// # Safety
    /// The caller must ensure that any dereference is valid, aligned, initialized, and obeys Rust's
    /// aliasing rules. The pointer may become invalid as soon as the device disconnects or GLFW
    /// terminates. Do not retain it past either boundary.
    #[must_use]
    pub unsafe fn joystick_user_pointer(&self, id: crate::JoystickId) -> *mut std::ffi::c_void {
        crate::glfw_backend::get_joystick_user_pointer(id)
    }

    /// Sets GLFW's process-wide user pointer for a joystick slot.
    ///
    /// GLFW does not own or synchronize the pointed-to value. It clears the pointer when the
    /// device disconnects.
    ///
    /// # Safety
    /// The caller owns the allocation and must keep it valid for every access through this pointer
    /// until the slot disconnects or the pointer is replaced. Concurrent access must be
    /// synchronized by the caller, and no Rust reference may outlive the allocation.
    pub unsafe fn set_joystick_user_pointer(
        &self,
        id: crate::JoystickId,
        pointer: *mut std::ffi::c_void,
    ) {
        crate::glfw_backend::set_joystick_user_pointer(id, pointer);
    }
}

/// Builder for acquiring the shared GLFW runtime with an explicit first-init setting.
#[derive(Clone, Copy, Debug, Default)]
pub struct InputRuntimeBuilder {
    hat_buttons: Option<bool>,
}

impl InputRuntimeBuilder {
    /// Chooses whether GLFW also exposes hat directions in each joystick's button array.
    ///
    /// The default is `true`, matching GLFW 3.4. GLFW's separate hat query remains available
    /// regardless of this setting. This hint applies only when the first VMNL handle initializes
    /// GLFW; a conflicting explicit setting while the runtime is active returns a structured
    /// [`VMNLErrorKind::GlfwInitializationConfigConflict`].
    #[inline]
    #[must_use]
    pub const fn hat_buttons(mut self, enabled: bool) -> Self {
        self.hat_buttons = Some(enabled);
        self
    }

    /// Acquires the shared runtime, using GLFW defaults unless a setting was supplied.
    ///
    /// # Errors
    /// Returns `GlfwInitFailed` if GLFW cannot initialize, a configuration conflict if this
    /// explicit setting differs from the active runtime, or `InvalidState` if another thread
    /// currently owns VMNL's active GLFW runtime.
    pub fn build(self) -> VMNLResult<InputRuntime> {
        let requested = self
            .hat_buttons
            .map(|hat_buttons| InputRuntimeConfig { hat_buttons });
        Ok(InputRuntime {
            runtime: GlfwRuntime::acquire(requested)?,
            joystick_events: Rc::default(),
            joystick_subscription: None,
            has_polled_joystick_events: false,
        })
    }
}

/// Internal owner shared by `InputRuntime`, `VMNLInstance`, and their windows.
pub(crate) struct GlfwRuntime {
    // Fields drop in declaration order. The lifecycle guard must stay after this token so the
    // active-thread marker remains set until the last GLFW token has terminated GLFW.
    glfw: glfw::Glfw,
    pub(crate) configuration: InputRuntimeConfig,
    callbacks: RefCell<GlfwCallbacks>,
    joystick_events: RefCell<JoystickEventDispatcher>,
    joystick_callback_panic: RefCell<Option<Box<dyn Any + Send>>>,
    _lifecycle_guard: RuntimeLifecycleGuard,
}

#[derive(Default)]
struct GlfwCallbacks {
    error_callback: Option<Box<dyn FnMut(VMNLErrorKind, String)>>,
    error_captures: Vec<Vec<String>>,
}

impl std::fmt::Debug for GlfwRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GlfwRuntime")
            .field("configuration", &self.configuration)
            .finish_non_exhaustive()
    }
}

impl GlfwRuntime {
    /// Acquires the current thread's shared runtime, honoring an optional explicit configuration.
    pub(crate) fn acquire(requested: Option<InputRuntimeConfig>) -> VMNLResult<Rc<Self>> {
        if let Some(runtime) = THREAD_GLFW_RUNTIME.with(|slot| slot.borrow().upgrade()) {
            check_configuration(runtime.configuration, requested)?;
            return Ok(runtime);
        }

        let current_thread = thread::current().id();
        let mut state = GLFW_INIT_LOCK.lock().map_err(|_| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "GLFW initialization lock is poisoned".into(),
            ))
        })?;
        match *state {
            RuntimeState::Inactive => *state = RuntimeState::Initializing(current_thread),
            RuntimeState::Initializing(owner) if owner == current_thread => {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                    "GLFW initialization is already in progress on this thread".into(),
                )));
            }
            RuntimeState::Active(owner) if owner == current_thread => {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                    "GLFW runtime registry lost its active owner".into(),
                )));
            }
            RuntimeState::Initializing(_) | RuntimeState::Active(_) => {
                return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                    "GLFW runtime is active or initializing on another thread".into(),
                )));
            }
        }
        drop(state);

        let lifecycle_guard = RuntimeLifecycleGuard {
            owner: current_thread,
        };

        let configuration = requested.unwrap_or_default();
        glfw::init_hint(glfw::InitHint::JoystickHatButtons(
            configuration.hat_buttons,
        ));
        let glfw = crate::glfw_backend::init(|error, description| {
            log::error!("GLFW error {error:?}: {description}");
        })
        .map_err(|_| VMNLError::new(VMNLErrorKind::GlfwInitFailed))?;
        log::debug!(
            "initialized GLFW {} backend",
            crate::glfw_backend::backend_name(&glfw)
        );

        let mut state = GLFW_INIT_LOCK.lock().map_err(|_| {
            VMNLError::new(VMNLErrorKind::InvalidState(
                "GLFW initialization lock is poisoned".into(),
            ))
        })?;
        if *state != RuntimeState::Initializing(current_thread) {
            return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                "GLFW runtime ownership changed during initialization".into(),
            )));
        }
        *state = RuntimeState::Active(current_thread);
        drop(state);

        let runtime = Rc::new_cyclic(|weak_runtime: &Weak<GlfwRuntime>| {
            let mut glfw = glfw;
            let weak_runtime = weak_runtime.clone();
            crate::glfw_backend::set_error_callback(&mut glfw, move |error, description| {
                if let Some(runtime) = weak_runtime.upgrade() {
                    runtime.dispatch_glfw_error(error, description);
                } else {
                    log::error!("GLFW error {error:?}: {description}");
                }
            });
            Self {
                glfw,
                configuration,
                callbacks: RefCell::new(GlfwCallbacks::default()),
                joystick_events: RefCell::new(JoystickEventDispatcher::default()),
                joystick_callback_panic: RefCell::new(None),
                _lifecycle_guard: lifecycle_guard,
            }
        });
        THREAD_GLFW_RUNTIME.with(|slot| *slot.borrow_mut() = Rc::downgrade(&runtime));
        Ok(runtime)
    }

    /// Borrows the shared glfw-rs token without changing its process-wide reference count.
    #[inline]
    pub(crate) const fn glfw(&self) -> &glfw::Glfw {
        &self.glfw
    }

    pub(crate) fn sample_joystick(
        &self,
        id: crate::JoystickId,
    ) -> VMNLResult<Option<crate::JoystickSample>> {
        let (sample, errors) =
            self.capture_glfw_errors(|| crate::glfw_backend::sample_joystick(id));
        self.resume_joystick_callback_panic();
        finish_joystick_sample(sample, errors)
    }

    pub(crate) fn set_error_callback(
        &self,
        callback: Option<Box<dyn FnMut(VMNLErrorKind, String)>>,
    ) {
        self.callbacks.borrow_mut().error_callback = callback;
    }

    fn dispatch_glfw_error(&self, error: glfw::Error, description: String) {
        if let Some(errors) = self.callbacks.borrow_mut().error_captures.last_mut() {
            errors.push(format!("{error:?}: {description}"));
        }

        let kind = crate::glfw_backend::map_error(error);
        let message = crate::glfw_backend::callback_message(error, description);
        if let Some(callback) = self.callbacks.borrow_mut().error_callback.as_mut() {
            callback(kind, message);
        } else {
            log::error!("GLFW error {error:?}: {message}");
        }
    }

    fn capture_glfw_errors<R>(&self, operation: impl FnOnce() -> R) -> (R, Vec<String>) {
        self.callbacks.borrow_mut().error_captures.push(Vec::new());
        let result = operation();
        let errors = self
            .callbacks
            .borrow_mut()
            .error_captures
            .pop()
            .unwrap_or_default();
        (result, errors)
    }

    pub(crate) fn subscribe_joystick_events(
        self: &Rc<Self>,
        queue: Rc<RefCell<JoystickEventQueue>>,
    ) -> JoystickEventSubscription {
        if self.joystick_events.borrow_mut().subscribe(&queue) {
            let weak_runtime = Rc::downgrade(self);
            let mut glfw = self.glfw.clone();
            glfw.set_joystick_callback(move |id, event| {
                let Some(runtime) = weak_runtime.upgrade() else {
                    return;
                };
                capture_joystick_callback_panic(&runtime.joystick_callback_panic, || {
                    runtime.dispatch_joystick_event(id, event);
                });
            });
        }

        JoystickEventSubscription {
            runtime: Rc::clone(self),
            queue,
        }
    }

    pub(crate) fn unsubscribe_joystick_events(&self, queue: &Rc<RefCell<JoystickEventQueue>>) {
        if self.joystick_events.borrow_mut().unsubscribe(queue) {
            let mut glfw = self.glfw.clone();
            glfw.unset_joystick_callback();
        }
    }

    fn dispatch_joystick_event(&self, id: glfw::JoystickId, event: glfw::JoystickEvent) {
        let Some(event) = translate_joystick_event(self.glfw.get_time(), id, event) else {
            return;
        };
        let id = match event.kind() {
            crate::EventKind::JoystickConnected { id }
            | crate::EventKind::JoystickDisconnected { id } => *id,
            _ => return,
        };
        let user_pointer = crate::glfw_backend::get_joystick_user_pointer(id);
        let queues = self.joystick_events.borrow_mut().subscribers();
        dispatch_joystick_event_to_subscribers(
            queues,
            &event,
            user_pointer,
            &self.joystick_callback_panic,
        );
    }

    pub(crate) fn resume_joystick_callback_panic(&self) {
        if let Some(panic) = take_joystick_callback_panic(&self.joystick_callback_panic) {
            resume_unwind(panic);
        }
    }
}

fn translate_joystick_event(
    timestamp: f64,
    raw_id: glfw::JoystickId,
    event: glfw::JoystickEvent,
) -> Option<crate::Event> {
    let id = crate::JoystickId::from_index(raw_id as usize)?;
    let kind = match event {
        glfw::JoystickEvent::Connected => crate::EventKind::JoystickConnected { id },
        glfw::JoystickEvent::Disconnected => crate::EventKind::JoystickDisconnected { id },
    };
    Some(crate::Event::new(timestamp, kind))
}

pub(crate) struct JoystickEventSubscription {
    runtime: Rc<GlfwRuntime>,
    queue: Rc<RefCell<JoystickEventQueue>>,
}

impl Drop for JoystickEventSubscription {
    fn drop(&mut self) {
        self.runtime.unsubscribe_joystick_events(&self.queue);
    }
}

fn capture_joystick_callback_panic(
    pending: &RefCell<Option<Box<dyn Any + Send>>>,
    callback: impl FnOnce(),
) {
    if let Err(panic) = catch_unwind(AssertUnwindSafe(callback)) {
        let mut pending = pending.borrow_mut();
        if pending.is_none() {
            *pending = Some(panic);
        }
    }
}

fn take_joystick_callback_panic(
    pending: &RefCell<Option<Box<dyn Any + Send>>>,
) -> Option<Box<dyn Any + Send>> {
    pending.borrow_mut().take()
}

#[derive(Default)]
struct JoystickEventDispatcher {
    queues: Vec<Weak<RefCell<JoystickEventQueue>>>,
}

impl JoystickEventDispatcher {
    fn subscribe(&mut self, queue: &Rc<RefCell<JoystickEventQueue>>) -> bool {
        self.prune();
        if self
            .queues
            .iter()
            .filter_map(Weak::upgrade)
            .any(|registered| Rc::ptr_eq(&registered, queue))
        {
            return false;
        }

        let was_empty = self.queues.is_empty();
        self.queues.push(Rc::downgrade(queue));
        was_empty
    }

    fn unsubscribe(&mut self, queue: &Rc<RefCell<JoystickEventQueue>>) -> bool {
        self.queues.retain(|registered| {
            registered
                .upgrade()
                .is_some_and(|registered| !Rc::ptr_eq(&registered, queue))
        });
        self.queues.is_empty()
    }

    fn subscribers(&mut self) -> Vec<Rc<RefCell<JoystickEventQueue>>> {
        let mut subscribers = Vec::new();
        self.queues.retain(|registered| {
            let Some(queue) = registered.upgrade() else {
                return false;
            };
            subscribers.push(queue);
            true
        });
        subscribers
    }

    fn prune(&mut self) {
        self.queues
            .retain(|registered| registered.strong_count() > 0);
    }
}

#[derive(Default)]
pub(crate) struct JoystickEventQueue {
    events: VecDeque<crate::Event>,
    callback: Option<Box<JoystickEventCallback>>,
}

type JoystickEventCallback = dyn FnMut(&crate::Event, *mut std::ffi::c_void);

impl JoystickEventQueue {
    pub(crate) fn drain_events(&mut self) -> std::collections::vec_deque::Drain<'_, crate::Event> {
        self.events.drain(..)
    }

    pub(crate) fn clear(&mut self) {
        self.events.clear();
    }
}

fn dispatch_joystick_event_to_queue(
    queue: &Rc<RefCell<JoystickEventQueue>>,
    event: &crate::Event,
    user_pointer: *mut std::ffi::c_void,
    pending_panic: &RefCell<Option<Box<dyn Any + Send>>>,
) {
    let (mut callback, should_call) = {
        let mut queue = queue.borrow_mut();
        queue.events.push_back(event.clone());
        (queue.callback.take(), pending_panic.borrow().is_none())
    };

    let Some(mut callback_fn) = callback.take() else {
        return;
    };
    let result = if should_call {
        catch_unwind(AssertUnwindSafe(|| callback_fn(event, user_pointer)))
    } else {
        Ok(())
    };
    queue.borrow_mut().callback = Some(callback_fn);
    if let Err(panic) = result {
        let mut pending = pending_panic.borrow_mut();
        if pending.is_none() {
            *pending = Some(panic);
        }
    }
}

fn dispatch_joystick_event_to_subscribers(
    queues: Vec<Rc<RefCell<JoystickEventQueue>>>,
    event: &crate::Event,
    user_pointer: *mut std::ffi::c_void,
    pending_panic: &RefCell<Option<Box<dyn Any + Send>>>,
) {
    for queue in queues {
        dispatch_joystick_event_to_queue(&queue, event, user_pointer, pending_panic);
    }
}

fn glfw_input_error(
    operation: &'static str,
    errors: impl IntoIterator<Item = String>,
) -> VMNLError {
    VMNLError::new(VMNLErrorKind::GlfwInputOperationFailed {
        operation,
        message: errors.into_iter().collect::<Vec<_>>().join("; "),
    })
}

fn finish_joystick_sample(
    sample: Result<Option<crate::JoystickSample>, String>,
    callback_errors: Vec<String>,
) -> VMNLResult<Option<crate::JoystickSample>> {
    if !callback_errors.is_empty() {
        return Err(glfw_input_error("joystick sample", callback_errors));
    }
    sample.map_err(|message| glfw_input_error("joystick sample", vec![message]))
}

/// Clears the global owner marker only after `GlfwRuntime::glfw` has dropped.
#[derive(Debug)]
struct RuntimeLifecycleGuard {
    owner: ThreadId,
}

impl Drop for RuntimeLifecycleGuard {
    fn drop(&mut self) {
        // Recover the metadata lock if it was poisoned: clearing the owner after GLFW termination
        // is required before a later initialization can safely apply new hints.
        let mut state = GLFW_INIT_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if matches!(
            *state,
            RuntimeState::Initializing(owner) | RuntimeState::Active(owner)
                if owner == self.owner
        ) {
            *state = RuntimeState::Inactive;
        }
    }
}

fn check_configuration(
    active: InputRuntimeConfig,
    requested: Option<InputRuntimeConfig>,
) -> VMNLResult<()> {
    if let Some(requested) = requested {
        if requested != active {
            return Err(VMNLError::new(
                VMNLErrorKind::GlfwInitializationConfigConflict {
                    active_hat_buttons: active.hat_buttons,
                    requested_hat_buttons: requested.hat_buttons,
                },
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::panic)]

    use super::*;
    use std::cell::Cell;

    #[test]
    fn joystick_dispatcher_fans_out_independent_queues_without_replay_or_consumption() {
        let mut dispatcher = JoystickEventDispatcher::default();
        let first = Rc::new(RefCell::new(JoystickEventQueue::default()));
        let second = Rc::new(RefCell::new(JoystickEventQueue::default()));
        let pending_panic = RefCell::new(None);

        assert!(dispatcher.subscribe(&first));
        let connected = crate::Event::new(
            1.0,
            crate::EventKind::JoystickConnected {
                id: crate::JoystickId::Joystick2,
            },
        );
        dispatch_joystick_event_to_subscribers(
            dispatcher.subscribers(),
            &connected,
            std::ptr::null_mut(),
            &pending_panic,
        );
        assert!(!dispatcher.subscribe(&first));
        assert!(!dispatcher.subscribe(&second));
        let disconnected = crate::Event::new(
            2.0,
            crate::EventKind::JoystickDisconnected {
                id: crate::JoystickId::Joystick2,
            },
        );
        dispatch_joystick_event_to_subscribers(
            dispatcher.subscribers(),
            &disconnected,
            std::ptr::null_mut(),
            &pending_panic,
        );

        let first_events: Vec<_> = first.borrow_mut().drain_events().collect();
        let second_events: Vec<_> = second.borrow_mut().drain_events().collect();
        assert_eq!(first_events.len(), 2);
        assert_eq!(
            first_events[0].timestamp_seconds().to_bits(),
            1.0_f64.to_bits()
        );
        assert_eq!(second_events.len(), 1);
        assert_eq!(
            second_events[0].timestamp_seconds().to_bits(),
            2.0_f64.to_bits()
        );
        assert!(matches!(
            second_events[0].kind(),
            crate::EventKind::JoystickDisconnected {
                id: crate::JoystickId::Joystick2
            }
        ));

        let next = crate::Event::new(
            3.0,
            crate::EventKind::JoystickConnected {
                id: crate::JoystickId::Joystick2,
            },
        );
        dispatch_joystick_event_to_subscribers(
            dispatcher.subscribers(),
            &next,
            std::ptr::null_mut(),
            &pending_panic,
        );
        assert_eq!(first.borrow().events.len(), 1);
        assert_eq!(second.borrow().events.len(), 1);
        assert!(!dispatcher.unsubscribe(&first));
        assert!(dispatcher.unsubscribe(&second));
    }

    #[test]
    fn joystick_connection_translation_preserves_slot_and_clock_time() {
        let connected = translate_joystick_event(
            2.5,
            glfw::JoystickId::Joystick2,
            glfw::JoystickEvent::Connected,
        )
        .expect("GLFW joystick slot should map");
        assert_eq!(connected.timestamp_seconds().to_bits(), 2.5_f64.to_bits());
        assert!(matches!(
            connected.kind(),
            crate::EventKind::JoystickConnected {
                id: crate::JoystickId::Joystick2
            }
        ));

        let disconnected = translate_joystick_event(
            3.5,
            glfw::JoystickId::Joystick2,
            glfw::JoystickEvent::Disconnected,
        )
        .expect("GLFW joystick slot should map");
        assert_eq!(
            disconnected.timestamp_seconds().to_bits(),
            3.5_f64.to_bits()
        );
        assert!(matches!(
            disconnected.kind(),
            crate::EventKind::JoystickDisconnected {
                id: crate::JoystickId::Joystick2
            }
        ));
    }

    #[test]
    fn joystick_callback_panic_is_deferred_after_fanout_and_skips_more_callbacks() {
        let pending = RefCell::new(None);
        let first = Rc::new(RefCell::new(JoystickEventQueue::default()));
        let second = Rc::new(RefCell::new(JoystickEventQueue::default()));
        first.borrow_mut().callback = Some(Box::new(|_, _| panic!("callback panic")));
        let second_callback_called = Rc::new(Cell::new(false));
        let second_callback_observed = Rc::clone(&second_callback_called);
        second.borrow_mut().callback = Some(Box::new(move |_, _| {
            second_callback_observed.set(true);
        }));
        let event = crate::Event::new(
            1.0,
            crate::EventKind::JoystickConnected {
                id: crate::JoystickId::Joystick2,
            },
        );

        dispatch_joystick_event_to_subscribers(
            vec![Rc::clone(&first), Rc::clone(&second)],
            &event,
            std::ptr::null_mut(),
            &pending,
        );

        assert!(pending.borrow().is_some());
        assert_eq!(first.borrow().events.len(), 1);
        assert_eq!(second.borrow().events.len(), 1);
        assert!(!second_callback_called.get());

        let resumed = catch_unwind(AssertUnwindSafe(|| {
            resume_unwind(take_joystick_callback_panic(&pending).expect("panic was captured"));
        }));
        assert!(resumed.is_err());
    }

    #[test]
    fn synchronous_callback_receives_the_user_pointer_without_the_registry_borrowed() {
        let dispatcher = Rc::new(RefCell::new(JoystickEventDispatcher::default()));
        let late_queue = Rc::new(RefCell::new(JoystickEventQueue::default()));
        let first_queue = Rc::new(RefCell::new(JoystickEventQueue::default()));
        dispatcher.borrow_mut().subscribe(&first_queue);
        let expected_pointer = Box::into_raw(Box::new(7_u8)).cast::<std::ffi::c_void>();
        let expected_address = expected_pointer as usize;
        let observed = Rc::new(Cell::new(false));
        let observed_callback = Rc::clone(&observed);
        let callback_dispatcher = Rc::clone(&dispatcher);
        let callback_late_queue = Rc::clone(&late_queue);
        first_queue.borrow_mut().callback = Some(Box::new(move |event, user_pointer| {
            assert_eq!(user_pointer as usize, expected_address);
            assert!(matches!(
                event.kind(),
                crate::EventKind::JoystickDisconnected {
                    id: crate::JoystickId::Joystick2
                }
            ));
            callback_dispatcher
                .borrow_mut()
                .subscribe(&callback_late_queue);
            observed_callback.set(true);
        }));

        let subscribers = dispatcher.borrow_mut().subscribers();
        let pending_panic = RefCell::new(None);
        let disconnected = crate::Event::new(
            2.0,
            crate::EventKind::JoystickDisconnected {
                id: crate::JoystickId::Joystick2,
            },
        );
        dispatch_joystick_event_to_subscribers(
            subscribers,
            &disconnected,
            expected_pointer,
            &pending_panic,
        );

        assert!(observed.get());
        assert_eq!(first_queue.borrow().events.len(), 1);
        assert!(late_queue.borrow().events.is_empty());
        // SAFETY: This is the exact allocation created above and the callback did not retain or
        // free the pointer. Reclaim it once the synchronous callback test has completed.
        unsafe { drop(Box::from_raw(expected_pointer.cast::<u8>())) };
    }

    #[test]
    fn input_runtime_shares_configuration_and_drops_glfw_at_last_owner() {
        glfw::init_hint(glfw::InitHint::Platform(glfw::Platform::Null));

        let first = InputRuntime::builder()
            .hat_buttons(false)
            .build()
            .expect("Null GLFW runtime should initialize");
        let runtime = Rc::downgrade(&first.runtime);
        let acquired = InputRuntime::acquire().expect("default acquisition should adopt config");
        let context_owner = GlfwRuntime::acquire(None).expect("context should adopt config");
        let matching = InputRuntime::builder()
            .hat_buttons(false)
            .build()
            .expect("matching explicit config should reuse runtime");

        for index in 0..crate::JoystickId::COUNT {
            let id = crate::JoystickId::from_index(index).expect("valid joystick slot");
            assert!(first
                .sample_joystick(id)
                .expect("Null backend query should succeed")
                .is_none());
        }

        first
            .update_gamepad_mappings("00000000000000000000000000000001,VMNL test,a:b0,")
            .expect("valid mapping should update");

        let callback_count = Rc::new(Cell::new(0));
        let observed_callback_count = Rc::clone(&callback_count);
        first.runtime.set_error_callback(Some(Box::new(move |_, _| {
            observed_callback_count.set(observed_callback_count.get() + 1);
        })));
        let invalid_mapping = first
            .update_gamepad_mappings("abcdef,invalid mapping,a:b0,")
            .expect_err("mapping parser callback error must fail the VMNL operation");
        assert!(matches!(
            invalid_mapping.kind(),
            VMNLErrorKind::GlfwInputOperationFailed {
                operation: "gamepad mapping update",
                message,
            } if message.contains("InvalidValue")
        ));
        assert_eq!(callback_count.get(), 1);
        first.runtime.set_error_callback(None);

        assert!(Rc::ptr_eq(&first.runtime, &acquired.runtime));
        assert!(Rc::ptr_eq(&first.runtime, &context_owner));
        assert!(Rc::ptr_eq(&first.runtime, &matching.runtime));
        assert!(!acquired.configuration().hat_buttons());

        let rejected_from_worker = thread::spawn(|| {
            matches!(
                InputRuntime::acquire(),
                Err(error) if matches!(error.kind(), VMNLErrorKind::InvalidState(_))
            )
        })
        .join()
        .expect("worker acquisition should return normally");
        assert!(rejected_from_worker);

        let Err(conflict) = InputRuntime::builder().hat_buttons(true).build() else {
            panic!("conflicting config should fail before another GLFW init");
        };
        assert!(matches!(
            conflict.kind(),
            VMNLErrorKind::GlfwInitializationConfigConflict {
                active_hat_buttons: false,
                requested_hat_buttons: true,
            }
        ));

        drop(first);
        drop(acquired);
        drop(matching);
        assert!(runtime.upgrade().is_some());
        drop(context_owner);
        assert!(runtime.upgrade().is_none());

        let next = InputRuntime::builder()
            .hat_buttons(true)
            .build()
            .expect("last-owner drop should allow a new config");
        assert!(next.configuration().hat_buttons());
        drop(next);

        let default = InputRuntime::acquire().expect("omitted config should use GLFW defaults");
        assert!(default.configuration().hat_buttons());
    }

    #[test]
    fn standalone_sample_distinguishes_absent_present_and_backend_failures() {
        let absent =
            finish_joystick_sample(Ok(None), Vec::new()).expect("an absent slot is not an error");
        assert!(absent.is_none());

        let expected = crate::JoystickSample::from_native(
            crate::JoystickId::Joystick2,
            Some("test controller".to_owned()),
            None,
            vec![0.25],
            vec![1],
            vec![0],
            None,
        );
        let present = finish_joystick_sample(Ok(Some(expected.clone())), Vec::new())
            .expect("a present unmapped slot is valid")
            .expect("sample should be present");
        assert_eq!(present, expected);
        assert!(present.gamepad().is_none());

        let backend_error = finish_joystick_sample(Err("query failed".to_owned()), Vec::new())
            .expect_err("backend query failure should be returned");
        assert!(matches!(
            backend_error.kind(),
            VMNLErrorKind::GlfwInputOperationFailed {
                operation: "joystick sample",
                message,
            } if message == "query failed"
        ));

        let callback_error =
            finish_joystick_sample(Ok(None), vec!["PlatformError: backend callback".to_owned()])
                .expect_err("callback errors override GLFW sentinel values");
        assert!(matches!(
            callback_error.kind(),
            VMNLErrorKind::GlfwInputOperationFailed { message, .. }
                if message == "PlatformError: backend callback"
        ));
    }

    #[test]
    fn configuration_conflict_preserves_active_and_requested_values() {
        let active = InputRuntimeConfig { hat_buttons: false };
        let requested = InputRuntimeConfig { hat_buttons: true };

        let result = check_configuration(active, Some(requested));

        assert!(matches!(
            result,
            Err(error)
                if matches!(
                    error.kind(),
                    VMNLErrorKind::GlfwInitializationConfigConflict {
                        active_hat_buttons: false,
                        requested_hat_buttons: true,
                    }
                )
        ));
    }
}
