use chess::{Board, PlayerType};

fn main() {
    let mut board = Board::new(PlayerType::HumanPlayer, PlayerType::HumanPlayer);

    for i in 1..=4 {
        let (_board, num_positions, duration) = board.benchmark(i);
        board = _board;
        println!("Took {} ms for depth {} with {} moves", duration / 1000, i, num_positions);
    }

}
