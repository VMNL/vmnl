# vmnl_audio

Independent audio domain with a private CPAL backend. Control defaults to manual;
`thread(true)` explicitly enables the VMNL control worker.

- [Public API, usage, defaults, and costs](../../docs/api/reference/audio.md)
- [Audio architecture and acceptance contract](../../docs/audio.md)
- [Build prerequisites](../../docs/build.md)
- [Tests and qualification limits](../../docs/testing.md#audio-tests)

Use `vmnl_audio` directly or the facade's optional `vmnl::audio` module. Audio-only
facade clients disable default features. Generate item-level Rustdoc with
`cargo doc -p vmnl_audio --no-deps`.
