extern crate glfw;
use std::{
    error::Error,
    fmt
};
use glfw::{
    Action,
    Key
};
use std::sync::Arc;

pub type VMNLResult<T> = Result<T, VMNLError>;

#[derive(Debug)]
pub enum VMNLError
{
    VMNLInitFailed,
    WindowCreationFailed,
    VulkanInitFailed,
    InvalidState(&'static str),
}

impl fmt::Display for VMNLError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            Self::VMNLInitFailed =>                     write!(f, "VMNL Error: VMNL initialization failed"),
            Self::WindowCreationFailed =>               write!(f, "VMNL Error: window creation failed"),
            Self::VulkanInitFailed =>                   write!(f, "VMNL Error: Vulkan initialization failed"),
            Self::InvalidState(msg) =>   write!(f, "VMNL Error: invalid state: {msg}"),
        }
    }
}

impl Error for VMNLError {}

/// cf: https://github.com/PistonDevelopers/glfw-rs
struct WindowHandle
{
    instance:             glfw::Glfw,
    context:              glfw::PWindow,
    events:               glfw::GlfwReceiver<(f64, glfw::WindowEvent)>
}

struct WindowState
{
    is_ready:             bool,
    is_open:              bool
}

struct WindowConfig
{
    is_close_with_escape: bool,
    title:                String,
    width:                u32,
    height:               u32
}

pub struct Window
{
    window_handle:        WindowHandle,
    window_state:         WindowState,
    window_config:        WindowConfig
}

impl Window
{
    fn init_window(
        mut instance: glfw::Glfw,
        width: u32,
        height: u32,
        title: &str
    ) -> (glfw::PWindow, glfw::GlfwReceiver<(f64, glfw::WindowEvent)>)
    {
        return instance
        .create_window(width, height, title, glfw::WindowMode::Windowed)
        .expect("VMNL Error: Failed to create VMNL window.");
    }

    pub fn new(
        width: u32,
        height: u32,
        title: &str
    ) -> VMNLResult<Self>
    {
        #[cfg(feature = "safe")] {
            if ready == false {
                return;
            }
        }
        let mut instance = glfw::init(glfw::fail_on_errors)
        .map_err(|_| VMNLError::VMNLInitFailed)?;
        println!("VMNL log: Window Initialized.");
        instance.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        let (mut window, events) =
        Window::init_window(instance.clone(), width, height, title);

        if window.is_visible() == false {
            window.show();
        }
        window.set_key_polling(true);
        println!("VMNL log: Window created.");
        Ok(Self {
            window_handle: WindowHandle {
                instance,
                context: window.into(),
                events
            },
            window_state: WindowState {
                is_ready: true,
                is_open: false
            },
            window_config: WindowConfig {
                is_close_with_escape: true,
                title:                title.to_string(),
                width,
                height
            }
        })
    }

    pub fn is_open(&mut self) -> bool
    {
        #[cfg(feature = "safe")] {
            if ready == false {
                return false;
            }
        }
        self.window_state.is_open = !self.window_handle.context.should_close();
        return self.window_state.is_open;
    }

    pub fn is_ready(&self) -> bool
    {
        return self.window_state.is_ready;
    }

    pub fn should_close_with_escape_pressed(
        &mut self,
        closed: bool
    ) -> ()
    {
        self.window_config.is_close_with_escape = closed;
    }

    pub fn poll_event(&mut self) -> ()
    {
        #[cfg(feature = "safe")] {
            if ready == false {
                return;
            }
        }
        self.window_handle.instance.poll_events();
        for (_, event) in glfw::flush_messages(&self.window_handle.events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    if self.window_config.is_close_with_escape {
                        self.window_handle.context.set_should_close(true);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn get_width(&self) -> u32
    {
        return self.window_config.width;
    }

    pub fn get_height(&self) -> u32
    {
        return self.window_config.height;
    }

    pub fn get_glfw_window(&self) -> &glfw::PWindow
    {
        return &self.window_handle.context;
    }

}

impl Drop for Window
{
    fn drop(&mut self) -> ()
    {
        eprintln!("VMNL log: Window named \"{}\" destroyed.", self.window_config.title);
    }
}
