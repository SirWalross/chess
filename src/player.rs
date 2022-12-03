use std::io::Write;

use crate::bot;
use crate::{position::Position, Board, Move};
use rand::seq::SliceRandom;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PlayerType {
    HumanPlayer,
    Bot,
}

impl PlayerType {
    pub fn _move(&self, board: &mut Board) -> Move {
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

                if chars.len() == 6
                    && !['q', 'r', 'b', 'n']
                        .iter()
                        .any(|c| *c == chars[4].to_ascii_lowercase())
                {
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
                    ['q', 'r', 'b', 'n']
                        .iter()
                        .position(|c| *c == chars[4].to_ascii_lowercase())
                        .unwrap() as u8
                        + 1
                } else {
                    0
                };

                if !start.move_in_bounds(0, 0) {
                    println!("Invalid start position");
                    continue;
                }

                if !end.move_in_bounds(0, 0) {
                    println!("Invalid end position");
                    continue;
                }

                let _move = Move::from_positions(start, end, 0, promotion, false, false);
                let _move = board.move_generator.get_move(start, end, promotion);
                if _move.is_none() {
                    println!("Invalid move");
                    continue;
                }

                return _move.unwrap();
            },
            Self::Bot => bot::make_move(board),
        }
    }
}
