use vmnl::{Window, Graphics};

fn main()
{
    let mut win = Window::new(1920, 1080, "Window")
    .expect("Failed");
    let mut triangle = Graphics::new(&win);

    while win.is_open() {
        win.poll_event();
        triangle.draw_triangle();
    }
}
