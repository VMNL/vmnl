use vmnl::Window;

fn main()
{
    let mut win = Window::new()
    .expect("Failed");

    while win.is_open() {
        win.poll_event();
    }
}
