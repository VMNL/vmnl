# Maintenance

These pages define the documentation contract, API and GLFW portability procedures, and generated
inventory checks.

- [`glfw_portability_protocol.md`](glfw_portability_protocol.md) defines the mandatory process for
  a new GLFW use or platform limitation.
- [`glfw_platform_inventory.md`](glfw_platform_inventory.md) is generated from the canonical TOML
  inventory and includes GLFW functions not currently exposed by VMNL.
- [`keyboard_capability_matrix.md`](keyboard_capability_matrix.md) tracks GLFW keyboard
  capabilities through their VMNL contracts, deterministic evidence, and native qualification.

Run `just docs-api-check` before review; run `just docs-api-update` only after an intentional
surface, inventory, or documentation change.
