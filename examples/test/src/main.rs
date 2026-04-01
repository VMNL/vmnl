use vmnl::{Window, Graphics, Context, VMNLVertex, VMNLResult, Key, MouseButton};

fn main() -> VMNLResult<()>
{
    let ctx: Context = Context::new()?;
    let mut win: Window = Window::new(&ctx, 1920, 1080, "Window")?;
    let vertex: Graphics = Graphics::create_vertices(
        &ctx,
        VMNLVertex { position: [1020.0, 800.0], color: [0.0,   255.0, 0.0]   },
        VMNLVertex { position: [400.0,  800.0], color: [255.0, 0.0,   0.0]   },
        VMNLVertex { position: [1020.0, 400.0], color: [0.0,   0.0,   255.0] }
    );
    let vertex2: Graphics = Graphics::create_vertices(
        &ctx,
        VMNLVertex { position: [400.0,  400.0], color: [255.0, 255.0, 0.0]   },
        VMNLVertex { position: [400.0,  800.0], color: [255.0, 0.0,   0.0]   },
        VMNLVertex { position: [1020.0, 400.0], color: [0.0,   0.0,   255.0] }
    );

    while win.is_open() {
        win.poll_event();
        if win.input().mouse().is_released(MouseButton::Left) {
            println!("Mouse button left is released");
        }
        if win.input().keyboard().is_down(Key::A) {
            println!("Key A is down");
        }
        win.render(&[&vertex, &vertex2]);
    }
    Ok(())
}
