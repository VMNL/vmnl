# Reference page template

Every item page uses these headings, retaining “Not applicable” or “Not specified” where appropriate:

1. `Public path and maturity`
2. `Purpose and use cases`
3. `Public API`
4. `Construction, defaults, and validation`
5. `Units, coordinates, and valid ranges`
6. `Ownership, lifecycle, and threading`
7. `Errors, panics, and failure conditions`
8. `Allocation, transfers, synchronization, and GPU cost`
9. `Platform, Vulkan, and display constraints`
10. `Example and related types`

Exact signatures and local contracts belong in Rustdoc. The page must name every public field, variant, inherent method, and relevant explicit implementation. Generic blanket implementations are omitted.

