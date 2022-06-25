use termion::color;

use crate::_move::Move;
use crate::piece::{Piece, PieceColor, PieceMoves, PieceType, Position};
use crate::player::PlayerType;
use core::num;
use std::fmt;

struct Change {
    start: u8,
    end: u8,
    captured_type: i8,
    captured_uuid: u8,
    captured_position: u8,
    two_square_advance: u8,
    fifty_move_counter: u8,
    promotion: bool,
    able_to_castle: [bool; 4],
    castling: u8,
}

impl Change {
    pub(crate) fn new(
        start: u8,
        end: u8,
        captured_type: i8,
        captured_uuid: u8,
        captured_position: u8,
        two_square_advance: u8,
        fifty_move_counter: u8,
        promotion: bool,
        able_to_castle: [bool; 4],
        castling: u8,
    ) -> Self {
        Self {
            start,
            end,
            captured_type,
            captured_uuid,
            captured_position,
            two_square_advance,
            fifty_move_counter,
            promotion,
            able_to_castle,
            castling,
        }
    }
}

pub struct Board {
    pub(crate) board: [Piece; 8 * 8],
    pub(crate) fields_under_attack: [u16; 8 * 8],
    pub(crate) allowed_moves: Vec<(Move, Piece)>,
    pub(crate) piece_moves: PieceMoves,
    pub(crate) turn: PieceColor,
    players: [PlayerType; 2],
    ply: usize,
    pub(crate) able_to_castle: [bool; 4], // for white, black for rook1 and rook2 respectively
    fifty_move_counter: u8,
    pub(crate) two_square_advance: u8, // intermittend position of the pawn while advancing two squares starting at 1
    pub(crate) pinned_piece_positions: Vec<Vec<(u8, u8, u8)>>,
    // positions on this list are for a pinned piece
    // (index of position, uuid of pinned piece and position of attacker)
    pub(crate) safe_positions: [u16; 8 * 8], // safe positions for each piece
    changes: Vec<Change>,
}

impl Board {
    pub fn new(player1: PlayerType, player2: PlayerType) -> Board {
        let mut board = Board {
            board: [Piece::new(PieceType::None, PieceColor::None, 0); 64],
            fields_under_attack: [0; 64],
            allowed_moves: Vec::new(),
            piece_moves: PieceMoves::new(),
            turn: PieceColor::White,
            players: [player1, player2],
            ply: 0,
            able_to_castle: [true, true, true, true],
            fifty_move_counter: 0,
            two_square_advance: 0,
            pinned_piece_positions: Vec::new(),
            safe_positions: [0xffff; 64],
            changes: Vec::new(),
        };
        board.populate();
        board
    }

    fn populate(&mut self) -> () {
        for board_spot in 0..64 {
            let uuid = board_spot as u8;
            match board_spot {
                0 | 7 => self.board[board_spot].set(PieceType::Rook, PieceColor::White, uuid),
                56 | 63 => {
                    self.board[board_spot].set(PieceType::Rook, PieceColor::Black, 63 - uuid)
                }
                1 | 6 => self.board[board_spot].set(PieceType::Knight, PieceColor::White, uuid),
                57 | 62 => {
                    self.board[board_spot].set(PieceType::Knight, PieceColor::Black, 63 - uuid)
                }
                2 | 5 => self.board[board_spot].set(PieceType::Bishop, PieceColor::White, uuid),
                58 | 61 => {
                    self.board[board_spot].set(PieceType::Bishop, PieceColor::Black, 63 - uuid)
                }
                3 => self.board[board_spot].set(PieceType::Queen, PieceColor::White, uuid),
                59 => self.board[board_spot].set(PieceType::Queen, PieceColor::Black, 63 - uuid),
                4 => self.board[board_spot].set(PieceType::King, PieceColor::White, uuid),
                60 => self.board[board_spot].set(PieceType::King, PieceColor::Black, 63 - uuid),
                8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 => {
                    self.board[board_spot].set(PieceType::Pawn, PieceColor::White, uuid)
                }
                48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 => {
                    self.board[board_spot].set(PieceType::Pawn, PieceColor::Black, 63 - uuid)
                }
                _ => (),
            };
        }
    }

    fn generate_fields_under_attack(&mut self) {
        for field in &mut self.fields_under_attack {
            *field = 0;
        }

        self.pinned_piece_positions.clear();
        self.generate_moves(true);

        self.allowed_moves.iter().for_each(|(_move, piece)| {
            if (_move.flags & (1 << 7)) == 0 {
                self.fields_under_attack[_move.end as usize] |= 1 << piece.uuid;
            }
        });
    }

    fn generate_safe_positions(&mut self) {
        let king_index = self
            .board
            .iter()
            .position(|p| p.get() == (PieceType::King as i8) * (self.turn.clone() as i8))
            .unwrap();
        if self.fields_under_attack[king_index] != 0 {
            // king under attack
            let attacking_pieces = self.fields_under_attack[king_index];

            if attacking_pieces.is_power_of_two() {
                // only one attacker

                let attacking_piece_index = self
                    .board
                    .iter()
                    .position(|p| {
                        1 << p.uuid == attacking_pieces
                            && -p.get().signum() == self.turn.clone() as i8
                    })
                    .unwrap();

                for (i, p) in self.safe_positions.iter_mut().enumerate() {
                    if self.fields_under_attack[i] & 0xffff & attacking_pieces != 0
                        && i != king_index
                        && Position::from_index(i as u8).is_on_path_between(
                            Position::from_index(king_index as u8),
                            Position::from_index(attacking_piece_index as u8),
                        )
                    {
                        // safe position is the path between the attacker and the king
                        *p = 0xffff;
                    } else {
                        *p = 0;
                    }
                }

                // safe position is also capturing the attacker
                self.safe_positions[attacking_piece_index] = 0xffff;
            } else {
                // with multiple attackers no safe position
                for p in &mut self.safe_positions {
                    *p = 0;
                }
            }

            // reset pinned pieces as king is directly under attack
            self.pinned_piece_positions.clear();
        } else if !self.pinned_piece_positions.is_empty() {
            // king indirectly attacked through pinned piece

            let not_pinned_pieces = self
                .pinned_piece_positions
                .iter()
                .map(|(v)| v[0].1)
                .fold(0xffff as u16, |org, x| org ^ (1 << x));

            let pinned_pieces = !not_pinned_pieces;

            for p in &mut self.safe_positions {
                *p = not_pinned_pieces;
            }

            for p in self.pinned_piece_positions.iter() {
                for (i, uuid, _) in p.iter() {
                    // pinned piece is allowed to move along attack vector from attacker
                    self.safe_positions[*i as usize] |= 1 << *uuid;
                }
            }
        } else {
            for p in &mut self.safe_positions {
                *p = 0xffff;
            }
        }
    }

    pub fn generate_moves(&mut self, all: bool) -> () {
        // all: wether to generate all moves, used for generate_fields_under_attack
        self.allowed_moves.clear();
        self.allowed_moves.reserve(40);

        let mut pinned_piece_positions = Vec::new();

        for (index, piece) in self
            .board
            .iter()
            .enumerate()
            .filter(|(_, p)| p.get().signum() == self.turn.clone() as i8)
        {
            let moves = if all {
                piece.generate_all_moves(&self, index as u8, &mut pinned_piece_positions)
            } else {
                piece.generate_moves(&self, index as u8)
            };

            // println!(
            //     "Piece {}, at {} with allowed moves: {:?}",
            //     piece.as_char(),
            //     index,
            //     moves
            // );
            self.allowed_moves.extend(
                moves
                    .iter()
                    .map(|(i, flags)| (Move::from_flags(index as u8, *i, *flags), *piece)),
            );
        }

        self.pinned_piece_positions = pinned_piece_positions;
    }

    fn make_move(&mut self, _move: Move) {
        let mut piece = self.board[_move.start as usize];

        self.board[_move.start as usize] = Piece::empty();

        let fifty_move_counter = self.fifty_move_counter;

        if piece.piece_type() != PieceType::Pawn && self.board[_move.end as usize].get() == 0 {
            self.fifty_move_counter += 1;
        } else {
            self.fifty_move_counter = 0;
        }

        let captured = self.board[if _move.en_passant() {
            self.two_square_advance as usize - 1
        } else {
            _move.end as usize
        }];

        let captured_position = if _move.en_passant() {
            self.two_square_advance - 1
        } else {
            _move.end
        };

        if _move.en_passant() {
            // en passant
            debug_assert!(self.board[_move.end as usize].piece_type() == PieceType::None);
            debug_assert!(piece.piece_type() == PieceType::Pawn);
            debug_assert!(self.two_square_advance != 0);
            self.board[self.two_square_advance as usize - 1] = Piece::empty();
        }

        if _move.promotion() != 0 {
            piece.promotion(_move.promotion());
        }

        self.board[_move.end as usize] = piece;

        self.changes.push(Change::new(
            _move.start,
            _move.end,
            captured.get(),
            captured.uuid,
            captured_position,
            self.two_square_advance,
            fifty_move_counter,
            _move.promotion() != 0,
            self.able_to_castle.clone(),
            _move.castling(),
        ));

        if _move.castling() != 0 {
            let old_rook_pos =
                if _move.castling() == 1 { 0 } else { 7 } + ((_move.start as usize) / 8) * 8;
            let new_rook_pos =
                if _move.castling() == 1 { 3 } else { 5 } + ((_move.start as usize) / 8) * 8;

            let rook = self.board[old_rook_pos];

            self.board[old_rook_pos] = Piece::empty();

            debug_assert!(self.board[new_rook_pos].piece_type() == PieceType::None);

            self.board[new_rook_pos] = rook;
        }

        if _move.two_square_advance() {
            self.two_square_advance = (_move.end + _move.start) / 2 + 1;
        } else {
            self.two_square_advance = 0;
        }

        if piece.piece_type() == PieceType::King {
            self.able_to_castle[((piece.piece_color() as i8) * -1 + 1) as usize] = false;
            self.able_to_castle[((piece.piece_color() as i8) * -1 + 2) as usize] = false;
        } else if piece.piece_type() == PieceType::Rook
            && (_move.start % 8 == 0 || _move.start % 8 == 7)
        {
            self.able_to_castle[((piece.piece_color() as i8) * -1 + 1) as usize
                + (_move.start as usize % 8) / 7] = false;
        }
    }

    fn undo_move(&mut self) {
        let change = self.changes.pop().expect("No move to undo");
        self.able_to_castle = change.able_to_castle;
        self.two_square_advance = change.two_square_advance;
        self.fifty_move_counter = change.fifty_move_counter;
        self.board[change.start as usize] = self.board[change.end as usize];

        self.board[change.captured_position as usize] =
            Piece::from_type(change.captured_type, change.captured_uuid);

        if change.castling != 0 {
            let old_rook_pos =
                if change.castling == 1 { 3 } else { 5 } + ((change.start as usize) / 8) * 8;
            let new_rook_pos =
                if change.castling == 1 { 0 } else { 7 } + ((change.start as usize) / 8) * 8;

            let rook = self.board[old_rook_pos];

            self.board[old_rook_pos] = Piece::empty();

            debug_assert!(self.board[new_rook_pos].piece_type() == PieceType::None);

            self.board[new_rook_pos] = rook;
        }

        if change.promotion {
            let piece = &mut self.board[change.start as usize];
            piece.set(PieceType::Pawn, piece.piece_color(), piece.uuid);
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

        self.generate_moves(false);

        let mut num_positions = 0;

        let moves = self.allowed_moves.clone();

        for _move in moves {
            self.make_move(_move.0);

            self.generate_fields_under_attack();

            self.ply += 1;
            self.turn = if self.turn == PieceColor::White {
                PieceColor::Black
            } else {
                PieceColor::White
            };

            self.generate_safe_positions();

            let _num_positions;
            (self, _num_positions, _) = self.benchmark(depth - 1);
            num_positions += _num_positions;

            self.undo_move();

            self.generate_fields_under_attack();

            self.ply -= 1;
            self.turn = if self.turn == PieceColor::White {
                PieceColor::Black
            } else {
                PieceColor::White
            };

            self.generate_safe_positions();
        }

        (self, num_positions, now.elapsed().as_micros())
    }

    pub fn play_round(&mut self) -> bool {
        let now = std::time::Instant::now();
        self.generate_moves(false);

        if self.allowed_moves.len() == 0 || self.fifty_move_counter == 50 {
            return true;
        }

        let _move = self.players[self.ply % 2]._move(self);

        self.make_move(_move);

        self.generate_fields_under_attack();

        self.ply += 1;
        self.turn = if self.turn == PieceColor::White {
            PieceColor::Black
        } else {
            PieceColor::White
        };

        self.generate_safe_positions();

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

        for (i, piece) in self.board.iter().enumerate() {
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

        for (i, piece) in self.board.iter().enumerate() {
            if i % 8 == 0 {
                string.push_str(&color::Fg(color::White).to_string());
                string.push(char::from_digit((i / 8) as u32 + 1, 10).unwrap());
                string.push(' ');
                string.push_str(&color::Fg(color::LightWhite).to_string());
            }

            if self.allowed_moves.iter().any(|(p, _)| p.end as usize == i) {
                string.push_str(&color::Fg(color::Cyan).to_string());
            }

            string.push(piece.as_char());

            string.push_str(&color::Fg(color::LightWhite).to_string());

            if i % 8 == 7 {
                string.push_str(&color::Fg(color::White).to_string());
                string.push(' ');
                string.push(char::from_digit((i / 8) as u32 + 1, 10).unwrap());

                string.push('\t');

                for is_under_attack in self.fields_under_attack[(i / 8 * 8)..(i / 8 * 8 + 8)].iter()
                {
                    string.push(if *is_under_attack != 0 { '▆' } else { '.' });
                    string.push(' ');
                }
                string.push('\t');

                for safe in self.safe_positions[(i / 8 * 8)..(i / 8 * 8 + 8)].iter() {
                    string.push(if *safe == 0xffff { '▆' } else { '.' });
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
