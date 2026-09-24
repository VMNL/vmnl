// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

#define _GNU_SOURCE
#include <limits.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>
#include <wayland-client.h>
#include <errno.h>

#if defined(__STDC_VERSION__) && __STDC_VERSION__ >= 202311L
    // C17 or later
    #define UNUSED_PARAM [[maybe_unused]]
#elif defined(__GNUC__) || defined(__clang__)
    // GCC and Clang
    #define UNUSED_PARAM __attribute__((unused))
#else
    // MSVC or other compilers
    #define UNUSED_PARAM
#endif

typedef struct shm_lookup {
    struct wl_shm *shm;
} shm_lookup;

static void registry_global(
    void *data,
    struct wl_registry *registry,
    uint32_t name,
    const char *interface,
    UNUSED_PARAM uint32_t version)
{
    shm_lookup *lookup = (shm_lookup *)data;

    if (lookup->shm == NULL && strcmp(interface, "wl_shm") == 0) {
        lookup->shm = wl_registry_bind(registry, name, &wl_shm_interface, 1);
    }
}

static void registry_global_remove(
    UNUSED_PARAM void *data,
    UNUSED_PARAM struct wl_registry *registry,
    UNUSED_PARAM uint32_t name)
{}

static const struct wl_registry_listener registry_listener =
{
    registry_global,
    registry_global_remove
};

struct wl_buffer *vmnl_map_wayland_probe(
    struct wl_display *display,
    struct wl_surface *surface,
    int width,
    int height,
    int *error)
{
    *error = 1;
    if (!display || !surface || width <= 0 || height <= 0 ||
        width > INT_MAX / 4 || height > INT_MAX / (width * 4)) {
        const char msg[] = "Invalid parameters for Wayland probe buffer\n";
        write(STDERR_FILENO, msg, sizeof(msg) - 1);
        return NULL;
    }
    shm_lookup lookup = { NULL };
    struct wl_registry *registry = wl_display_get_registry(display);
    if (registry == NULL) {
        const char msg[] = "Failed to get Wayland registry: ";
        write(STDERR_FILENO, msg, sizeof(msg) - 1);
        write(STDERR_FILENO, strerror(errno), strlen(strerror(errno)));
        return NULL;
    }
    if (wl_registry_add_listener(registry, &registry_listener, &lookup) < 0 ||
        wl_display_roundtrip(display) < 0 || lookup.shm == NULL) {
        const char msg[] = "Failed to bind wl_shm interface: ";
        write(STDERR_FILENO, msg, sizeof(msg) - 1);
        write(STDERR_FILENO, strerror(errno), strlen(strerror(errno)));
        wl_registry_destroy(registry);
        return NULL;
    }
    wl_registry_destroy(registry);
    *error = 2;
    const int size = width * 4 * height;
    const int fd = memfd_create("vmnl-platform-probe", MFD_CLOEXEC);
    if (fd < 0) {
        const char msg[] = "Failed to create memfd: ";
        write(STDERR_FILENO, msg, sizeof(msg) - 1);
        write(STDERR_FILENO, strerror(errno), strlen(strerror(errno)));
        goto done;
    }
    if (ftruncate(fd, size) < 0) {
        const char msg[] = "Failed to truncate memfd: ";
        write(STDERR_FILENO, msg, sizeof(msg) - 1);
        write(STDERR_FILENO, strerror(errno), strlen(strerror(errno)));
        if (close(fd) < 0) {
            const char msg[] = "Failed to close memfd: ";
            write(STDERR_FILENO, msg, sizeof(msg) - 1);
            write(STDERR_FILENO, strerror(errno), strlen(strerror(errno)));
        }
        goto done;
    }
    struct wl_shm_pool *pool = wl_shm_create_pool(lookup.shm, fd, size);
    if (close(fd) < 0) {
        const char msg[] = "Failed to close memfd: ";
        write(STDERR_FILENO, msg, sizeof(msg) - 1);
        write(STDERR_FILENO, strerror(errno), strlen(strerror(errno)));
        if (pool != NULL) {
            wl_shm_pool_destroy(pool);
        }
        goto done;
    }
    if (pool == NULL) {
        const char msg[] = "Failed to create wl_shm_pool: ";
        write(STDERR_FILENO, msg, sizeof(msg) - 1);
        write(STDERR_FILENO, strerror(errno), strlen(strerror(errno)));
        goto done;
    }
    struct wl_buffer *buffer = wl_shm_pool_create_buffer(
        pool,
        0,
        width,
        height,
        width * 4,
        WL_SHM_FORMAT_XRGB8888
    );
    wl_shm_pool_destroy(pool);
    if (buffer == NULL) {
        const char msg[] = "Failed to create wl_buffer: ";
        write(STDERR_FILENO, msg, sizeof(msg) - 1);
        write(STDERR_FILENO, strerror(errno), strlen(strerror(errno)));
        goto done;
    }

    // A zero-filled shm buffer maps the GLFW NoApi surface without a GPU context.
    wl_surface_attach(surface, buffer, 0, 0);
    wl_surface_damage(surface, 0, 0, width, height);
    wl_surface_commit(surface);
    *error = 3;
    if (wl_display_flush(display) < 0) {
        const char msg[] = "Failed to flush Wayland display: ";
        write(STDERR_FILENO, msg, sizeof(msg) - 1);
        write(STDERR_FILENO, strerror(errno), strlen(strerror(errno)));
        wl_buffer_destroy(buffer);
        buffer = NULL;
    }
    wl_shm_destroy(lookup.shm);
    return buffer;

done:
    wl_shm_destroy(lookup.shm);
    return NULL;
}

void vmnl_destroy_wayland_probe_buffer(struct wl_buffer *buffer)
{
    wl_buffer_destroy(buffer);
}
