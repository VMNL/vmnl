// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

#include <stdint.h>
#include <unistd.h>
#include <X11/Xlib.h>

_Static_assert(sizeof(Window) == sizeof(uintptr_t), "X11 Window must fit the Rust usize handle");

int vmnl_paint_x11_probe(Display *display, uintptr_t window, int width, int height)
{
    if (display == NULL || window == None || width <= 0 || height <= 0) {
        write(2, "Invalid parameters to vmnl_paint_x11_probe\n", 43);
        return 1;
    }

    GC context = XCreateGC(display, (Window)window, 0, NULL);
    if (context == NULL) {
        write(2, "Failed to create X11 graphics context\n", 38);
        return 2;
    }

    // Xwayland needs initial contents to map a NoApi window on some compositors.
    XSetForeground(display, context, 0);
    XFillRectangle(display, (Window)window, context, 0, 0,
        (unsigned int)width, (unsigned int)height);
    XFreeGC(display, context);
    XSync(display, False);
    return 0;
}
