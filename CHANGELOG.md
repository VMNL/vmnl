# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- Emit stick click and direction transitions, including return to center, for the
  mapped gamepad in GLFW slot 1 through `Window::poll_events`.
- Emit joystick connection and disconnection events for GLFW slot 1 by comparing
  presence between polls, independently of gamepad mapping. An already connected
  device emits a connection event on the first poll.

### Fixed

- Allow an optional SDL mapping file through `VMNL_GAMEPAD_MAPPINGS` at context
  creation for controllers missing a built-in GLFW mapping. Include a Linux Xbox
  360 stick/click mapping for GUID `030000005e0400008e02000045050000`; hardware
  verification of right-stick axes and clicks remains pending.

- Add opt-in `VMNL_GAMEPAD_DIAGNOSTICS` output on P to inspect raw and mapped
  controller state without changing mappings or the normal example output.
