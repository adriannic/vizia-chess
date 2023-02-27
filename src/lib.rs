use std::fs;

use chess::{BitBoard, Board, ChessMove, Square};
use vizia::{image, prelude::*};

enum ChessEvent {
    TileClicked(i32, i32),
}

#[derive(Lens)]
pub struct Chess {
    board: Board,
    images: [String; 64],
    selected: Option<(i32, i32)>,
}

impl View for Chess {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|chess_event, meta| match chess_event {
            ChessEvent::TileClicked(x, y) => {
                println!("Tile on position ({x}, {y}) clicked!");
                let pos = BitBoard::from_square(unsafe { Square::new((y * 8 + x) as u8) });
                if let Some((i, j)) = self.selected {
                    if (x, y) == (&i, &j) {
                        self.selected = None
                    } else if pos == self.board.color_combined(self.board.side_to_move()) & pos {
                        self.selected = Some((*x, *y));
                    } else {
                        let to = pos.to_square();
                        let from = unsafe { Square::new((j * 8 + i) as u8) };
                        let new_move = ChessMove::new(from, to, None);
                        println!("Move: {new_move}");
                        if self.board.legal(new_move) {
                            println!("Move is legal!");
                            self.board = self.board.make_move_new(new_move);
                            self.images = get_paths_from_pos(&self.board);
                            self.selected = None;
                        }
                    }
                } else {
                    if pos == self.board.color_combined(self.board.side_to_move()) & pos {
                        self.selected = Some((*x, *y));
                    }
                }
                meta.consume();
            }
        });
    }
}

impl Chess {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {
            board: Board::default(),
            images: get_paths_from_pos(&Board::default()),
            selected: None,
        }
        .build(cx, |cx| {
            cx.add_stylesheet("./assets/stylesheets/styles.css")
                .expect("Stylesheet should exist");
            cx.set_image_loader(|cx, path| {
                let path = path.to_string();
                if path == "" {
                    cx.load_image(
                        path.to_owned(),
                        image::load_from_memory(include_bytes!("../assets/sprites/empty.png"))
                            .expect("File should exist"),
                        vizia::resource::ImageRetentionPolicy::DropWhenUnusedForOneFrame,
                    );
                } else {
                    let image = fs::read(format!("./assets/sprites/{}.png", path))
                        .expect("File should exist");
                    cx.load_image(
                        path.to_owned(),
                        image::load_from_memory(&image).expect("File should exist"),
                        vizia::resource::ImageRetentionPolicy::DropWhenUnusedForOneFrame,
                    );
                }
            });
            HStack::new(cx, |cx| {
                Label::new(
                    cx,
                    Chess::board.map(|value| format!("{:?}", value.status())),
                )
                .height(Auto)
                .width(Stretch(1.0))
                .color(Color::white());
                Label::new(
                    cx,
                    Chess::board.map(|value| format!("{:?}", value.side_to_move())),
                )
                .size(Auto)
                .color(Color::white());
            })
            .height(Auto)
            .width(Stretch(1.0));
            VStack::new(cx, |cx| {
                for y in 0..8 {
                    HStack::new(cx, |cx| {
                        for x in 0..8 {
                            Element::new(cx)
                                .height(Stretch(1.0))
                                .width(Stretch(0.125))
                                .class(&format!("tile-{}", (x + y) % 2))
                                .class("tile")
                                .on_press(move |cx| cx.emit(ChessEvent::TileClicked(x, 7 - y)))
                                .image(
                                    Chess::images
                                        .map(move |value| value[(y * 8 + x) as usize].clone()),
                                )
                                .checked(Chess::selected.map(move |value| match value {
                                    Some((i, j)) => (&x, &(7 - y)) == (i, j),
                                    None => false,
                                }));
                        }
                    })
                    .width(Stretch(1.0))
                    .height(Stretch(0.125));
                }
            })
            .class("board");
        })
    }
}

fn get_paths_from_pos(board: &Board) -> [String; 64] {
    board
        .to_string()
        .trim()
        .split_whitespace()
        .next()
        .expect("There should be at least one element")
        .chars()
        .filter(|c| *c != '/')
        .flat_map(|c| {
            if "12345678".contains(c) {
                vec![String::new(); c.to_digit(10).expect("Should be a digit") as usize]
            } else {
                vec![String::from(c)]
            }
        })
        .collect::<Vec<String>>()
        .try_into()
        .expect("Vec should be of length 64")
}
