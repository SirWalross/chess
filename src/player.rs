use crate::{Board, Move, position::Position};
use rand::seq::SliceRandom;
use std::io::Write;

pub enum PlayerType {
    HumanPlayer,
    Bot,
}

impl PlayerType {
    pub fn _move(&self, board: &Board) -> Move {
        match self {
            Self::HumanPlayer => loop {
                print!("> ");
                std::io::stdout().flush().unwrap();
                let mut input = String::with_capacity(4);
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("error unable to read input");

                let chars: Vec<char> = input.chars().collect();

                if (chars.len() != 5 && chars.len() != 6)
                    || !chars[0].is_ascii_alphabetic()
                    || !chars[2].is_ascii_alphabetic()
                    || !chars[1].is_ascii_digit()
                    || !chars[3].is_ascii_digit()
                {
                    println!(
                        "Invalid syntax, use <start><end>[<promotion piece>], ex.: a2a3, or a7a8q"
                    );
                    continue;
                }

                if chars.len() == 6 && !['q', 'r', 'b', 'n'].iter().any(|c| *c == chars[4].to_ascii_lowercase()) {
                    println!("Invalid <promition piece>, only 'q', 'r', 'b' and 'n' are allowed.");
                    continue;
                }

                let start = Position::new(
                    chars[1].to_digit(10).unwrap() as i8 - 1,
                    (chars[0].to_ascii_lowercase() as u32 - 'a' as u32) as i8,
                );

                let end = Position::new(
                    chars[3].to_digit(10).unwrap() as i8 - 1,
                    (chars[2].to_ascii_lowercase() as u32 - 'a' as u32) as i8,
                );

                let promotion: u8 = if chars.len() == 6 {
                    ['q', 'r', 'b', 'n'].iter().position(|c| *c == chars[4].to_ascii_lowercase()).unwrap() as u8 + 1
                } else {
                    0
                };

                if !start.in_bounds(0, 0) {
                    println!("Invalid start position");
                    continue;
                }

                if !end.in_bounds(0, 0) {
                    println!("Invalid end position");
                    continue;
                }

                let mut _move = Move::from_positions(start, end, 0, promotion, false, false);
                let move_index = board.move_generator.moves.iter().position(|m| {
                    m.start == start.index() as u8
                        && m.end == end.index() as u8
                        && m.promotion() == _move.promotion()
                });
                if move_index.is_none() {
                    println!("Invalid move");
                    continue;
                }

                _move = board.move_generator.moves[move_index.unwrap()].clone();
                return _move;
            },
            Self::Bot => {
                let _move = board
                    .move_generator
                    .moves
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .clone();
                println!("Player {:?} made the move {}", board.data.white_turn, _move);
                _move
            }
        }
    }
}
