use chess::{board::PerftPositions, Board, PlayerType};
use iced::executor;
use iced::widget::{button, checkbox, container, text, Column, Row};
use iced::{Alignment, Command, Element, Length, Sandbox, Settings, Subscription};

fn main() -> iced::Result {
    Chess::run(Settings::default())

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
        let test: Vec<i8> = (0..8).collect();
        let board = Column::with_children(
            test.iter()
                .map(|file: &i8| {
                    Row::with_children(
                        test.iter()
                            .map(|rank: &i8| {
                                button("w")
                                    .on_press(Message::HighlightMessage)
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                            })
                            .map(Element::from)
                            .collect(),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                })
                .map(Element::from)
                .collect(),
        )
        .width(Length::Fill)
        .height(Length::Fill);
        let content = Column::new().height(Length::Fill).push(board);
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn update(&mut self, message: Message) {}
}
