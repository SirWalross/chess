#![allow(dead_code, unused)]

pub mod _move;
pub mod board;
pub mod change;
pub mod move_generator;
pub mod piece;
pub mod piece_list;
pub mod piece_moves;
pub mod player;
pub mod position;
pub mod state;

pub use _move::Move;
pub use board::Board;
pub use piece::{Piece, PieceColor, PieceType};
pub use player::PlayerType;

#[cfg(test)]
mod tests {
    use crate::board::PerftPositions;

    use super::*;

    fn run_test(mut board: Board, results: &[usize], depth: u8) {
        for i in 1..=depth {
            let (_board, num_positions, duration) = board.benchmark(i, i);
            board = _board;

            assert_eq!(num_positions, results[i as usize - 1]);
        }
    }

    #[test]
    fn position1() {
        let results = [20, 400, 8902, 197281, 4865609, 119060324, 3195901860];
        let mut board = Board::new(PlayerType::HumanPlayer, PlayerType::HumanPlayer);
        run_test(board, &results, 7);
    }

    #[test]
    fn position2() {
        let results = [48, 2039, 97862, 4085603, 193690690, 8031647685];
        let mut board = Board::new(PlayerType::HumanPlayer, PlayerType::HumanPlayer);
        board.load_fen(PerftPositions::POSITION2);
        run_test(board, &results, 6);
    }

    #[test]
    fn position3() {
        let results = [14, 191, 2812, 43238, 674624, 11030083];
        let mut board = Board::new(PlayerType::HumanPlayer, PlayerType::HumanPlayer);
        board.load_fen(PerftPositions::POSITION3);
        run_test(board, &results, 6);
    }

    #[test]
    fn position4() {
        let results = [6, 264, 9467, 422333, 15833292, 706045033];
        let mut board = Board::new(PlayerType::HumanPlayer, PlayerType::HumanPlayer);
        board.load_fen(PerftPositions::POSITION4);
        run_test(board, &results, 6);
    }

    #[test]
    fn position5() {
        let results = [44, 1486, 62379, 2103487, 89941194];
        let mut board = Board::new(PlayerType::HumanPlayer, PlayerType::HumanPlayer);
        board.load_fen(PerftPositions::POSITION5);
        run_test(board, &results, 5);
    }

    #[test]
    fn position6() {
        let results = [46, 2079, 89890, 3894594, 164075551, 6923051137];
        let mut board = Board::new(PlayerType::HumanPlayer, PlayerType::HumanPlayer);
        board.load_fen(PerftPositions::POSITION6);
        run_test(board, &results, 6);
    }
}
