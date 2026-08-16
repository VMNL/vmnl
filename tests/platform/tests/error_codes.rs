// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Regression tests for total conversion of GLFW 3.4 error codes.

use glfw::{ffi, Error};

#[test]
fn all_glfw_3_4_error_codes_round_trip() {
    let codes = [
        ffi::GLFW_NO_ERROR,
        ffi::GLFW_NOT_INITIALIZED,
        ffi::GLFW_NO_CURRENT_CONTEXT,
        ffi::GLFW_INVALID_ENUM,
        ffi::GLFW_INVALID_VALUE,
        ffi::GLFW_OUT_OF_MEMORY,
        ffi::GLFW_API_UNAVAILABLE,
        ffi::GLFW_VERSION_UNAVAILABLE,
        ffi::GLFW_PLATFORM_ERROR,
        ffi::GLFW_FORMAT_UNAVAILABLE,
        ffi::GLFW_NO_WINDOW_CONTEXT,
        ffi::GLFW_CURSOR_UNAVAILABLE,
        ffi::GLFW_FEATURE_UNAVAILABLE,
        ffi::GLFW_FEATURE_UNIMPLEMENTED,
        ffi::GLFW_PLATFORM_UNAVAILABLE,
    ];

    for code in codes {
        assert_eq!(Error::from_raw(code).as_raw(), code);
    }
}

#[test]
fn arbitrary_raw_codes_never_require_an_enum_transmute() {
    for code in [i32::MIN, -1, 1, 0x0001_000f, i32::MAX] {
        assert_eq!(Error::from_raw(code), Error::Unknown(code));
        assert_eq!(Error::from_raw(code).as_raw(), code);
    }
}
