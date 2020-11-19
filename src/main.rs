use bracket_lib::prelude::*;

use text_rts::State;

fn main() -> BError {
    let tw = 24;
    let th = 24;
    let w = 1366 / tw;
    let h = 768 / th;

    let ctx = BTermBuilder::simple(w, h)?
        .with_tile_dimensions(tw, th)
        .with_advanced_input(true)
        .with_fps_cap(60.0)
        .with_fullscreen(true)
        .with_title("TextRTS")
        .build()?;
    let gs = State::new(w - 1, h);

    main_loop(ctx, gs)
}
