use vmnl::{
    Context, Event, Key, LineCap, MouseButton, Rgba, Shape, VMNLResult, Vector2f, Vertex, Window,
};

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Rgba {
    Rgba { r, g, b, a }
}

fn handle_event_test(event: &Event) {
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
        Event::Resized { width, height } => {
            println!("[Event] Resized: {}x{}", width, height);
        }
        Event::FramebufferResized { width, height } => {
            println!("[Event] Framebuffer resized: {}x{}", width, height);
        }
        Event::KeyPressed { key, repeat } => {
            println!("[Event] Key pressed: {:?} (repeat: {})", key, repeat);
        }
        Event::KeyReleased { key } => {
            println!("[Event] Key released: {:?}", key);
        }
        Event::MouseMoved { x, y } => {
            println!("[Event] Mouse moved: {} {}", x, y);
        }
        Event::MouseEntered => {
            println!("[Event] Mouse entered window");
        }
        Event::MouseLeft => {
            println!("[Event] Mouse left window");
        }
        Event::MouseButtonPressed { button } => {
            println!("[Event] Mouse button pressed: {:?}", button);
        }
        Event::MouseButtonReleased { button } => {
            println!("[Event] Mouse button released: {:?}", button);
        }
        Event::MouseScrolled { dx, dy } => {
            println!("[Event] Mouse scrolled: {} {}", dx, dy);
        }
        Event::Text(c) => {
            println!("[Event] Text input: {}", c);
        }
    }
}

fn handle_keybind_test(win: &mut Window) {
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

fn handle_mousebind_test(win: &mut Window) {
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

fn create_pentagon_indexed(ctx: &Context) -> VMNLResult<Shape> {
    const VERTICES: [Vertex; 6] = [
        // Center
        Vertex {
            position: Vector2f { x: 700.0, y: 600.0 },
            color: rgba(255, 255, 255, 255),
        },
        // Top
        Vertex {
            position: Vector2f { x: 700.0, y: 350.0 },
            color: rgba(255, 0, 0, 255),
        },
        // Upper right
        Vertex {
            position: Vector2f { x: 938.0, y: 523.0 },
            color: rgba(255, 255, 0, 255),
        },
        // Lower right
        Vertex {
            position: Vector2f { x: 847.0, y: 802.0 },
            color: rgba(0, 255, 0, 255),
        },
        // Lower left
        Vertex {
            position: Vector2f { x: 553.0, y: 802.0 },
            color: rgba(0, 255, 255, 255),
        },
        // Upper left
        Vertex {
            position: Vector2f { x: 462.0, y: 523.0 },
            color: rgba(0, 0, 255, 255),
        },
    ];
    const INDICES: [u32; 15] = [
        0, 1, 2, // center -> top -> upper right
        0, 2, 3, // center -> upper right -> lower right
        0, 3, 4, // center -> lower right -> lower left
        0, 4, 5, // center -> lower left -> upper left
        0, 5, 1, // center -> upper left -> top
    ];

    Shape::indexed(VERTICES, INDICES).build(ctx)
}

fn main() -> VMNLResult<()> {
    let ctx: Context = Context::new()?;
    let mut win: Window = Window::builder()
        .size(1920, 1080)
        .size_limit(Some(600), Some(600), Some(2000), Some(1500))?
        .set_clear_color(rgba(0, 0, 0, 255))
        .build(&ctx)?;
    let triangle: Shape = Shape::triangle([
        Vertex {
            position: Vector2f {
                x: 1200.0,
                y: 300.0,
            },
            color: rgba(255, 0, 0, 255),
        },
        Vertex {
            position: Vector2f {
                x: 1600.0,
                y: 300.0,
            },
            color: rgba(0, 255, 0, 255),
        },
        Vertex {
            position: Vector2f {
                x: 1200.0,
                y: 500.0,
            },
            color: rgba(0, 0, 255, 255),
        },
    ])
    .build(&ctx)?;
    let pentagon_indexed: Shape = create_pentagon_indexed(&ctx)?;
    let rectangle: Shape = Shape::rect(100.0, 300.0)
        .position(1400.0, 800.0)
        .color(rgba(255, 0, 0, 255))
        .rotation(90.0)
        .build(&ctx)?;
    let line: Shape = Shape::line(
        Vector2f { x: 100.0, y: 500.0 },
        Vector2f { x: 300.0, y: 700.0 },
    )
    .color(rgba(255, 255, 0, 255))
    .width(50.0)
    .cap(LineCap::Round)
    .build(&ctx)?;

    println!(
        "Monitors: {}",
        win.monitor()
            .names()
            .iter()
            .map(|name| name.clone().unwrap_or("Unknown".to_string()))
            .collect::<Vec<String>>()
            .join(", ")
    );
    while win.is_open() {
        for event in win.poll_events() {
            handle_event_test(&event);
        }
        handle_keybind_test(&mut win);
        handle_mousebind_test(&mut win);
        win.render([&rectangle, &triangle, &pentagon_indexed, &line])
            .per_object()?;
    }
    Ok(())
}
