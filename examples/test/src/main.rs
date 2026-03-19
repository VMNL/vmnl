use vmnl::{Window, Graphics, VMNLVertex};

fn main()
{
    let mut win: Window = Window::new(1920, 1080, "Window")
    .expect("Failed");
    let vertex: Graphics = Graphics::create_vertices(
        VMNLVertex { position: [1920.0, 1080.0], color: [0.0, 0.0, 255.0] },
        VMNLVertex { position: [0.0,  1080.0], color: [0.0, 0.0, 100.0] },
        VMNLVertex { position: [1920.0, 0.0], color: [0.0, 0.0, 100.0] }
    );

    while win.is_open() {
        win.poll_event();
        win.draw(&vertex);
    }
}
