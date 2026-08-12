# Raw traits and derive macros

| Item | Namespace | Role |
|---|---|---|
| [`BufferContents`](buffer_contents.md) | Type | GPU-buffer byte-layout marker |
| [`Vertex`](vertex.md) | Type | Vertex-input description marker |
| [`Pod`](pod.md) | Type | Plain-old-data marker |
| [`Zeroable`](zeroable.md) | Type | All-zero-bit-pattern marker |
| [`Pod`, `Vertex`, `Zeroable`](derive_macros.md) | Macro | Proc-macro derives for underlying contracts |

Traits and macros intentionally share names but occupy separate Rust namespaces. The generic blanket adapters are described but omitted from generated API inventory as required by the snapshot policy.

