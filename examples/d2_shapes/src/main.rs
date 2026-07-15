// SPDX-FileCopyrightText: 2026 VMNL
// SPDX-License-Identifier: MIT

use vmnl::{
    common::{BufferMemoryPreference, Rgba},
    d2::{Anchor, LineCap, Shape, Vector2f, Vertex2D},
    Context, Event, Key, MouseButton, PresentMode, RenderMode, VMNLResult, Window,
};

fn v2(x: f32, y: f32) -> Vector2f {
    Vector2f { x, y }
}

fn vertex(x: f32, y: f32, color: Rgba) -> Vertex2D {
    Vertex2D {
        position: v2(x, y),
        color,
    }
}

fn handle_event(event: &Event) {
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

fn handle_keyboard(win: &mut Window) {
    if win.input().keyboard().is_pressed(Key::E) {
        println!("[Keybind] Key E is pressed");
    }
    if win.input().keyboard().is_released(Key::E) {
        println!("[Keybind] Key E is released");
    }
    if win.input().keyboard().is_down(Key::E) {
        println!("[Keybind] Key E is down");
    }
    if win.input().keyboard().is_pressed(Key::Escape) {
        win.close();
    }
}

fn handle_mouse(win: &mut Window) {
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
    const VERTICES: [Vertex2D; 6] = [
        Vertex2D {
            position: Vector2f { x: 700.0, y: 600.0 },
            color: Rgba::rgb(255, 255, 255),
        },
        Vertex2D {
            position: Vector2f { x: 700.0, y: 350.0 },
            color: Rgba::rgb(255, 0, 0),
        },
        Vertex2D {
            position: Vector2f { x: 938.0, y: 523.0 },
            color: Rgba::rgb(255, 255, 0),
        },
        Vertex2D {
            position: Vector2f { x: 847.0, y: 802.0 },
            color: Rgba::rgb(0, 255, 0),
        },
        Vertex2D {
            position: Vector2f { x: 553.0, y: 802.0 },
            color: Rgba::rgb(0, 255, 255),
        },
        Vertex2D {
            position: Vector2f { x: 462.0, y: 523.0 },
            color: Rgba::rgb(0, 0, 255),
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
        .title("VMNL d2_shapes")
        .size(1920, 1080)
        .size_limit(Some(600), Some(600), Some(2000), Some(1500))?
        .set_clear_color(Rgba::rgb(12, 16, 24))
        .present_mode(PresentMode::Auto)
        .build(&ctx)?;

    let triangle: Shape = Shape::triangle(v2(1200.0, 300.0), v2(1600.0, 300.0), v2(1200.0, 500.0))
        .vertex_colors(Rgba::RED, Rgba::GREEN, Rgba::BLUE)
        .buffer_memory_preference(BufferMemoryPreference::Host)
        .build(&ctx)?;

    let triangle_from_vertices: Shape = Shape::triangle_from_vertices([
        vertex(1280.0, 560.0, Rgba::rgba(255, 255, 255, 180)),
        vertex(1600.0, 620.0, Rgba::rgba(255, 180, 0, 180)),
        vertex(1380.0, 820.0, Rgba::rgba(0, 220, 255, 180)),
    ])
    .buffer_memory_preference(BufferMemoryPreference::Device)
    .build(&ctx)?;

    let pentagon_indexed: Shape = create_pentagon_indexed(&ctx)?;

    let rectangle: Shape = Shape::rect(100.0, 300.0)
        .position(1400.0, 800.0)
        .color([255, 0, 0, 255])
        .rotation(90.0)
        .anchor(Anchor::Center)
        .buffer_memory_preference(BufferMemoryPreference::Device)
        .build(&ctx)?;

    let custom_origin_rect: Shape = Shape::rect(220.0, 120.0)
        .position(160.0, 140.0)
        .color(Rgba::rgba(0, 180, 255, 190))
        .origin(20.0, 90.0)
        .rotation(-25.0)
        .buffer_memory_preference(BufferMemoryPreference::Host)
        .build(&ctx)?;

    let anchored_rect: Shape = Shape::rect(160.0, 160.0)
        .position(420.0, 160.0)
        .color(Rgba::rgba(255, 255, 255, 120))
        .anchor(Anchor::BottomRight)
        .rotation(35.0)
        .build(&ctx)?;

    let line_butt: Shape = Shape::line(v2(100.0, 500.0), v2(300.0, 700.0))
        .color(Rgba::YELLOW)
        .width(30.0)
        .cap(LineCap::Butt)
        .build(&ctx)?;
    let line_round: Shape = Shape::line(v2(120.0, 760.0), v2(420.0, 860.0))
        .color(Rgba::CYAN)
        .width(42.0)
        .cap(LineCap::Round)
        .build(&ctx)?;
    let line_square: Shape = Shape::line(v2(160.0, 940.0), v2(480.0, 900.0))
        .color(Rgba::MAGENTA)
        .width(36.0)
        .cap(LineCap::Square)
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
    println!("Press Escape to close. Render mode alternates between PerObject and Batched.");

    let mut frame: u64 = 0;
    while win.is_open() {
        for event in win.poll_events() {
            handle_event(&event);
        }
        handle_keyboard(&mut win);
        handle_mouse(&mut win);

        let mode = if frame % 240 < 120 {
            RenderMode::PerObject
        } else {
            RenderMode::Batched
        };

        win.render()
            .mode(mode)
            .draw2d([
                &rectangle,
                &custom_origin_rect,
                &anchored_rect,
                &triangle,
                &triangle_from_vertices,
                &pentagon_indexed,
                &line_butt,
                &line_round,
                &line_square,
            ])
            .submit()?;
        frame = frame.wrapping_add(1);
    }
    Ok(())
}
