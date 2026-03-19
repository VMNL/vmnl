use vmnl::{Window, Graphics, VMNLVertex};

fn main()
{
    let mut win: Window = Window::new(1920, 1080, "Window")
    .expect("Failed");
    let vertex: Graphics = Graphics::create_vertices(
        VMNLVertex { position: [1020.0, 800.0], color: [255.0, 0.0, 0.0] },
        VMNLVertex { position: [400.0,    800.0], color: [255.0, 0.0, 0.0] },
        VMNLVertex { position: [1020.0, 400.0], color: [255.0, 0.0, 0.0] }
    );

    while win.is_open() {
        win.poll_event();
        win.draw(&vertex);
    }
}
