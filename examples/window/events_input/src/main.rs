// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

use vmnl::{
    Context, Cursor, CursorMode, Event, EventKind, Key, MouseButton, PresentMode, StandardCursor,
    VMNLResult, Window,
};

fn print_event(context: &Context, event: &Event) {
    match event.kind() {
        EventKind::KeyPressed {
            key,
            scancode,
            modifiers,
            repeat,
        } => print_key_event(
            context,
            event.timestamp_seconds(),
            "pressed",
            *key,
            *scancode,
            modifiers,
            Some(*repeat),
        ),
        EventKind::KeyReleased {
            key,
            scancode,
            modifiers,
        } => print_key_event(
            context,
            event.timestamp_seconds(),
            "released",
            *key,
            *scancode,
            modifiers,
            None,
        ),
        kind => println!("[event @ {:.6}s] {kind:?}", event.timestamp_seconds()),
    }
}

fn print_key_event(
    context: &Context,
    timestamp: f64,
    action: &str,
    key: Key,
    event_scancode: vmnl::Scancode,
    modifiers: &vmnl::Modifiers,
    repeat: Option<bool>,
) {
    let queried_scancode = context.get_key_scancode(key);
    println!(
        "[key @ {timestamp:.6}s] action={action} key={key:?} name={:?} \
         event_scancode={} queried_scancode={queried_scancode:?} scancode_name={:?} \
         scancode_matches={} modifiers={modifiers:?} repeat={repeat:?}",
        context.get_key_name(key),
        event_scancode.as_raw(),
        context.get_scancode_name(event_scancode),
        queried_scancode == Some(event_scancode),
    );
}

fn print_key_query(context: &Context, key: Key) {
    let scancode = context.get_key_scancode(key);
    println!(
        "key query: key={key:?} name={:?} scancode={scancode:?} scancode_name={:?}",
        context.get_key_name(key),
        scancode.and_then(|value| context.get_scancode_name(value)),
    );
}

fn print_monitor_summary(window: &Window) {
    let monitor_names = window
        .monitor()
        .names()
        .into_iter()
        .map(|name| name.unwrap_or_else(|| "Unknown".to_string()))
        .collect::<Vec<_>>();

    println!("monitors: {}", monitor_names.join(", "));
    if let Some(primary) = window.monitor().primary() {
        println!(
            "primary: name={:?} pos={:?} workarea={:?} scale={:?}",
            primary.name, primary.position, primary.workarea, primary.content_scale
        );
    }
    println!("monitor count: {}", window.monitor().infos().len());
}

fn configure_runtime_window(window: &mut Window) -> VMNLResult<()> {
    window.set_error_callback(|kind, message| {
        eprintln!("[glfw] {kind:?}: {message}");
    });

    window.set_title("VMNL window_events_input");
    window.set_size(960, 540)?;
    window.set_size_limits(Some(320), Some(240), Some(1920), Some(1080))?;
    window.set_aspect_ratio(Some((16, 9)))?;
    window.set_position(80, 80);
    window.opacity(0.96);
    window.set_clear_color([20, 24, 32, 255]);

    window.unconfigure_window_polling();
    window.enable_keyboard_polling();
    window.enable_mouse_polling();
    window.enable_window_state_polling();
    window.set_char_polling(true);
    window.set_char_mods_polling(true);
    window.set_scroll_polling(true);
    window.set_content_scale_polling(true);
    window.set_drag_and_drop_polling(true);
    window.set_refresh_polling(true);
    window.set_sticky_keys(true);
    window.set_sticky_mouse_buttons(true);
    window.set_lock_key_modifier_reporting(true);

    Ok(())
}

fn apply_keybinds(
    window: &mut Window,
    standard_cursor: &Cursor,
    custom_cursor: &Cursor,
) -> VMNLResult<()> {
    let keyboard = window.input().keyboard();
    let close = keyboard.is_pressed(Key::Escape);
    let focus = keyboard.is_pressed(Key::F);
    let iconify = keyboard.is_pressed(Key::I);
    let maximize = keyboard.is_pressed(Key::M);
    let restore = keyboard.is_pressed(Key::R);
    let hide_show = keyboard.is_pressed(Key::H);
    let clear_aspect = keyboard.is_pressed(Key::C);
    let cursor_normal = keyboard.is_pressed(Key::N);
    let cursor_hidden = keyboard.is_pressed(Key::V);
    let cursor_disabled = keyboard.is_pressed(Key::D);
    let cursor_captured = keyboard.is_pressed(Key::G);
    let center_cursor = keyboard.is_pressed(Key::P);
    let assign_standard_cursor = keyboard.is_pressed(Key::S);
    let assign_custom_cursor = keyboard.is_pressed(Key::U);
    let restore_default_cursor = keyboard.is_pressed(Key::O);
    let toggle_sticky_keys = keyboard.is_pressed(Key::K);
    let toggle_text_delivery = keyboard.is_pressed(Key::F9);
    let toggle_legacy_text_delivery = keyboard.is_pressed(Key::F10);
    let any_arrow = keyboard.is_any_down(&[Key::Left, Key::Right, Key::Up, Key::Down]);
    let any_was_used = keyboard.is_one_used();
    let any_was_pressed = keyboard.is_one_pressed();
    let any_was_released = keyboard.is_one_released();

    let mouse = window.input().mouse();
    let mouse_used = mouse.is_one_used();
    let left_or_right_down = mouse.is_any_down(&[MouseButton::Left, MouseButton::Right]);
    let toggle_key_delivery = mouse.is_pressed(MouseButton::Right);

    if close {
        window.close();
    }
    if focus {
        window.focus();
    }
    if iconify {
        window.iconify();
    }
    if maximize {
        window.maximize();
    }
    if restore {
        window.restore();
    }
    if hide_show {
        window.hide();
        window.show();
    }
    if clear_aspect {
        window.set_aspect_ratio(None)?;
    }
    if cursor_normal {
        window.set_cursor_mode(CursorMode::Normal)?;
    }
    if cursor_hidden {
        window.set_cursor_mode(CursorMode::Hidden)?;
    }
    if cursor_disabled {
        window.set_cursor_mode(CursorMode::Disabled)?;
    }
    if cursor_captured {
        window.set_cursor_mode(CursorMode::Captured)?;
    }
    if center_cursor {
        let (width, height) = window.get_size();
        window.set_cursor_position(f64::from(width) / 2.0, f64::from(height) / 2.0)?;
    }
    if assign_standard_cursor {
        window.set_cursor(Some(standard_cursor))?;
    }
    if assign_custom_cursor {
        window.set_cursor(Some(custom_cursor))?;
    }
    if restore_default_cursor {
        window.set_cursor(None)?;
    }
    if toggle_sticky_keys {
        let enabled = !window.is_sticky_keys_enabled();
        window.set_sticky_keys(enabled);
        println!("[input] sticky_keys={enabled}");
    }
    if toggle_text_delivery {
        let enabled = !window.is_char_polling_enabled();
        window.set_char_polling(enabled);
        println!("[input] text_delivery={enabled}");
    }
    if toggle_legacy_text_delivery {
        let enabled = !window.is_char_mods_polling_enabled();
        window.set_char_mods_polling(enabled);
        println!("[input] legacy_text_with_modifiers_delivery={enabled}");
    }
    if toggle_key_delivery {
        let enabled = !window.is_key_polling_enabled();
        window.set_key_polling(enabled);
        println!("[input] key_delivery={enabled}");
    }
    if any_arrow {
        println!("[input] arrow key is down");
    }
    if any_was_used || any_was_pressed || any_was_released {
        println!(
            "[input] keyboard used={any_was_used} pressed={any_was_pressed} released={any_was_released}"
        );
    }
    if mouse_used || left_or_right_down {
        println!("[input] mouse used={mouse_used} left_or_right_down={left_or_right_down}");
    }

    Ok(())
}

fn main() -> VMNLResult<()> {
    let context = Context::new()?;
    let mut window = Window::builder()
        .title("VMNL window events")
        .size(800, 600)
        .unset_configure_window_polling()
        .preferred_present_mode(PresentMode::Mailbox)
        .build(&context)?;

    configure_runtime_window(&mut window)?;
    let standard_cursor = Cursor::standard(StandardCursor::Arrow).build(&context)?;
    let (width, height) = (32_u32, 32_u32);
    let mut custom_pixels = vec![0_u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            if x == y || x + y == width - 1 {
                let offset = ((y * width + x) * 4) as usize;
                custom_pixels[offset..offset + 4].copy_from_slice(&[255, 0, 0, 255]);
            }
        }
    }

    let custom_cursor = Cursor::rgba8(width, height, &custom_pixels)
        .hotspot(16, 16)
        .hotspot_marker([0, 255, 0, 255])
        .build(&context)?;
    window.set_cursor(Some(&standard_cursor))?;
    let raw_mouse_motion_supported = context.is_raw_mouse_motion_supported();
    if raw_mouse_motion_supported {
        window.set_raw_mouse_motion(true)?;
    }
    print_monitor_summary(&window);

    window.set_time(0.0);
    window.post_empty_event();
    window.wait_events_timeout(0.001);

    println!("title: {}", window.get_title());
    println!(
        "size: {:?}, framebuffer: {:?}",
        window.get_size(),
        window.get_framebuffer_size()
    );
    println!("content_scale: {:?}", window.get_content_scale());
    println!(
        "position: {:?}, opacity: {}",
        window.get_position(),
        window.get_opacity()
    );
    println!(
        "timer: value={} frequency={}",
        window.get_timer_value(),
        window.get_timer_frequency()
    );
    println!(
        "ready={} visible={} focused={}",
        window.is_ready(),
        window.is_visible(),
        window.is_focused()
    );
    println!(
        "cursor: position={:?} hovered={} mode={:?} raw_supported={} raw_enabled={}",
        window.get_cursor_position(),
        window.is_cursor_hovered(),
        window.get_cursor_mode(),
        raw_mouse_motion_supported,
        window.is_raw_mouse_motion_enabled()
    );
    println!(
        "input modes: sticky_keys={} sticky_mouse_buttons={} lock_key_modifier_reporting={}",
        window.is_sticky_keys_enabled(),
        window.is_sticky_mouse_buttons_enabled(),
        window.is_lock_key_modifier_reporting_enabled()
    );
    println!(
        "keyboard delivery: key={} text={} legacy_text_with_modifiers={}",
        window.is_key_polling_enabled(),
        window.is_char_polling_enabled(),
        window.is_char_mods_polling_enabled(),
    );
    for key in [Key::A, Key::Semicolon, Key::Kp0, Key::Escape] {
        print_key_query(&context, key);
    }
    println!("keys: Escape close, F focus, I iconify, M maximize, R restore, H hide/show, C clear aspect");
    println!("keyboard mode: K toggle sticky keys");
    println!(
        "keyboard delivery: right click toggles key events, F9 text, F10 legacy modified text"
    );
    println!("cursor keys: N normal, V hidden, D disabled, G captured, P center");
    println!("cursor resources: S pointing hand, U custom RGBA8, O backend default");

    while window.is_open() {
        for event in window.poll_events() {
            print_event(&context, &event);
        }
        apply_keybinds(&mut window, &standard_cursor, &custom_cursor)?;
        // println!(
        //     "time={:.3} iconified={} maximized={} focused={}",
        //     window.get_time(),
        //     window.is_iconified(),
        //     window.is_maximized(),
        //     window.is_focused()
        // );
        // println!(
        //     "mode={:?}, hovered={}, position={:?}",
        //     window.get_cursor_mode(),
        //     window.is_cursor_hovered(),
        //     window.get_cursor_position(),
        // );
        window.render().submit()?;
    }

    window.unset_error_callback();
    Ok(())
}
