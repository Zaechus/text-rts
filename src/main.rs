use winit::event_loop::EventLoop;

use bracket_lib::prelude::*;

use text_rts::State;

fn main() -> BError {
    let tw = 24;
    let th = 24;

    let size = if let Some(monitor) = EventLoop::new().available_monitors().next() {
        (monitor.size().width / tw, monitor.size().height / th)
    } else {
        (1366 / tw, 768 / th)
    };

    let ctx = BTermBuilder::simple(size.0, size.1)?
        .with_tile_dimensions(tw, th)
        .with_advanced_input(true)
        .with_fps_cap(60.0)
        .with_fullscreen(true)
        .with_title("TextRTS")
        .build()?;
    let gs = State::new(size.0 - 1, size.1);

    main_loop(ctx, gs)
}
