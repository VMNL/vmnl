# VMNL - Vulkan Multimedia Networking Library

<div align="center">

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Vulkan](https://img.shields.io/badge/Vulkan-AE0F28?style=for-the-badge&logo=Vulkan&logoColor=fff)
![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)
![Windows](https://img.shields.io/badge/Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-000000?style=for-the-badge&logo=apple&logoColor=white)

<br>

![CI](https://img.shields.io/github/actions/workflow/status/VMNL/vmnl/ci.yml?style=flat-square)
![License](https://img.shields.io/github/license/VMNL/vmnl?style=flat-square)
![Repo size](https://img.shields.io/github/repo-size/VMNL/vmnl?style=flat-square)

</div>

---

## 📌 Table of Contents

- [Overview](#-overview)
- [Design Principles](#-design-principles)
- [Features](#-features)
- [Build](#-build)
- [Status](#️-current-state)
- [Roadmap](#-roadmap)
- [References](#-references)
- [Authors](#️-authors)

---

## 👁️‍🗨️ Overview

**VMNL (Vulkan Multimedia Networking Library)** is a **Rust** library built on **Vulkan**.

Goal: provide a **predictable**, **high-performance**, and **modular** base for:
- Game engines
- Real-time applications
- Rendering systems

VMNL unifies:
- Graphics
- Audio *(planned)*
- Networking *(planned)*

Constraint: **no hidden complexity, only structured complexity.**

---

## ⚠️ Current State

- API unstable
- Modules incomplete
- Architecture still evolving
- Breaking changes expected

---

## 🧠 Design Principles

- **Explicit control**
  - GPU resources, synchronization, pipelines
- **No hidden cost**
  - No implicit allocation or state
- **Deterministic behavior**
- **Modular design**
  - `graphics`
  - `audio`
  - `network`
- **Reproducible architecture**
  - No global state
  - Explicit device / queue selection

---

## ⚙️ Features

### Graphics

- Vulkan instance / device setup
- Physical device selection (no surface dependency)
- Queue family management
- Swapchain lifecycle
- Render pass + framebuffer
- Graphics pipeline
- Vertex buffers
- Push constants
- Command buffers
- Frame synchronization
- Planned:
  - Independant clear function
  - Subbuffer rendering (basic optimisation)

### Windowing / Input

- GLFW integration
- Planned:
  - Event system
  - Immediate input (polling)
  - Unified input abstraction

---

### Low-level

- Thin abstraction over Vulkan
- Full control over:
  - memory
  - pipelines
  - synchronization

### High-level (planned)

- Simplified API:
  - window
  - rendering primitives/texture/text
  - scene system
- API utilities:
  - get_global_bounds/get_local_bounds
  - intersects
  - contains
  - compute_aabb/compute_screen_bounds
  - upload_buffer/upload_texture_with_staging
  - generate_mipmaps/batch_renderer.draw()
  - create_camera_2d/create_camera_3d
  - draw_debug_bounds/draw_debug_line/draw_debug_grid
---

## 📦 Build

### Requirements

- Rust (stable)
- Cargo
- Vulkan loader (`libvulkan.so`, `vulkan-1.dll`, etc.)
- Vulkan-compatible GPU

### Run

```bash
git clone git@github.com:VMNL/vmnl.git
cd vmnl
cargo run
```

---

## 🎯 Roadmap

### Short-term
- Instance / device stabilization
- Window ↔ renderer separation
- Input system
- Resource management
- Vertex rendering
- Audio module

### Mid-term
- Shape rendering
- Texture rendering + system (staging, batching, mipmaps)
- Text rendering
- 2D abstraction
- Scene system
- High-level API utilities

### Long-term
- Networking
- Cross-platform robustness
- API freeze
- ECS
- C / C++ bindings
- 3D abstraction

---

## 📚 References

- Vulkan Specification
  https://registry.khronos.org/vulkan/

- Vulkano
  https://vulkano.rs/

- GLFW
  https://www.glfw.org/

---

## ✍️ Authors

- [Hugo DUDA](https://github.com/hugoduda) - **Lead tech** Vulkan/GLFW & low-level/high-level **referent/developer**
- [Maxence PIERRE](https://github.com/Anexoms) - **Product owner** & Low-level **developer**
- [Nathan FLACHAT](https://github.com/NathanFlachat) - low-level/high-level **developer**
- [Naouel Bouhali](https://github.com/BouhaliNaouel) - high-level **developer**
- [Julien Michel](https://github.com/JulienMICHELgithub) - Web **referent/developer**
- [Laszlo SERDET](https://github.com/lszsrd) - Networking **referent/developer**
