use vmnl::{Window, Graphics, Context, VMNLVertex, VMNLResult, Key, MouseButton, Event};

fn handle_event_test(
    event: &Event,
    win: &mut Window
) -> ()
{
    match event {
        Event::Closed => {
            println!("[Event] Closed");
            win.close();
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
        win.close();
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
        for event in win.poll_events() {
            handle_event_test(&event, &mut win);
        }
        handle_keybind_test(&mut win);
        handle_mousebind_test(&mut win);
        win.render(&[&vertex, &vertex2]);
    }
    return Ok(());
}
