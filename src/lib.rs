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
    use super::*;

    #[test]
    fn position1() {
        let results = [20, 400, 8902, 197281, 4865609, 119060324];
        let mut board = Board::new(PlayerType::HumanPlayer, PlayerType::HumanPlayer);
        for i in 1..=6 {
            let (_board, num_positions, duration) = board.benchmark(i, i);
            board = _board;
            board.reset();

            assert_eq!(num_positions, results[i as usize - 1]);
        }
    }

    // #[test]
    // fn position2() {
    //     let results = [48, 2039, 97862, 4085603, 193690690];
    //     let mut board = Board::new(PlayerType::HumanPlayer, PlayerType::HumanPlayer);
    //     board.load_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
    //     for i in 1..=5 {
    //         let (_board, num_positions, duration) = board.benchmark(i);
    //         board = _board;
    //         board.reset();
    //         board.load_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");

    //         assert_eq!(num_positions, results[i as usize - 1]);
    //     }
    // }
}
