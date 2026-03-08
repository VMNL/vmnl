mod vmnl_instance;
use vmnl_instance::VMNLInstance;

pub struct Graphics
{
    vmnl_instance: VMNLInstance
}

impl Graphics
{
    pub fn new() -> Self
    {
        Self {
            vmnl_instance: VMNLInstance::new()
        }
    }
}
