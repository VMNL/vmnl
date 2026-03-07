extern crate glfw;
use std::{
    error::Error,
    fmt
};
use glfw::{
    Action,
    Key
};

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

pub struct Window
{
    instance:             glfw::Glfw,
    context:              glfw::PWindow,
    events:               glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    is_ready:             bool,
    is_open:              bool,
    is_close_with_escape: bool
}

impl Window
{
    fn init_window(mut instance: glfw::Glfw) -> (glfw::PWindow, glfw::GlfwReceiver<(f64, glfw::WindowEvent)>)
    {
        return instance
        .create_window(1920, 1080, "VMNL Window", glfw::WindowMode::Windowed)
        .expect("VMNL Error: Failed to create VMNL window");
    }

    pub fn new() -> VMNLResult<Self>
    {
        let instance = glfw::init(glfw::fail_on_errors)
        .map_err(|_| VMNLError::VMNLInitFailed)?;
        println!("VMNL log: VMNL Initialized.");
        // instance.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        let (mut window, events) =
        Window::init_window(instance.clone());

        if window.is_visible() == false {
            window.show();
        }
        window.set_key_polling(true);
        println!("VMNL log: Window created.");
        Ok(Self {
            instance,
            events:               events,
            context:              window,
            is_open:              false,
            is_ready:             true,
            is_close_with_escape: true
        })
    }

    pub fn is_open(&mut self) -> bool
    {
        self.is_open = !self.context.should_close();
        return self.is_open;
    }

    pub fn is_ready(&self) -> bool
    {
        return self.is_ready;
    }

    pub fn should_close_with_escape_pressed(&mut self, closed: bool)
    {
        self.is_close_with_escape = closed;
    }

    pub fn poll_event(&mut self)
    {
        self.instance.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    if self.is_close_with_escape {
                        self.context.set_should_close(true);
                    }
                }
                _ => {}
            }
        }
    }

}
