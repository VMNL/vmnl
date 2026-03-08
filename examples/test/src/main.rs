use vmnl::{Window, Graphics};

fn main()
{
    let mut _win = Window::new(1920, 1080, "Window")
    .expect("Failed");
    let _graphics = Graphics::new();

    // while win.is_open() {
    //     win.poll_event();
    // }
}
