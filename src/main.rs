use bracket_lib::prelude::*;

use text_rts::State;

fn main() {
    let ctx = BTermBuilder::simple(80, 40)
        .with_tile_dimensions(16, 16)
        .with_title("TextRTS")
        .build();
    let gs = State::new(80, 40);

    main_loop(ctx, gs);
}
