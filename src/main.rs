use bracket_lib::prelude::*;

use text_rts::State;

fn main() {
    let ctx = BTermBuilder::simple(60, 30)
        .unwrap()
        .with_tile_dimensions(20, 20)
        .with_fullscreen(true)
        .with_advanced_input(true)
        .with_title("TextRTS")
        .build()
        .unwrap();
    let gs = State::new(60, 30);

    main_loop(ctx, gs).unwrap();
}
