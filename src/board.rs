use termion::color;

use crate::_move::{Move, MoveFlags};
use crate::change::Change;
use crate::move_generator::MoveGenerator;
use crate::piece::{Piece, PieceColor, PieceType};
use crate::piece_list::PieceList;
use crate::piece_moves::PieceMoves;
use crate::player::PlayerType;
use crate::position::Position;
use crate::state::State;
use array_init::array_init;
use core::num;
use std::fmt;

#[allow(non_snake_case)]
pub mod PerftPositions {
    pub const POSITION1: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    pub const POSITION2: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ";
    pub const POSITION3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -";
    pub const POSITION4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    pub const POSITION5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    pub const POSITION6: &str =
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
}

pub(crate) struct Data {
    pub(crate) board: [Piece; 8 * 8],
    pub(crate) piece_moves: PieceMoves,
    pub(crate) white_turn: bool,
    pub(crate) piece_list: PieceList,
    pub(crate) not_able_to_castle: u8, // bit 0 white queenside, bit 1 white kingside, bit 2 black queenside, bit 3 black kingside
    pub(crate) two_square_advance: u8, // intermittend position of the pawn while advancing two squares starting at 1
}

pub struct Board {
    pub(crate) move_generator: MoveGenerator,
    pub(crate) data: Data,
    pub state: State,
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
            state: State::Playing,
            move_generator: MoveGenerator::new(),
            players: [player1, player2],
            ply: 0,
            fifty_move_counter: 0,
            changes: Vec::new(),
        };
        board.load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        board
    }

    pub fn load_fen(&mut self, fen: &str) {
        self.state = State::Playing;
        self.data.not_able_to_castle = 0x0f;

        for piece in &mut self.data.board {
            piece.set_empty()
        }

        let sections: Vec<&str> = fen.split(' ').collect();
        let mut file = 7;
        let mut rank = 0;
        let mut white_uuid = 0;
        let mut black_uuid = 0;

        for c in sections[0].chars() {
            if c == '/' {
                rank = 0;
                file -= 1;
            } else if c.is_ascii_digit() {
                rank += c.to_digit(10).unwrap() as usize;
            } else {
                let _type = PieceType::from_char(c.to_ascii_lowercase());
                if c.is_ascii_uppercase() {
                    self.data.board[rank + 8 * file].set(_type, PieceColor::White, white_uuid);
                    white_uuid += 1;
                } else {
                    self.data.board[rank + 8 * file].set(_type, PieceColor::Black, black_uuid);
                    black_uuid += 1;
                }
                rank += 1;
            }
        }

        self.data.white_turn = sections[1].chars().next().unwrap() == 'w';

        let castling_rights = if sections.len() > 2 {
            sections[2]
        } else {
            "KQkq"
        };

        self.data.not_able_to_castle ^= if castling_rights.contains('Q') {
            1 << 0
        } else {
            0
        };
        self.data.not_able_to_castle ^= if castling_rights.contains('K') {
            1 << 1
        } else {
            0
        };
        self.data.not_able_to_castle ^= if castling_rights.contains('q') {
            1 << 2
        } else {
            0
        };
        self.data.not_able_to_castle ^= if castling_rights.contains('k') {
            1 << 3
        } else {
            0
        };

        if sections.len() > 3 {
            if sections[3].len() > 1 {
                // contains position
                let rank = (sections[3].chars().next().unwrap().to_ascii_lowercase() as u32
                    - 'a' as u32) as u8;
                let file = sections[3].chars().next().unwrap().to_digit(10).unwrap() as u8 - 1;
                self.data.two_square_advance = rank + file * 8;
            }
        }

        if sections.len() > 4 {
            self.fifty_move_counter = sections[4].parse::<u8>().unwrap_or(0);
        }

        self.data.piece_list.populate(&self.data.board);
    }

    pub(crate) fn make_move(&mut self, _move: &Move) {
        let mut piece = self.data.board[_move.start as usize].clone();

        self.data.board[_move.start as usize] = Piece::empty();

        let fifty_move_counter = self.fifty_move_counter;

        if !piece.is_pawn() && self.data.board[_move.end as usize].is_empty() {
            self.fifty_move_counter += 1;
        } else {
            self.fifty_move_counter = 0;
        }

        let captured_position = if _move.en_passant() {
            (self.data.two_square_advance as i8 - 8 * piece.color()) as u8
        } else {
            _move.end
        };

        let mut captured = self.data.board[captured_position as usize].clone();

        debug_assert!(!captured.is_king());

        if _move.en_passant() {
            // en passant
            debug_assert!(self.data.board[_move.end as usize].is_empty());
            debug_assert!(piece.is_pawn());
            debug_assert!(self.data.two_square_advance != 0);
            self.data.piece_list.remove(&captured);
            self.data.board[captured_position as usize].set_empty();
        } else if !captured.is_empty() {
            self.data.piece_list.remove(&captured);
        }

        if _move.promotion() != 0 {
            self.data.piece_list.remove(&piece);
            piece.promotion(_move.promotion());
            self.data.piece_list.add(&piece, _move.end);
        }

        let two_square_advance = self.data.two_square_advance;
        let not_able_to_castle = self.data.not_able_to_castle;

        if _move.castling() != 0 {
            let old_rook_pos = if _move.castling() == MoveFlags::QUEENSIDE_CASTLING {
                0
            } else {
                7
            } + ((_move.start as usize) / 8) * 8;
            let new_rook_pos = if _move.castling() == MoveFlags::QUEENSIDE_CASTLING {
                3
            } else {
                5
            } + ((_move.start as usize) / 8) * 8;

            let rook = self.data.board[old_rook_pos].clone();
            debug_assert!(!rook.is_empty());

            self.data.board[old_rook_pos] = Piece::empty();

            debug_assert!(self.data.board[new_rook_pos].is_empty());

            self.data.piece_list._move(&rook, new_rook_pos as u8);
            self.data.board[new_rook_pos] = rook;
        }

        if _move.two_square_advance() {
            self.data.two_square_advance = (_move.end + _move.start) / 2;
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
            _move.en_passant(),
            not_able_to_castle,
            _move.castling(),
        ));

        self.data.piece_list._move(&piece, _move.end);
        self.data.board[_move.end as usize] = piece;

        self.ply += 1;
        self.data.white_turn = !self.data.white_turn;
    }

    pub(crate) fn undo_move(&mut self) {
        self.state = State::Playing;
        let change = self.changes.pop().expect("No move to undo");

        let mut piece = self.data.board[change.end as usize].clone();

        self.fifty_move_counter = change.fifty_move_counter;
        self.data.not_able_to_castle = change.not_able_to_castle;
        self.data.two_square_advance = change.two_square_advance;

        if !change.captured.is_empty() {
            self.data
                .piece_list
                .add(&change.captured, change.captured_position);
            self.data.board[change.captured_position as usize] = change.captured;
        } else {
            self.data.board[change.captured_position as usize].set_empty();
        }

        if change.en_passant {
            self.data.board[change.end as usize].set_empty();
        }

        if change.promotion {
            self.data.piece_list.remove(&piece);
            piece.promotion(0);
            self.data.piece_list.add(&piece, change.start);
        }

        if change.castling != 0 {
            let new_rook_pos = if change.castling == MoveFlags::QUEENSIDE_CASTLING {
                0
            } else {
                7
            } + ((change.end as usize) / 8) * 8;
            let old_rook_pos = if change.castling == MoveFlags::QUEENSIDE_CASTLING {
                3
            } else {
                5
            } + ((change.end as usize) / 8) * 8;

            let rook = self.data.board[old_rook_pos].clone();
            debug_assert!(!rook.is_empty());

            self.data.board[old_rook_pos] = Piece::empty();

            debug_assert!(self.data.board[new_rook_pos].is_empty());

            self.data.piece_list._move(&rook, new_rook_pos as u8);
            self.data.board[new_rook_pos] = rook;
        }

        self.data.piece_list._move(&piece, change.start);
        self.data.board[change.start as usize] = piece;

        self.ply -= 1;
        self.data.white_turn = !self.data.white_turn;
    }

    #[inline(always)]
    pub(crate) fn pawn_on_original_position(&self, index: u8, color: i8) -> bool {
        Position::from_index(index).file == 1 - 5 * (color - 1) / 2
    }

    pub fn reset(&mut self) {
        self.ply = 0;
        self.data.white_turn = true;
        self.data.not_able_to_castle = 0;
        self.data.two_square_advance = 0;
        self.fifty_move_counter = 0;
        self.data.piece_list = PieceList::new();
        self.load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }

    pub fn generate_moves(&mut self) {
        self.move_generator.generate_moves(&self.data);
    }

    pub fn benchmark(mut self, depth: u8, start: u8) -> (Self, usize, u128) {
        let now = std::time::Instant::now();

        if depth == 0 {
            return (self, 1, 0);
        }

        self.move_generator.generate_moves(&self.data);

        // self.move_generator.moves.iter().for_each(|m| print!("{},", m));
        // println!("");

        if depth == 1 {
            let _num_positions = self.move_generator.moves.len();
            return (self, _num_positions, 0);
        }

        self.check_game_state();

        if self.state != State::Playing {
            return (self, 0, 0);
        }

        let mut num_positions = 0;

        // let board = self.data.board.clone();

        for (i, _move) in self.move_generator.moves.clone().iter().enumerate() {

            self.make_move(&_move);

            let _num_positions;

            (self, _num_positions, _) = self.benchmark(depth - 1, start);

            num_positions += _num_positions;

            self.undo_move();

            // assert!(board == self.data.board);
        }

        (self, num_positions, now.elapsed().as_micros())
    }

    pub fn check_game_state(&mut self) {
        // depends on previous call to MoveGenerator::generate_moves

        // checkmate or stalemate
        if self.move_generator.moves.len() == 0 {
            if self.move_generator.in_check {
                self.state = if self.data.white_turn {
                    State::WhiteIsMated
                } else {
                    State::BlackIsMated
                };
                return;
            } else {
                self.state = State::Stalemate;
                return;
            }
        }

        // fifty move rule
        if self.fifty_move_counter >= 100 {
            self.state = State::FiftyMoveRule;
        }

        // TODO: check for threefold repetition

        // insufficient material
        if self.data.piece_list.pawn_count()
            + self.data.piece_list.rook_count()
            + self.data.piece_list.queen_count()
            == 0
        {
            if self.data.piece_list.knight_count() + self.data.piece_list.bishop_count() <= 1 {
                // only king v king, king v king + bishop or king v king + knight
                self.state = State::InsufficientMaterial;
                return;
            }
        }
    }

    pub fn play_round(&mut self) -> bool {
        if self.state != State::Playing {
            return true;
        }

        self.move_generator.generate_moves(&self.data);

        self.check_game_state();

        if self.state != State::Playing {
            return true;
        }

        let player = self.players[self.ply % 2];

        let _move = player._move(self);

        self.make_move(&_move);

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

        for i in 0..64 {
            let real_index = i % 8 + (7 - i / 8) * 8;
            if i % 8 == 0 {
                string.push_str(&color::Fg(color::White).to_string());
                string.push(char::from_digit(8 - (i / 8) as u32, 10).unwrap());
                string.push(' ');
                string.push_str(&color::Fg(color::Cyan).to_string());
            }

            string.push(self.data.board[real_index].as_unicode_char());

            if i % 8 == 7 {
                string.push_str(&color::Fg(color::White).to_string());
                string.push(' ');
                string.push(char::from_digit(8 - (i / 8) as u32, 10).unwrap());
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

        for i in 0..64 {
            let real_index = i % 8 + (7 - i / 8) * 8;
            if i % 8 == 0 {
                string.push_str(&color::Fg(color::White).to_string());
                string.push(char::from_digit(8 - (i / 8) as u32, 10).unwrap());
                string.push(' ');
                string.push_str(&color::Fg(color::LightWhite).to_string());
            }

            if self
                .move_generator
                .moves
                .iter()
                .any(|p| p.end as usize == real_index)
            {
                string.push_str(&color::Fg(color::Cyan).to_string());
            }

            string.push(self.data.board[real_index].as_unicode_char());

            string.push_str(&color::Fg(color::LightWhite).to_string());

            if i % 8 == 7 {
                string.push_str(&color::Fg(color::White).to_string());
                string.push(' ');
                string.push(char::from_digit(8 - (i / 8) as u32, 10).unwrap());

                string.push('\t');

                for j in 0..8 {
                    string.push(
                        if self.move_generator.fields_under_attack
                            & (1 << ((real_index / 8) * 8 + j))
                            != 0
                        {
                            '▆'
                        } else {
                            '·'
                        },
                    );
                    string.push(' ');
                }

                string.push('\t');

                for j in 0..8 {
                    string.push(
                        if self.move_generator.attacking_rays & (1 << ((real_index / 8) * 8 + j))
                            != 0
                        {
                            '▆'
                        } else {
                            '·'
                        },
                    );
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
