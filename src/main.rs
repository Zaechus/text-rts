use bracket_lib::prelude::*;

use text_rts::State;

fn main() -> BError {
    let ctx = BTermBuilder::simple(60, 30)?
        .with_tile_dimensions(20, 20)
        .with_fullscreen(true)
        .with_advanced_input(true)
        .with_title("TextRTS")
        .build()?;
    let gs = State::new(60, 30);

    main_loop(ctx, gs)
}
