use termion::color;

use crate::_move::Move;
use crate::change::Change;
use crate::move_generator::MoveGenerator;
use crate::piece::{Piece, PieceColor, PieceType};
use crate::piece_list::PieceList;
use crate::piece_moves::PieceMoves;
use crate::player::PlayerType;
use crate::position::Position;
use array_init::array_init;
use core::num;
use std::fmt;

pub(crate) struct Data {
    pub(crate) board: [Piece; 8 * 8],
    pub(crate) piece_moves: PieceMoves,
    pub(crate) white_turn: bool,
    pub(crate) piece_list: PieceList,
    pub(crate) not_able_to_castle: u8, // for white, black, for queenside and kingside respectively
    pub(crate) two_square_advance: u8, // intermittend position of the pawn while advancing two squares starting at 1
}

pub struct Board {
    pub(crate) move_generator: MoveGenerator,
    pub(crate) data: Data,
    players: [PlayerType; 2],
    ply: usize,
    fifty_move_counter: u8,
    changes: Vec<Change>,
}

impl Board {
    pub fn new(player1: PlayerType, player2: PlayerType) -> Board {
        let mut board = Board {
            data: Data {
                board: array_init(|_| Piece::empty()),
                piece_moves: PieceMoves::new(),
                white_turn: true,
                piece_list: PieceList::new(),
                not_able_to_castle: 0x0f,
                two_square_advance: 0,
            },
            move_generator: MoveGenerator::new(),
            players: [player1, player2],
            ply: 0,
            fifty_move_counter: 0,
            changes: Vec::new(),
        };
        board.populate();
        board.data.piece_list.populate(&board.data.board);
        board
    }

    fn populate(&mut self) -> () {
        for board_spot in 0..64 {
            let uuid = board_spot as u8;
            match board_spot {
                0 | 7 => self.data.board[board_spot].set(PieceType::Rook, PieceColor::White, uuid),
                56 | 63 => {
                    self.data.board[board_spot].set(PieceType::Rook, PieceColor::Black, 63 - uuid)
                }
                1 | 6 => self.data.board[board_spot].set(PieceType::Knight, PieceColor::White, uuid),
                57 | 62 => {
                    self.data.board[board_spot].set(PieceType::Knight, PieceColor::Black, 63 - uuid)
                }
                2 | 5 => self.data.board[board_spot].set(PieceType::Bishop, PieceColor::White, uuid),
                58 | 61 => {
                    self.data.board[board_spot].set(PieceType::Bishop, PieceColor::Black, 63 - uuid)
                }
                3 => self.data.board[board_spot].set(PieceType::Queen, PieceColor::White, uuid),
                59 => self.data.board[board_spot].set(PieceType::Queen, PieceColor::Black, 63 - uuid),
                4 => self.data.board[board_spot].set(PieceType::King, PieceColor::White, uuid),
                60 => self.data.board[board_spot].set(PieceType::King, PieceColor::Black, 63 - uuid),
                8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 => {
                    self.data.board[board_spot].set(PieceType::Pawn, PieceColor::White, uuid)
                }
                48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 => {
                    self.data.board[board_spot].set(PieceType::Pawn, PieceColor::Black, 63 - uuid)
                }
                _ => (),
            };
        }
    }

    fn make_move(&mut self, _move: &Move) {
        let mut piece = self.data.board[_move.start as usize].clone();

        self.data.board[_move.start as usize] = Piece::empty();

        let fifty_move_counter = self.fifty_move_counter;

        if !piece.is_pawn() && self.data.board[_move.end as usize].is_empty() {
            self.fifty_move_counter += 1;
        } else {
            self.fifty_move_counter = 0;
        }

        let captured_position = if _move.en_passant() {
            self.data.two_square_advance - 1
        } else {
            _move.end
        };

        let captured = self.data.board[captured_position as usize].clone();

        if _move.en_passant() {
            // en passant
            debug_assert!(self.data.board[_move.end as usize].is_empty());
            debug_assert!(piece.is_pawn());
            debug_assert!(self.data.two_square_advance != 0);
            let captured_pawn = &mut self.data.board[self.data.two_square_advance as usize - 1];
            self.data.piece_list.remove(captured_pawn);
            captured_pawn.set_empty();
        }

        if _move.promotion() != 0 {
            self.data.piece_list.remove(&piece);
            piece.promotion(_move.promotion());
            self.data.piece_list.add(&piece, _move.end);
        }

        if !captured.is_empty() {
            self.data.piece_list.remove(&captured);
        }

        let two_square_advance = self.data.two_square_advance;
        let not_able_to_castle = self.data.not_able_to_castle;

        if _move.castling() != 0 {
            let old_rook_pos =
                if _move.castling() == 1 { 0 } else { 7 } + ((_move.start as usize) / 8) * 8;
            let new_rook_pos =
                if _move.castling() == 1 { 3 } else { 5 } + ((_move.start as usize) / 8) * 8;

            let rook = self.data.board[old_rook_pos].clone();

            self.data.board[old_rook_pos] = Piece::empty();

            debug_assert!(self.data.board[new_rook_pos].is_empty());

            self.data.piece_list._move(&rook, new_rook_pos as u8);
            self.data.board[new_rook_pos] = rook;
        }

        if _move.two_square_advance() {
            self.data.two_square_advance = (_move.end + _move.start) / 2 + 1;
        } else {
            self.data.two_square_advance = 0;
        }

        if piece.is_king() {
            self.data.not_able_to_castle |= 0x03 << (2 * (!self.data.white_turn as u8));
        } else if piece.is_rook() && (_move.start == 0 || _move.start == 7) {
            // white castling
            self.data.not_able_to_castle |= 0x01 << ((_move.start != 0) as u8);
        } else if piece.is_rook() && (_move.start == 56 || _move.start == 63) {
            // black castling
            self.data.not_able_to_castle |= 0x04 << ((_move.start != 56) as u8);
        } else if captured.is_rook() && (_move.end == 0 || _move.end == 7) {
            // white castling
            self.data.not_able_to_castle |= 0x01 << ((_move.end != 0) as u8);
        } else if captured.is_rook() && (_move.end == 56 || _move.end == 63) {
            // black castling
            self.data.not_able_to_castle |= 0x04 << ((_move.end != 56) as u8);
        }

        self.changes.push(Change::new(
            _move.start,
            _move.end,
            captured,
            captured_position as u8,
            two_square_advance,
            fifty_move_counter,
            _move.promotion() != 0,
            not_able_to_castle,
            _move.castling(),
        ));

        self.data.piece_list._move(&piece, _move.end);
        self.data.board[_move.end as usize] = piece;
    }

    fn undo_move(&mut self) {
        let change = self.changes.pop().expect("No move to undo");
        self.data.not_able_to_castle = change.not_able_to_castle;
        self.data.two_square_advance = change.two_square_advance;
        self.fifty_move_counter = change.fifty_move_counter;
        self.data.board[change.start as usize] = self.data.board[change.end as usize].clone();

        self.data.board[change.captured_position as usize] = change.captured;

        if change.castling != 0 {
            let old_rook_pos =
                if change.castling == 1 { 3 } else { 5 } + ((change.start as usize) / 8) * 8;
            let new_rook_pos =
                if change.castling == 1 { 0 } else { 7 } + ((change.start as usize) / 8) * 8;

            let rook = self.data.board[old_rook_pos].clone();

            self.data.board[old_rook_pos] = Piece::empty();

            debug_assert!(self.data.board[new_rook_pos].is_empty());

            self.data.piece_list._move(&rook, new_rook_pos as u8);
            self.data.board[new_rook_pos] = rook;
        }

        if change.promotion {
            self.data.board[change.start as usize].promotion(1);
        }
    }

    #[inline(always)]
    pub(crate) fn pawn_on_original_position(&self, index: u8, color: i8) -> bool {
        Position::from_index(index).file == 1 - 5 * (color - 1) / 2
    }

    pub fn benchmark(mut self, depth: u8) -> (Self, u32, u128) {
        let now = std::time::Instant::now();

        if depth == 0 {
            return (self, 1, 0);
        }

        self.move_generator.generate_moves(&self.data);

        let mut num_positions = 0;

        for _move in self.move_generator.moves.clone() {
            self.make_move(&_move);

            self.ply += 1;
            self.data.white_turn = !self.data.white_turn;

            let _num_positions;
            (self, _num_positions, _) = self.benchmark(depth - 1);
            num_positions += _num_positions;

            self.undo_move();

            self.ply -= 1;
            self.data.white_turn = !self.data.white_turn;
        }

        (self, num_positions, now.elapsed().as_micros())
    }

    pub fn play_round(&mut self) -> bool {
        let now = std::time::Instant::now();

        self.move_generator.generate_moves(&self.data);

        if self.move_generator.moves.len() == 0 || self.fifty_move_counter == 50 {
            return true;
        }

        let _move = self.players[self.ply % 2]._move(self);

        self.make_move(&_move);

        self.ply += 1;
        self.data.white_turn = !self.data.white_turn;

        println!("{} µs", now.elapsed().as_micros());

        false
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string: String = String::new();

        string.push(' ');
        for i in 0..8 {
            string.push(' ');
            string.push((('a' as i8) + (i % 8)) as u8 as char);
        }
        string.push('\n');

        string.push_str(&color::Fg(color::Cyan).to_string());

        for (i, piece) in self.data.board.iter().enumerate() {
            if i % 8 == 0 {
                string.push_str(&color::Fg(color::White).to_string());
                string.push(char::from_digit((i / 8) as u32 + 1, 10).unwrap());
                string.push(' ');
                string.push_str(&color::Fg(color::Cyan).to_string());
            }

            string.push(piece.as_char());

            if i % 8 == 7 {
                string.push_str(&color::Fg(color::White).to_string());
                string.push(' ');
                string.push(char::from_digit((i / 8) as u32 + 1, 10).unwrap());
                string.push('\n');
                string.push_str(&color::Fg(color::Cyan).to_string());
            } else if i % 8 != 7 {
                string.push(' ');
            }
        }

        string.push_str(&color::Fg(color::White).to_string());

        string.push(' ');
        for i in 0..8 {
            string.push(' ');
            string.push((('a' as i8) + (i % 8)) as u8 as char);
        }

        write!(f, "{}", string)
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string: String = String::new();

        string.push(' ');
        for i in 0..8 {
            string.push(' ');
            string.push((('a' as i8) + (i % 8)) as u8 as char);
        }
        string.push('\n');

        string.push_str(&color::Fg(color::LightWhite).to_string());

        for (i, piece) in self.data.board.iter().enumerate() {
            if i % 8 == 0 {
                string.push_str(&color::Fg(color::White).to_string());
                string.push(char::from_digit((i / 8) as u32 + 1, 10).unwrap());
                string.push(' ');
                string.push_str(&color::Fg(color::LightWhite).to_string());
            }

            if self
                .move_generator
                .moves
                .iter()
                .any(|p| p.end as usize == i)
            {
                string.push_str(&color::Fg(color::Cyan).to_string());
            }

            string.push(piece.as_char());

            string.push_str(&color::Fg(color::LightWhite).to_string());

            if i % 8 == 7 {
                string.push_str(&color::Fg(color::White).to_string());
                string.push(' ');
                string.push(char::from_digit((i / 8) as u32 + 1, 10).unwrap());

                string.push('\t');

                for safe in self.move_generator.attacking_rays[(i / 8 * 8)..(i / 8 * 8 + 8)].iter()
                {
                    string.push(if *safe { '▆' } else { '.' });
                    string.push(' ');
                }

                string.push('\n');
                string.push_str(&color::Fg(color::Cyan).to_string());
            } else if i % 8 != 7 {
                string.push(' ');
            }
        }

        string.push_str(&color::Fg(color::White).to_string());

        string.push(' ');
        for i in 0..8 {
            string.push(' ');
            string.push((('a' as i8) + (i % 8)) as u8 as char);
        }

        write!(f, "{}", string)
    }
}
