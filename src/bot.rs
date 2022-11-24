use crate::{state::State, Board, Move, PieceType};

fn material_value(board: &Board) -> i32 {
    let mut sum = 0;

    for piece in board.data.board.iter() {
        sum += PieceType::from_piece(piece.piece).value() * (piece.color() as i32);
    }
    sum
}

fn evaluate_position(board: &Board) -> i32 {
    material_value(board) * if board.data.white_turn { 1 } else { -1 }
}

fn search_moves(board: &mut Board, depth: i32, mut alpha: i32, beta: i32) -> (i32, Option<Move>) {
    let now = std::time::Instant::now();
    let mut _best_move = None;

    if depth == 0 {
        return (evaluate_position(board), None);
    }

    board.generate_moves();

    board.check_game_state();

    if board.move_generator.moves.len() == 0 {
        if board.state == State::WhiteIsMated || board.state == State::BlackIsMated {
            return if (board.state == State::WhiteIsMated) == board.data.white_turn {
                (i32::MIN + 1, None)
            } else {
                (i32::MAX, None)
            };
        }
        return (0, None);
    }

    for (i, _move) in board.move_generator.moves.clone().iter().enumerate() {
        board.make_move(&_move);

        let (evaluation, _) = search_moves(board, depth - 1, -beta, -alpha);

        board.undo_move();

        if -evaluation >= beta {
            return (beta, None);
        }
        if depth == 4 && -evaluation == 6 {
            print!("");
        }
        _best_move = if -evaluation > alpha { Some(_move.clone()) } else { _best_move };
        alpha = std::cmp::max(alpha, -evaluation);
    }
    if depth == 4 {
        print!("");
    }
    (alpha, _best_move)
}

pub(crate) fn make_move(board: &mut Board) -> Move {
    let depth = 5;
    let now = std::time::Instant::now();
    let (a, _move) = search_moves(board, depth, i32::MIN + 1, i32::MAX);

    println!("Took {} µs for depth {}", now.elapsed().as_micros(), depth);
    _move.unwrap()
}
