use chess::{board::PerftPositions, Board, PlayerType};
use iced::alignment::{Horizontal, Vertical};
use iced::executor;
use iced::theme::{self, Button, Theme};
use iced::widget::button::{Appearance, StyleSheet};
use iced::widget::{button, checkbox, container, text, Column, Row};
use iced::window;
use iced::{Alignment, Color, Command, Element, Length, Sandbox, Settings, Subscription};

fn main() -> iced::Result {
    Chess::run(Settings {
        window: window::Settings {
            size: (600, 570),
            resizable: false,
            ..window::Settings::default()
        },
        ..Settings::default()
    })

    // loop {
    //     println!("{:?}", board);
    //     if board.play_round() {
    //         println!("{}", board.state);
    //         break;
    //     }
    //     print!("");
    // }
}

struct Chess {
    board: Board,
}

#[derive(Debug, Clone, Copy)]
struct Position {
    rank: i8,
    file: i8,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    HighlightMessage,
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

impl Sandbox for Chess {
    type Message = Message;

    fn new() -> Self {
        Self {
            board: Board::new(PlayerType::Bot, PlayerType::HumanPlayer),
        }
    }

    fn title(&self) -> String {
        String::from("Events - Iced")
    }

    fn view(&self) -> Element<Message> {
        let test: Vec<u8> = (0..8).collect();
        let board = Column::with_children(
            test.iter()
                .map(|file: &u8| {
                    Row::with_children(
                        test.iter()
                            .map(|rank: &u8| {
                                container(
                                    button(
                                        text(
                                            self.board
                                                .get_piece_at_position(*file, *rank)
                                                .as_char(),
                                        )
                                        .horizontal_alignment(Horizontal::Center)
                                        .vertical_alignment(Vertical::Center)
                                        .width(Length::Fill)
                                        .height(Length::Fill)
                                        .size(32),
                                    )
                                    .on_press(Message::HighlightMessage)
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                                    .style(
                                        if (file + rank) % 2 == 0 {
                                            Button::Primary
                                        } else {
                                            Button::Secondary
                                        },
                                    ),
                                )
                                //.padding(10)
                                .width(Length::Fill)
                                .height(Length::Fill)
                            })
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

    fn update(&mut self, message: Message) {}

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
