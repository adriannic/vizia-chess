use vizia::prelude::*;
use vizia_chess::Chess;

fn main() {
    Application::new(|cx| {
        Chess::new(cx).space(Stretch(0.125));
    })
    .run();
}
