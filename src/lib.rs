use std::fs;

use chess::{BitBoard, Board, ChessMove, Square};
use vizia::{image, prelude::*};

enum ChessEvent {
    TileClicked(i32),
}

#[derive(Lens)]
pub struct Chess {
    board: Board,
    images: [String; 64],
    selected: Option<(i32, bool)>,
}

impl View for Chess {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|chess_event, meta| match chess_event {
            ChessEvent::TileClicked(pos) => {
                let pos = if self.board.side_to_move() == chess::Color::White {
                    *pos
                } else {
                    63 - pos
                };
                let pos_board = BitBoard::from_square(unsafe { Square::new(pos as u8) });
                if let Some((selected_pos, flipped)) = self.selected {
                    if pos == selected_pos {
                        self.selected = None
                    } else if pos_board
                        == self.board.color_combined(self.board.side_to_move()) & pos_board
                    {
                        self.selected = Some((pos, flipped));
                    } else {
                        let to = pos_board.to_square();
                        let from = unsafe { Square::new(selected_pos as u8) };
                        let new_move = ChessMove::new(from, to, None);
                        println!("Move: {new_move}");
                        if self.board.legal(new_move) {
                            println!("Move is legal!");
                            self.board = self.board.make_move_new(new_move);
                            self.images = get_paths_from_pos(&self.board);
                            if self.board.side_to_move() == chess::Color::Black {
                                self.images.reverse();
                            }
                            self.selected = None;
                        }
                    }
                } else {
                    if pos_board == self.board.color_combined(self.board.side_to_move()) & pos_board
                    {
                        self.selected =
                            Some((pos, self.board.side_to_move() == chess::Color::Black));
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
            VStack::new(cx, |cx| {
                // Game status
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
                // Board
                VStack::new(cx, |cx| {
                    for y in 0..8 {
                        // Row
                        HStack::new(cx, |cx| {
                            for x in 0..8 {
                                // Square
                                Element::new(cx)
                                    .height(Stretch(1.0))
                                    .width(Stretch(0.125))
                                    .class(&format!("tile-{}", (x + y) % 2))
                                    .class("tile")
                                    .on_press(move |cx| {
                                        cx.emit(ChessEvent::TileClicked((7 - y) * 8 + x))
                                    })
                                    .image(
                                        Chess::images
                                            .map(move |value| value[(y * 8 + x) as usize].clone()),
                                    )
                                    .checked(Chess::selected.map(move |value| match value {
                                        Some((selected_pos, flipped)) if *flipped => {
                                            *selected_pos == 63 - ((7 - y) * 8 + x)
                                        }
                                        Some((selected_pos, _)) => *selected_pos == (7 - y) * 8 + x,
                                        None => false,
                                    }));
                            }
                        })
                        .width(Stretch(1.0))
                        .height(Stretch(0.125));
                    }
                })
                .class("board")
                .size(Pixels(500.0));
            })
            .size(Auto);
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
