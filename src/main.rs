use chess::{Board, PlayerType, board::PerftPositions};

fn main() {
    let mut board = Board::new(PlayerType::HumanPlayer, PlayerType::HumanPlayer);

    board.load_fen(PerftPositions::POSITION3);
    board.play_move("b4e4").unwrap();
    board.play_move("h4g5").unwrap();
    board.play_move("a5b4").unwrap();
    board.play_move("c7c5").unwrap();
    
    for i in 1..=1 {
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
