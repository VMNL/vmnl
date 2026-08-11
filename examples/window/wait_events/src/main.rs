// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Event-driven redraw using an explicit blocking wait.

use vmnl::{common::Rgba, d2::Shape, Context, Event, Key, PresentMode, VMNLResult, Window};

fn main() -> VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL wait events")
        .size(900, 600)
        .unset_configure_window_polling()
        .set_clear_color(Rgba::rgb(12, 16, 24))
        .present_mode(PresentMode::Auto)
        .build(&context)?;
    window.set_char_polling(true);
    window.set_close_polling(true);
    window.set_key_polling(true);

    let cool_shape = Shape::rect(320.0, 220.0)
        .position(290.0, 190.0)
        .color(Rgba::rgba(60, 170, 255, 255))
        .build(&context)?;
    let warm_shape = Shape::rect(320.0, 220.0)
        .position(290.0, 190.0)
        .color(Rgba::rgba(255, 150, 60, 255))
        .build(&context)?;

    let mut use_warm_shape = false;
    let mut redraw = true;
    println!("Press Space to change the shape or Escape to close.");

    while window.is_open() {
        if redraw {
            let shape = if use_warm_shape {
                &warm_shape
            } else {
                &cool_shape
            };
            window.render().draw2d([shape]).submit()?;
            redraw = false;
        }

        window.wait_events();
        for event in window.poll_events() {
            match event {
                Event::Closed
                | Event::KeyPressed {
                    key: Key::Escape, ..
                } => window.close(),
                Event::Text(' ') => {
                    use_warm_shape = !use_warm_shape;
                    redraw = true;
                }
                _ => redraw = true,
            }
        }
    }

    Ok(())
}
