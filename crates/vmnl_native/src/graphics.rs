mod vmnl_instance;
use vmnl_instance::VMNLInstance;
use crate::Window;

pub struct Graphics
{
    vmnl_instance: VMNLInstance
}

impl Graphics
{
    pub fn new(
        window: &Window
    ) -> Self
    {
        Self {
            vmnl_instance: VMNLInstance::new(window)
        }
    }
}
