use vmnl::{Window, Graphics, Context, VMNLVertex, VMNLResult, Key, MouseButton, Event, VMNLRect};

fn handle_event_test(
    event: &Event,
) -> ()
{
    match event {
        Event::Closed => {
            println!("[Event] Closed");
        }
        Event::FocusGained => {
            println!("[Event] Focus gained");
        }
        Event::FocusLost => {
            println!("[Event] Focus lost");
        }
        Event::Resized {width, height} => {
            println!("[Event] Resized: {}x{}", width, height);
        }
        Event::FramebufferResized {width, height} => {
            println!("[Event] Framebuffer resized: {}x{}", width, height);
        }
        Event::KeyPressed {key, repeat} => {
            println!("[Event] Key pressed: {:?} (repeat: {})", key, repeat);
        }
        Event::KeyReleased {key} => {
            println!("[Event] Key released: {:?}", key);
        }
        Event::MouseMoved {x, y} => {
            println!("[Event] Mouse moved: {} {}", x, y);
        }
        Event::MouseEntered => {
            println!("[Event] Mouse entered window");
        }
        Event::MouseLeft => {
            println!("[Event] Mouse left window");
        }
        Event::MouseButtonPressed {button} => {
            println!("[Event] Mouse button pressed: {:?}", button);
        }
        Event::MouseButtonReleased {button} => {
            println!("[Event] Mouse button released: {:?}", button);
        }
        Event::MouseScrolled {dx, dy} => {
            println!("[Event] Mouse scrolled: {} {}", dx, dy);
        }
        Event::Text(c) => {
            println!("[Event] Text input: {}", c);
        }
    }
}

fn handle_keybind_test(
    win: &mut Window
) -> ()
{
    if win.input().keyboard().is_pressed(Key::E) {
        println!("[Keybind] Key E is pressed");
    }
    if win.input().keyboard().is_released(Key::E) {
        println!("[Keybind] Key E is released");
    }
    if win.input().keyboard().is_down(Key::E) {
        println!("[Keybind] Key E is down");
    }
}

fn handle_mousebind_test(
    win: &mut Window
) -> ()
{
    if win.input().mouse().is_pressed(MouseButton::Left) {
        println!("[Mousebind] Mouse button left is pressed");
    }
    if win.input().mouse().is_released(MouseButton::Left) {
        println!("[Mousebind] Mouse button left is released");
    }
    if win.input().mouse().is_down(MouseButton::Left) {
        println!("[Mousebind] Mouse button left is down");
    }
}

fn create_quad_manual(
    ctx: &Context
) -> [Graphics; 2]
{
    let vertex: Graphics = Graphics::create_vertices_shape(
        &ctx,
        VMNLVertex { position: [1020.0, 800.0], color: [0.0,   255.0, 0.0]   },
        VMNLVertex { position: [400.0,  800.0], color: [255.0, 0.0,   0.0]   },
        VMNLVertex { position: [1020.0, 400.0], color: [0.0,   0.0,   255.0] }
    );
    let vertex2: Graphics = Graphics::create_vertices_shape(
        &ctx,
        VMNLVertex { position: [400.0,  400.0], color: [255.0, 255.0, 0.0]   },
        VMNLVertex { position: [400.0,  800.0], color: [255.0, 0.0,   0.0]   },
        VMNLVertex { position: [1020.0, 400.0], color: [0.0,   0.0,   255.0] }
    );

    return [vertex, vertex2];
}

fn create_quad_indexed(
    ctx: &Context
) -> Graphics
{
    let vertices: [VMNLVertex; 4] = [
        VMNLVertex { position: [400.0,  400.0], color: [255.0, 255.0, 0.0]   }, // ! top-left
        VMNLVertex { position: [1020.0, 400.0], color: [0.0,   255.0, 0.0]   }, // ! top-right
        VMNLVertex { position: [1020.0, 800.0], color: [0.0,   0.0,   255.0] }, // ! bottom-right
        VMNLVertex { position: [400.0,  800.0], color: [255.0, 0.0,   0.0]   }, // ! bottom-left
    ];
    let indices: [u32; 6] = [
        0, 1, 2, // ! first triangle (top-left, top-right, bottom-right)
        2, 3, 0, // ! second triangle (bottom-right, bottom-left, top-left)
    ];

    return Graphics::create_indexed_shape(&ctx, &vertices, &indices);
}

fn main() -> VMNLResult<()>
{
    let ctx:          Context       = Context::new()?;
    let mut win:      Window        = Window::new(&ctx, 1920, 1080, "Window")?;
    let _quad_manual: [Graphics; 2] = create_quad_manual(&ctx);
    let _quad_indexed: Graphics      = create_quad_indexed(&ctx);
    let rectangle: Graphics         = Graphics::create_rectangle_shape(
        &ctx,
        VMNLRect { position: [400.0, 400.0], size: [620.0, 400.0] },
        [255.0, 200.0, 0.0]
    );

    while win.is_open() {
        for event in win.poll_events() {
            handle_event_test(&event);
        }
        handle_keybind_test(&mut win);
        handle_mousebind_test(&mut win);
        win.render(&[&rectangle]);
    }
    return Ok(());
}
