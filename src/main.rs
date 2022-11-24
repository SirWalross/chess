use chess::{Board, PlayerType, board::PerftPositions};

fn main() {
    let mut board = Board::new(PlayerType::Bot, PlayerType::HumanPlayer);
    board.load_fen(PerftPositions::POSITION2);

    loop {
        println!("{:?}", board);
        if board.play_round() {
            println!("{}", board.state);
            break;
        }
        print!("");
    }
}
