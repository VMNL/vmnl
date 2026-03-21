use std::{
    error::Error,
    fmt
};

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
pub type VMNLResult<T> = Result<T, VMNLError>;
