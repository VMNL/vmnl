# Documentation review checklist

- The facade surface, Rustdoc, reference page, coverage matrix, and snapshot agree.
- Every public field, variant, method, relevant explicit impl, trait, alias, and derive macro is covered.
- `pub(crate)`, `pub(super)`, `raw::__private`, and blanket impls are absent.
- Defaults, validation, units, ownership, errors, allocations, synchronization, and platform limits are explicit.
- Performance/synchronization claims are proved or labelled not specified.
- 3D pages state that rendering is scaffolded.
- GPU/display snippets use `no_run`, not `ignore`.
- Workflows link existing examples instead of copying them.
- Every GLFW call used by VMNL has an inventory entry. New limitations follow the
  [GLFW portability protocol](glfw_portability_protocol.md).
- Callback errors, no-ops, sentinel values, and best-effort requests are not conflated.
- `just docs-api-check` and applicable repository checks pass.
