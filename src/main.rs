use chess::{Board, PlayerType};

fn main() {
    let mut board = Board::new(PlayerType::HumanPlayer, PlayerType::HumanPlayer);

    for i in 4..=4 {
        let (_board, num_positions, duration) = board.benchmark(i, i);
        board = _board;
        // board.reset();
        println!("Took {} µs for depth {} with {} moves", duration, i, num_positions);
    }
    // board.load_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    // loop {
    //     board.generate_moves();
    //     println!("{:?}", board);
    //     if board.play_round() {
    //         println!("{}", board.state);
    //         break;
    //     }
    //     print!("");
    // }

}
