#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Hugo Duda
# SPDX-License-Identifier: MIT

# Disposable-runner display setup only; local runs use joystick_ci.py directly.
set -euo pipefail
: "${VMNL_CI_BACKEND:?}" "${VMNL_CI_MODE:?}" "${VMNL_CI_ARTIFACT_DIR:?}"
case "$VMNL_CI_BACKEND" in x11|wayland) ;; *) exit 2 ;; esac
case "$VMNL_CI_MODE" in runtime|window) ;; *) exit 2 ;; esac
mkdir -p "$VMNL_CI_ARTIFACT_DIR"
unset WAYLAND_DISPLAY WAYLAND_SOCKET
export XDG_SESSION_TYPE="$VMNL_CI_BACKEND"
openbox >"$VMNL_CI_ARTIFACT_DIR/openbox.log" 2>&1 &
openbox_pid=$!
weston_pid=''
cleanup() {
    if [[ -n "$weston_pid" ]]; then kill "$weston_pid" 2>/dev/null || true; fi
    kill "$openbox_pid" 2>/dev/null || true
}
trap cleanup EXIT
for _ in {1..40}; do
    xprop -root _NET_SUPPORTING_WM_CHECK 2>/dev/null | grep -q 'window id' && break
    kill -0 "$openbox_pid"
    sleep 0.25
done
xprop -root _NET_SUPPORTING_WM_CHECK | grep -q 'window id'
if [[ "$VMNL_CI_BACKEND" == wayland ]]; then
    export XDG_RUNTIME_DIR
    XDG_RUNTIME_DIR=$(mktemp -d)
    export WAYLAND_DISPLAY=wayland-vmnl-joystick
    weston --backend=x11 --renderer=pixman --no-config --idle-time=0 \
        --socket="$WAYLAND_DISPLAY" >"$VMNL_CI_ARTIFACT_DIR/weston.log" 2>&1 &
    weston_pid=$!
    for _ in {1..40}; do
        [[ -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY" ]] && break
        kill -0 "$weston_pid"
        sleep 0.25
    done
    [[ -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY" ]]
fi
if [[ "$VMNL_CI_MODE" == window ]]; then
    vulkaninfo --summary >"$VMNL_CI_ARTIFACT_DIR/vulkaninfo.log" 2>&1
fi
python3 tools/joystick_ci.py "$VMNL_CI_BACKEND" "$VMNL_CI_MODE" \
    "$VMNL_CI_ARTIFACT_DIR/runs"
