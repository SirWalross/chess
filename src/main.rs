use chess::position::Position;
use chess::{board::PerftPositions, Board, PlayerType};
use iced::alignment::{Horizontal, Vertical};
use iced::theme::{self, Button, Theme};
use iced::widget::button::{Appearance, StyleSheet};
use iced::widget::{button, checkbox, container, text, Column, Container, Row};
use iced::{executor, Font};
use iced::{window, Renderer};
use iced::{Alignment, Color, Command, Element, Length, Sandbox, Settings, Subscription};

fn main() -> iced::Result {
    Chess::run(Settings {
        window: window::Settings {
            size: (600, 565),
            resizable: false,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

struct Chess {
    board: Board,
    active_piece: Option<Position>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    HighlightMessage(Position),
    DragMessage,
    DropMessage,
}

// struct DarkButtonTheme {

// }

// impl StyleSheet for DarkButtonTheme {
//     type Style = Button;

//     fn active(&self, style: &Self::Style) -> Appearance {

//     }
// }

const MESLO_LG_FONT: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/Meslo LG L DZ Regular Nerd Font Complete Mono.ttf"),
};

impl Chess {
    fn create_button(&self, pos: Position) -> Container<Message, Renderer> {
        let white_text_color: Color = Color::from([0.9, 0.9, 0.9]);
        let black_text_color: Color = Color::from([0.1, 0.1, 0.1]);

        let piece = self.board.get_piece_at_position(pos);

        let mut chess_button = button(
            text(piece.as_unicode_char_abs())
                .horizontal_alignment(Horizontal::Center)
                .vertical_alignment(Vertical::Center)
                .width(Length::Fill)
                .height(Length::Fill)
                .size(150)
                .font(MESLO_LG_FONT)
                .style(if piece.is_white() {
                    white_text_color
                } else {
                    black_text_color
                }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(if (pos.file + pos.rank) % 2 == 0 {
            Button::Primary
        } else {
            Button::Secondary
        });

        if self.active_piece.is_some() {
            // hightlight moves the piece is able to make
            let old_pos = self.active_piece.unwrap();
            if self.board.piece_able_to_move_to_pos(old_pos, pos) {
                chess_button = chess_button.on_press(Message::HighlightMessage(pos));
            }
        } else {
            // hightlight all moveable pieces
            if !piece.is_empty() && self.board.piece_able_to_move(pos) {
                chess_button = chess_button.on_press(Message::HighlightMessage(pos));
            }
        }

        container(chess_button)
            //.padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
    }
}

impl Sandbox for Chess {
    type Message = Message;

    fn new() -> Self {
        let mut board = Board::new(PlayerType::HumanPlayer, PlayerType::Bot);
        board.generate_moves();
        if !board.human_turn() {
            board.play_ply();
        }
        board.generate_moves();
        Self {
            board: board,
            active_piece: None,
        }
    }

    fn title(&self) -> String {
        String::from("Events - Iced")
    }

    fn view(&self) -> Element<Message> {
        let test: Vec<i8> = (0..8).collect();
        let board = Column::with_children(
            test.iter()
                .map(|file: &i8| {
                    Row::with_children(
                        test.iter()
                            .map(|rank: &i8| self.create_button(Position::new(7 - *file, *rank)))
                            .map(Element::from)
                            .collect(),
                    )
                    .height(Length::Fill)
                    .align_items(Alignment::Center)
                })
                .map(Element::from)
                .collect(),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(Alignment::Center);
        let content = Column::new().height(Length::Fill).push(board);
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::HighlightMessage(pos) => {
                if self.active_piece.is_some() && pos != self.active_piece.unwrap() {
                    // move piece
                    self.board.play_human_ply(self.active_piece.unwrap(), pos);
                    if !self.board.human_turn() {
                        self.board.play_ply();
                    }
                    self.board.generate_moves();
                    self.active_piece = None;
                } else if self.active_piece.is_some() {
                    // unhighlight piece
                    self.active_piece = None;
                } else {
                    // highlight button
                    self.active_piece = Some(pos);
                }
            }
            _ => {}
        };
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
