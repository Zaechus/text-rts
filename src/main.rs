use bracket_lib::prelude::*;

use text_rts::State;

fn main() -> BError {
    let ctx = BTermBuilder::simple80x50()
        .with_advanced_input(true)
        .with_automatic_console_resize(true)
        .with_fps_cap(60.0)
        .with_title("TextRTS")
        .build()?;
    let gs = State::new(80, 50);

    main_loop(ctx, gs)
}
