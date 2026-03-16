use vmnl::{Window, Graphics};

fn main()
{
    let mut win: Window = Window::new(1920, 1080, "Window")
    .expect("Failed");
    let vertex: Graphics = Graphics::create_vertex(
        [-0.5, -0.5, 1.0, 0.0],
        [ 0.0,  0.5, 1.0, 0.0],
        [ 0.5, -0.5, 1.0, 0.0]
    );

    while win.is_open() {
        win.poll_event();
        win.draw(&vertex);
    }
}
