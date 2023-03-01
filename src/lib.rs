use std::fs;

use chess::{BitBoard, Board, ChessMove, Square};
use vizia::{image, prelude::*};

enum ChessEvent {
    TileClicked(i32),
    ToggleFlipping,
    Reset,
}

#[derive(Lens)]
pub struct Chess {
    board: Board,
    images: [String; 64],
    selected: Option<(i32, bool)>,
    on_check: Option<(i32, bool)>,
    should_flip: bool,
}

impl View for Chess {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|chess_event, meta| match chess_event {
            ChessEvent::TileClicked(pos) => {
                let pos = if self.should_flip && self.board.side_to_move() == chess::Color::Black {
                    63 - pos
                } else {
                    *pos
                };
                let pos_board = BitBoard::from_square(unsafe { Square::new(pos as u8) });
                if let Some((selected_pos, flipped)) = self.selected {
                    let flipped = self.should_flip && flipped;
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
                        if self.board.legal(new_move) {
                            self.board = self.board.make_move_new(new_move);
                            self.update_board();
                            self.selected = None;
                        }
                    }
                } else {
                    if pos_board == self.board.color_combined(self.board.side_to_move()) & pos_board
                    {
                        self.selected = Some((
                            pos,
                            self.should_flip && self.board.side_to_move() == chess::Color::Black,
                        ));
                    }
                }
                meta.consume();
            }
            ChessEvent::ToggleFlipping => {
                self.should_flip ^= true;
                self.update_board();
                meta.consume();
            }
            ChessEvent::Reset => {
                self.board = Board::default();
                self.update_board();
                self.selected = None;
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
            on_check: None,
            should_flip: true,
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
                    );
                    Label::new(
                        cx,
                        Chess::board.map(|value| format!("{:?}", value.side_to_move())),
                    );
                })
                .class("board-state");
                // Board
                VStack::new(cx, |cx| {
                    for y in 0..8 {
                        // Row
                        HStack::new(cx, |cx| {
                            for x in 0..8 {
                                // Square
                                Element::new(cx)
                                    .class(&format!("tile-{}", (x + y) % 2))
                                    .toggle_class(
                                        "on-check",
                                        Chess::on_check.map(move |value| match value {
                                            Some((checked_pos, flipped)) if *flipped => {
                                                *checked_pos == 63 - ((7 - y) * 8 + x)
                                            }
                                            Some((checked_pos, _)) => {
                                                *checked_pos == (7 - y) * 8 + x
                                            }
                                            None => false,
                                        }),
                                    )
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
                        .class("board-row");
                    }
                })
                .class("board");

                HStack::new(cx, |cx| {
                    Button::new(
                        cx,
                        |cx| cx.emit(ChessEvent::Reset),
                        |cx| Label::new(cx, "Reset").color(Color::white()),
                    );
                    Checkbox::new(cx, Chess::should_flip)
                        .on_toggle(|cx| cx.emit(ChessEvent::ToggleFlipping));
                    Label::new(cx, "Board flipping");
                })
                .class("board-settings");
            })
            .class("board-frame");
        })
    }

    fn update_board(&mut self) {
        self.images = get_paths_from_pos(&self.board);
        if self.should_flip && self.board.side_to_move() == chess::Color::Black {
            self.images.reverse();
        }
        if self.board.checkers().clone().next().is_some() {
            let king_pos = self.board.king_square(self.board.side_to_move()).to_int();
            self.on_check = Some((
                king_pos as i32,
                self.should_flip && self.board.side_to_move() == chess::Color::Black,
            ));
        } else {
            self.on_check = None;
        }
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
