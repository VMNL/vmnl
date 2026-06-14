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

/// Draw submission strategy for objects inside each logical render pass.
///
/// The mode may change how objects are submitted inside one pass, but it must
/// not reorder calls to [`FrameRenderer::draw2d`] and [`FrameRenderer::draw3d`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum RenderMode {
    /// Submit each object independently.
    #[default]
    PerObject,
    /// Batch compatible objects when a backend supports it.
    ///
    /// This currently falls back to [`RenderMode::PerObject`].
    Batched,
}

/// Pending frame render pass.
enum FramePass<'g> {
    /// 2D render pass.
    D2 { items: Vec<RenderItem2D> },
    /// 3D render pass.
    D3 {
        camera: &'g Camera,
        items: Vec<RenderItem3D>,
    },
}

/// Pending frame render operation created by [`Window::render`].
///
/// `FrameRenderer` records the logical passes requested for the next frame.
/// No swapchain image is acquired and no GPU command is submitted until
/// [`FrameRenderer::submit`] is called.
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

    /// Set the draw submission strategy used by this frame.
    ///
    /// The mode applies to the objects inside each pass. It does not change the
    /// order of the passes added with [`FrameRenderer::draw2d`] or
    /// [`FrameRenderer::draw3d`].
    ///
    /// # Arguments
    /// - `mode`: Submission strategy used for objects inside each pass.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, RenderMode, Window};
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// # let rect = Shape::rect(100.0, 100.0).build(&context)?;
    /// window.render()
    ///     .mode(RenderMode::PerObject)
    ///     .draw2d([&rect])
    ///     .submit()?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn mode(mut self, mode: RenderMode) -> Self {
        self.mode = mode;
        self
    }

    /// Add a 2D render pass to the pending frame.
    ///
    /// Each item must implement [`Drawable2D`], which prevents passing 3D
    /// resources to this function at compile time. The pass is rendered in the
    /// same logical order as it was added when [`FrameRenderer::submit`] runs.
    ///
    /// # Arguments
    /// - `drawables`: Fixed-size array of 2D drawable references to render in this pass.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// let rect = Shape::rect(100.0, 100.0).build(&context)?;
    ///
    /// window.render()
    ///     .draw2d([&rect])
    ///     .submit()?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// Add a 3D render pass to the pending frame.
    ///
    /// Each item must implement [`Drawable3D`], which prevents passing 2D
    /// shapes to this function at compile time. The pass is accepted by the API
    /// so 3D resources can already be built and wired, but
    /// [`FrameRenderer::submit`] currently returns an explicit error if any 3D
    /// pass is present.
    ///
    /// # Arguments
    /// - `camera`: Camera parameters attached to this 3D pass.
    /// - `drawables`: Fixed-size array of 3D drawable references to render in this pass.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # use vmnl_graphics::common::Rgba;
    /// # use vmnl_graphics::d3::{Camera, Mesh, Vector3f, Vertex3D};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// let camera = Camera::default();
    /// let vertices = [
    ///     Vertex3D { position: Vector3f { x: 0.0, y: 0.0, z: 0.0 }, color: Rgba::new(255, 0, 0, 255) },
    ///     Vertex3D { position: Vector3f { x: 1.0, y: 0.0, z: 0.0 }, color: Rgba::new(0, 255, 0, 255) },
    ///     Vertex3D { position: Vector3f { x: 0.0, y: 1.0, z: 0.0 }, color: Rgba::new(0, 0, 255, 255) },
    /// ];
    /// let mesh = Mesh::indexed(vertices, [0, 1, 2]).build(&context)?;
    ///
    /// let result = window.render()
    ///     .draw3d(&camera, [&mesh])
    ///     .submit();
    /// assert!(result.is_err());
    /// # Ok(())
    /// # }
    /// ```
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
    /// The recorded passes are consumed by this call. A frame with no draw pass
    /// is valid and still clears/presents the swapchain image.
    ///
    /// For this backend version, 2D passes are submitted through the existing
    /// 2D renderer. If any 3D pass was recorded, this function returns
    /// `VMNLErrorKind::InvalidState("3D rendering is not implemented yet")`
    /// before acquiring a swapchain image.
    ///
    /// # Returns
    /// Returns `Ok(())` when command recording, queue submission, presentation,
    /// and optional event polling complete successfully.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, RenderMode, Window};
    /// # use vmnl_graphics::common::Rgba;
    /// # use vmnl_graphics::d2::Shape;
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// let rectangle = Shape::rect(200.0, 100.0)
    ///     .position(100.0, 150.0)
    ///     .color(Rgba::new(255, 0, 0, 255))
    ///     .build(&context)?;
    ///
    /// window.render()
    ///     .mode(RenderMode::PerObject)
    ///     .draw2d([&rectangle])
    ///     .submit()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `InvalidState` if a 3D pass is present. Otherwise returns an
    /// error if frame acquisition, command recording, submission, or
    /// presentation fails.
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
    /// Begin a new pending frame render operation.
    ///
    /// Use the returned [`FrameRenderer`] to append 2D or 3D passes, then call
    /// [`FrameRenderer::submit`] to render and present the frame.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use vmnl_graphics::{Context, Window};
    /// # fn main() -> vmnl_graphics::VMNLResult<()> {
    /// # let context = Context::new()?;
    /// # let mut window = Window::builder().build(&context)?;
    /// window.render().submit()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn render(&mut self) -> FrameRenderer<'_, '_> {
        FrameRenderer::new(self)
    }
}
