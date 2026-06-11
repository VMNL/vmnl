////////////////////////////////////////////////////////////////////////////////
/// SPDX-FileCopyrightText: 2026 Hugo Duda
/// SPDX-License-Identifier: MIT
///
/// Public rendering entry points for `Window`.
////////////////////////////////////////////////////////////////////////////////
use crate::d2::{Drawable2D, RenderItem2D};
use crate::d3::{Camera, Drawable3D, RenderItem3D};
use crate::window::Window;
use crate::{VMNLError, VMNLErrorKind, VMNLResult};

/// Draw submission strategy for objects inside a render pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum RenderMode {
    /// One object emits one draw call.
    #[default]
    PerObject,
    /// Batch compatible objects when implemented.
    Batched,
}

enum FramePass<'g> {
    D2 {
        items: Vec<RenderItem2D>,
    },
    D3 {
        camera: &'g Camera,
        items: Vec<RenderItem3D>,
    },
}

/// Pending frame render operation.
pub struct FrameRenderer<'w, 'g> {
    window: &'w mut Window,
    mode: RenderMode,
    passes: Vec<FramePass<'g>>,
}

impl<'w, 'g> FrameRenderer<'w, 'g> {
    pub(crate) fn new(window: &'w mut Window) -> Self {
        Self {
            window,
            mode: RenderMode::default(),
            passes: Vec::new(),
        }
    }

    /// Set the draw submission strategy for this frame.
    #[must_use]
    pub fn mode(mut self, mode: RenderMode) -> Self {
        self.mode = mode;
        self
    }

    /// Add a 2D render pass. Pass order is preserved exactly at submit time.
    #[must_use]
    pub fn draw2d<D, const N: usize>(mut self, drawables: [&'g D; N]) -> Self
    where
        D: Drawable2D + ?Sized,
    {
        self.passes.push(FramePass::D2 {
            items: drawables
                .into_iter()
                .map(Drawable2D::render_item_2d)
                .collect(),
        });
        self
    }

    /// Add a 3D render pass. Pass order is preserved exactly at submit time.
    #[must_use]
    pub fn draw3d<D, const N: usize>(mut self, camera: &'g Camera, drawables: [&'g D; N]) -> Self
    where
        D: Drawable3D + ?Sized,
    {
        self.passes.push(FramePass::D3 {
            camera,
            items: drawables
                .into_iter()
                .map(Drawable3D::render_item_3d)
                .collect(),
        });
        self
    }

    /// Submit the frame to the GPU.
    ///
    /// # Errors
    /// Returns an error if frame acquisition, command recording, submission, or presentation fails.
    pub fn submit(self) -> VMNLResult<()> {
        let mut items_2d: Vec<RenderItem2D> = Vec::new();

        for pass in self.passes {
            match pass {
                FramePass::D2 { items } => items_2d.extend(items),
                FramePass::D3 { camera, items } => {
                    let _ = (camera, items.len());
                    return Err(VMNLError::new(VMNLErrorKind::InvalidState(
                        "3D rendering is not implemented yet".to_string(),
                    )));
                }
            }
        }

        self.window.inner.render_2d(self.mode, &items_2d)
    }
}

impl Window {
    /// Begin a frame render operation.
    #[inline]
    #[must_use]
    pub fn render(&mut self) -> FrameRenderer<'_, '_> {
        FrameRenderer::new(self)
    }
}
