use crate::Board;
use std::{
    fmt,
    ops::{self, Range},
};

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub(crate) file: i8,
    pub(crate) rank: i8,
}

impl Position {
    pub fn new(file: i8, rank: i8) -> Position {
        Position { file, rank }
    }

    pub(crate) fn from_index(index: u8) -> Position {
        Position {
            file: (index / 8) as i8,
            rank: (index % 8) as i8,
        }
    }

    #[inline(always)]
    pub(crate) fn in_bounds(&self, index: u8, color: i8) -> bool {
        let file = (index / 8) as i8;
        let rank = (index % 8) as i8;
        file + self.file * color <= 7
            && file + self.file * color >= 0
            && rank + self.rank * color <= 7
            && rank + self.rank * color >= 0
    }

    fn rotate_180(mut self) -> Self {
        // rotate 180 degrees
        self.rank *= -1;
        self.file *= -1;
        self
    }

    fn rotate_90(mut self) -> Self {
        // rotate 90 degrees
        let file = self.file;
        self.file = -self.rank;
        self.rank = file;
        self
    }

    pub(crate) fn index(&self) -> i8 {
        self.file * 8 + self.rank
    }

    fn chebyshev_distance(&self) -> i8 {
        std::cmp::max(self.file.abs(), self.rank.abs())
    }

    pub fn is_on_path_between(&self, p1: Position, p2: Position) -> bool {
        let diff = p1 - p2;
        let diff1 = p1 - self.clone();
        let diff2 = self.clone() - p2;
        if diff.file.abs() != diff.rank.abs() && diff.file != 0 && diff.rank != 0 {
            // invalid diff, probably from knight
            false
        } else if diff.chebyshev_distance() < 2 {
            // no point between
            false
        } else if diff1.chebyshev_distance() + diff2.chebyshev_distance()
            != diff.chebyshev_distance()
        {
            // not on line
            false
        } else {
            true
        }
    }

    fn norm(mut self) -> Self {
        self.file = self.file.signum();
        self.rank = self.rank.signum();
        self
    }
}

impl std::ops::Add<u8> for Position {
    type Output = u8;

    #[inline(always)]
    fn add(self, other: u8) -> u8 {
        let file = (other / 8) as i8;
        let rank = (other % 8) as i8;
        ((file + self.file) * 8 + rank + self.rank) as u8
    }
}

impl std::ops::Sub<Position> for Position {
    type Output = Position;

    #[inline(always)]
    fn sub(mut self, other: Position) -> Self {
        self.file -= other.file;
        self.rank -= other.rank;
        self
    }
}

impl std::ops::Mul<i8> for Position {
    type Output = Position;

    fn mul(mut self, _rhs: i8) -> Position {
        self.file *= _rhs;
        self.rank *= _rhs;
        self
    }
}

/// holds all possible moves for all piece types
pub(crate) struct PieceMoves {
    pub(crate) pawn_moves: Vec<Position>,
    pub(crate) bishop_moves: Vec<Position>,
    pub(crate) knight_moves: Vec<Position>,
    pub(crate) rook_moves: Vec<Position>,
    pub(crate) queen_moves: Vec<Position>,
    pub(crate) king_moves: Vec<Position>,
    pub(crate) castling_moves: [[[Range<usize>; 2]; 2]; 2], // for black and white, check for castling and check for pieces and for left and right
    pub(crate) promotion_flags: [u8; 4],
}

impl PieceMoves {
    pub fn new() -> PieceMoves {
        let mut diag: Vec<Position> = (1..8 as i8).map(|i| Position::new(i, i)).collect();
        diag.extend(diag.clone().iter().map(|p| p.rotate_180()));
        diag.extend(diag.clone().iter().map(|p| p.rotate_90()));

        let mut horiz: Vec<Position> = (1..8 as i8).map(|i| Position::new(i, 0)).collect();
        horiz.extend(horiz.clone().iter().map(|p| p.rotate_90()));
        horiz.extend(horiz.clone().iter().map(|p| p.rotate_180()));

        let mut piece_moves = PieceMoves {
            pawn_moves: vec![
                Position::new(1, 0),
                Position::new(2, 0),
                Position::new(1, -1),
                Position::new(1, 1),
            ],
            bishop_moves: diag.clone(),
            knight_moves: vec![
                Position::new(2, 1),
                Position::new(2, -1),
                Position::new(-2, 1),
                Position::new(-2, -1),
                Position::new(1, 2),
                Position::new(1, -2),
                Position::new(-1, 2),
                Position::new(-1, -2),
            ],
            rook_moves: horiz.clone(),
            queen_moves: diag,
            king_moves: Vec::new(),
            castling_moves: [
                [[2..4, 5..7], [1..4, 5..7]],
                [[58..60, 61..63], [57..60, 61..63]],
            ],
            promotion_flags: [1 << 2, 2 << 2, 3 << 2, 4 << 2],
        };
        piece_moves.queen_moves.extend(horiz);
        piece_moves
            .king_moves
            .extend(piece_moves.queen_moves.iter().step_by(7));
        piece_moves.king_moves.push(Position::new(0, -2));
        piece_moves.king_moves.push(Position::new(0, 2));

        piece_moves
    }
}

#[inline(always)]
fn to_local(_move: u8, index: u8) -> u8 {
    // assume that resulting is always in bounds
    let file = (index / 8) as i8;
    let rank = (index % 8) as i8;
    if (_move & 0x40) != 0 {
        // 2nd or 3rd quadrant
        let file_offset = _move / 8;
        let rank_offset = _move % 8;
    } else {
        // 1st or 4th quadrant
        let file_offset = _move / 8;
        let rank_offset = _move % 8;
    }
    _move
}

#[repr(i8)]
#[derive(Eq, PartialEq)]
pub enum PieceType {
    None = 0,
    Pawn = 1,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub fn as_char(&self) -> char {
        match &self {
            PieceType::Pawn => 'p',
            PieceType::Bishop => 'b',
            PieceType::Knight => 'n',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
            PieceType::None => ' ',
        }
    }

    pub fn from_char(c: char) -> Self {
        match c {
            'p' => PieceType::Pawn,
            'b' => PieceType::Bishop,
            'n' => PieceType::Knight,
            'r' => PieceType::Rook,
            'q' => PieceType::Queen,
            'k' => PieceType::King,
            _ => PieceType::None,
        }
    }

    pub fn from_promotion(_type: u8) -> Self {
        match _type {
            1 => PieceType::Queen,
            2 => PieceType::Rook,
            3 => PieceType::Bishop,
            4 => PieceType::Knight,
            _ => PieceType::None,
        }
    }
}

#[repr(i8)]
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum PieceColor {
    White = 1,
    Black = -1,
    None = 0,
}

#[derive(Clone, Copy, Debug)]
pub struct Piece {
    piece: i8,
    pub(crate) uuid: u8, // only unique per side
}

impl Piece {
    pub fn new(_type: PieceType, color: PieceColor, uuid: u8) -> Piece {
        Piece {
            piece: (_type as i8) * (color as i8),
            uuid: uuid,
        }
    }

    pub(crate) fn from_type(piece: i8, uuid: u8) -> Piece{
        Piece {
            piece: piece,
            uuid: uuid
        }
    }

    pub fn empty() -> Piece {
        Piece { piece: 0, uuid: 0 }
    }

    pub fn set(&mut self, _type: PieceType, color: PieceColor, uuid: u8) -> () {
        self.piece = (_type as i8) * (color as i8);
        debug_assert!(self.uuid == 0);
        self.uuid = uuid;
    }

    pub(crate) fn promotion(&mut self, promotion: u8) {
        self.piece = self.piece.signum() * (PieceType::from_promotion(promotion) as i8);
        debug_assert!(self.piece != 0);
    }

    pub(crate) fn get(&self) -> i8 {
        self.piece
    }

    pub fn piece_type(&self) -> PieceType {
        match self.piece.abs() {
            1 => PieceType::Pawn,
            2 => PieceType::Bishop,
            3 => PieceType::Knight,
            4 => PieceType::Rook,
            5 => PieceType::Queen,
            6 => PieceType::King,
            _ => PieceType::None,
        }
    }

    pub fn piece_color(&self) -> PieceColor {
        match self.piece.signum() {
            -1 => PieceColor::Black,
            1 => PieceColor::White,
            _ => PieceColor::None,
        }
    }

    pub fn as_char(&self) -> char {
        let piece_char = self.piece_type().as_char();

        if self.piece_color() == PieceColor::White {
            piece_char.to_ascii_uppercase()
        } else if self.piece_color() == PieceColor::Black {
            piece_char
        } else {
            '.'
        }
    }

    pub(crate) fn generate_all_moves(
        &self,
        board: &Board,
        index: u8,
        pinned_piece_positions: &mut Vec<Vec<(u8, u8, u8)>>,
    ) -> Vec<(u8, u8)> {
        let mut moves = Vec::with_capacity(14);
        let mut i = 0;
        let color = self.piece.signum();
        let position_flag: u16 = 1 << self.uuid;

        let mut pinning_piece = 0; // index of move of the king
        let mut hit_count = 0;

        match self.piece_type() {
            PieceType::King => {
                while i < 10 {
                    let _move = board.piece_moves.king_moves[i];
                    let dest = _move * color + index;

                    if _move.in_bounds(index, color)
                        && i < 8
                        && board.fields_under_attack[dest as usize] == 0
                    {
                        moves.push((dest, 0));
                    } else if _move.in_bounds(index, color)
                        && i >= 8
                        && board.able_to_castle[i - 8 + ((color * -1 + 1) as usize)]
                        && board.fields_under_attack[dest as usize] == 0
                        && board.board[board.piece_moves.castling_moves
                            [((color * -1 + 1) / 2) as usize][0][i - 8]
                            .clone()]
                        .iter()
                        .all(|f| f.piece == 0)
                        && board.fields_under_attack[board.piece_moves.castling_moves
                            [((color * -1 + 1) / 2) as usize][1][i - 8]
                            .clone()]
                        .iter()
                        .all(|f| *f == 0)
                    {
                        moves.push((dest, i as u8 - 7));
                    }
                    i += 1;
                }
            }
            PieceType::Knight => {
                while i < 7 {
                    let _move = board.piece_moves.knight_moves[i];
                    let dest = _move * color + index;

                    if _move.in_bounds(index, color)
                        && board.safe_positions[dest as usize] & position_flag != 0
                    {
                        moves.push((dest, 0));
                    }
                    i += 1;
                }
            }
            PieceType::Pawn => {
                while i < 4 {
                    let _move = board.piece_moves.pawn_moves[i];
                    let dest = _move * color + index;

                    let in_bounds = _move.in_bounds(index, color);

                    if in_bounds
                        && board.safe_positions[dest as usize] & position_flag != 0
                        && i > 1
                    {
                        moves.push((dest, 0));
                    }
                    i += 1;
                }
            }
            PieceType::Bishop => {
                while i < 28 {
                    let _move = board.piece_moves.bishop_moves[i];
                    let dest = _move * color + index;

                    let inbounds = _move.in_bounds(index, color);
                    if inbounds && board.safe_positions[dest as usize] & position_flag != 0 {
                        if hit_count == 0 {
                            moves.push((dest, 0));
                        }

                        if board.board[dest as usize].piece != 0 {
                            hit_count += 1;

                            if hit_count == 1 {
                                pinning_piece = board.board[dest as usize].uuid;
                            }
                        }

                        if board.board[dest as usize].piece_type() == PieceType::King
                            && i % 7 != 0
                            && hit_count == 2
                        {
                            // pinning
                            pinned_piece_positions.push(
                                (0..(i % 7))
                                    .map(|counter| {
                                        (
                                            _move.norm() * color * (counter as i8) + index,
                                            pinning_piece,
                                            index,
                                        )
                                    })
                                    .collect(),
                            );
                        }
                    }
                    if inbounds
                        && hit_count != 2
                        && board.board[dest as usize].piece.signum() != color
                    {
                        // continue to next position
                        i += 1;
                    } else {
                        // jump to next direction
                        i += 7 - i % 7;
                    }

                    if i % 7 == 0 {
                        // changing direction -> reset hit_count
                        hit_count = 0;
                    }
                }
            }
            PieceType::Rook => {
                while i < 28 {
                    let _move = board.piece_moves.rook_moves[i];
                    let dest = _move * color + index;

                    let inbounds = _move.in_bounds(index, color);

                    if inbounds && board.safe_positions[dest as usize] & position_flag != 0 {
                        if hit_count == 0 {
                            moves.push((dest, 0));
                        }

                        if board.board[dest as usize].piece != 0 {
                            hit_count += 1;

                            if hit_count == 1 {
                                pinning_piece = board.board[dest as usize].uuid;
                            }
                        }

                        if board.board[dest as usize].piece_type() == PieceType::King
                            && i % 7 != 0
                            && hit_count == 2
                        {
                            // pinning
                            pinned_piece_positions.push(
                                (0..(i % 7))
                                    .map(|counter| {
                                        (
                                            _move.norm() * color * (counter as i8) + index,
                                            pinning_piece,
                                            index,
                                        )
                                    })
                                    .collect(),
                            );
                        }
                    }
                    if inbounds
                        && hit_count != 2
                        && board.board[dest as usize].piece.signum() != color
                    {
                        // continue to next position
                        i += 1;
                    } else {
                        // jump to next direction
                        i += 7 - i % 7;
                    }

                    if i % 7 == 0 {
                        // changing direction -> reset hit_count
                        hit_count = 0;
                    }
                }
            }
            PieceType::Queen => {
                while i < 56 {
                    let _move = board.piece_moves.queen_moves[i];
                    let dest = _move * color + index;

                    let inbounds = _move.in_bounds(index, color);
                    if inbounds && board.safe_positions[dest as usize] & position_flag != 0 {
                        if hit_count == 0 {
                            moves.push((dest, 0));
                        }

                        if board.board[dest as usize].piece != 0 {
                            hit_count += 1;

                            if hit_count == 1 {
                                pinning_piece = board.board[dest as usize].uuid;
                            }
                        }

                        if board.board[dest as usize].piece_type() == PieceType::King
                            && i % 7 != 0
                            && hit_count == 2
                        {
                            // pinning
                            pinned_piece_positions.push(
                                (0..=(i % 7))
                                    .map(|counter| {
                                        (
                                            _move.norm() * color * (counter as i8) + index,
                                            pinning_piece,
                                            index,
                                        )
                                    })
                                    .collect(),
                            );
                        }
                    }
                    if inbounds
                        && hit_count != 2
                        && board.board[dest as usize].piece.signum() != color
                    {
                        // continue to next position
                        i += 1;
                    } else {
                        // jump to next direction
                        i += 7 - i % 7;
                    }

                    if i % 7 == 0 {
                        // changing direction -> reset hit_count
                        hit_count = 0;
                    }
                }
            }
            _ => (),
        };
        moves
    }

    pub(crate) fn generate_moves(&self, board: &Board, index: u8) -> Vec<(u8, u8)> {
        let mut moves = Vec::with_capacity(14);
        let mut i = 0;
        let color = self.piece.signum();
        let position_flag: u16 = 1 << self.uuid;

        match self.piece_type() {
            PieceType::King => {
                while i < 10 {
                    let _move = board.piece_moves.king_moves[i];
                    let dest = _move * color + index;

                    if _move.in_bounds(index, color)
                        && i < 8
                        && board.board[dest as usize].piece.signum() != color
                        && board.fields_under_attack[dest as usize] == 0
                    {
                        moves.push((dest, 0));
                    } else if _move.in_bounds(index, color)
                        && i >= 8
                        && board.able_to_castle[i - 8 + ((color * -1 + 1) as usize)]
                        && board.fields_under_attack[dest as usize] == 0
                        && board.board[board.piece_moves.castling_moves
                            [((color * -1 + 1) / 2) as usize][0][i - 8]
                            .clone()]
                        .iter()
                        .all(|f| f.piece == 0)
                        && board.fields_under_attack[board.piece_moves.castling_moves
                            [((color * -1 + 1) / 2) as usize][1][i - 8]
                            .clone()]
                        .iter()
                        .all(|f| *f == 0)
                    {
                        moves.push((dest, i as u8 - 7));
                    }
                    i += 1;
                }
            }
            PieceType::Knight => {
                while i < 7 {
                    let _move = board.piece_moves.knight_moves[i];
                    let dest = _move * color + index;

                    if _move.in_bounds(index, color)
                        && board.board[dest as usize].piece.signum() != color
                        && board.safe_positions[dest as usize] & position_flag != 0
                    {
                        moves.push((dest, 0));
                    }
                    i += 1;
                }
            }
            PieceType::Pawn => {
                while i < 4 {
                    let _move = board.piece_moves.pawn_moves[i];
                    let dest = _move * color + index;

                    let in_bounds = _move.in_bounds(index, color);

                    if in_bounds
                        && board.safe_positions[dest as usize] & position_flag != 0
                        && ((board.board[dest as usize].piece.signum() == -color && i > 1)
                            || (board.board[dest as usize].piece.signum() == 0 && i == 0)
                            || (board.board[dest as usize].piece.signum() == 0
                                && board.board[(index as i8 + 8 * color) as usize]
                                    .piece
                                    .signum()
                                    == 0
                                && board.pawn_on_original_position(index, color)
                                && i == 1))
                    {
                        if dest / 8 == ((color + 1) / 2 * 7) as u8 {
                            // promotion

                            for flag in board.piece_moves.promotion_flags {
                                moves.push((dest, flag));
                            }
                        } else if (dest as i8 - index as i8).abs() == 16 {
                            // set two square advance flag

                            debug_assert!(i == 1);
                            moves.push((dest, (1 << 5)));
                        } else {
                            moves.push((dest, 0));
                        }
                    } else if in_bounds
                        && board.safe_positions[dest as usize] & position_flag != 0
                        && board.two_square_advance != 0
                        && i > 1
                        && dest == board.two_square_advance - 1
                    {
                        // set en passant flag
                        moves.push((dest, 1 << 6));
                    }
                    i += 1;
                }
            }
            PieceType::Bishop => {
                while i < 28 {
                    let _move = board.piece_moves.bishop_moves[i];
                    let dest = _move * color + index;

                    let inbounds = _move.in_bounds(index, color);
                    if inbounds
                        && board.safe_positions[dest as usize] & position_flag != 0
                        && board.board[dest as usize].piece.signum() != color
                    {
                        moves.push((dest, 0));
                    }
                    i += 1 + if inbounds {
                        (6 - i % 7) * ((board.board[dest as usize].piece.signum() != 0) as usize)
                    } else {
                        0
                    };
                }
            }
            PieceType::Rook => {
                while i < 28 {
                    let _move = board.piece_moves.rook_moves[i];
                    let dest = _move * color + index;

                    let inbounds = _move.in_bounds(index, color);

                    if inbounds
                        && board.safe_positions[dest as usize] & position_flag != 0
                        && board.board[dest as usize].piece.signum() != color
                    {
                        moves.push((dest, 0));
                    }
                    i += 1 + if inbounds {
                        (6 - i % 7) * ((board.board[dest as usize].piece.signum() != 0) as usize)
                    } else {
                        6 - i % 7
                    };
                }
            }
            PieceType::Queen => {
                while i < 56 {
                    let _move = board.piece_moves.queen_moves[i];
                    let dest = _move * color + index;

                    let inbounds = _move.in_bounds(index, color);
                    if inbounds
                        && board.safe_positions[dest as usize] & position_flag != 0
                        && board.board[dest as usize].piece.signum() != color
                    {
                        moves.push((dest, 0));
                    }
                    i += 1 + if inbounds {
                        (6 - i % 7) * ((board.board[dest as usize].piece.signum() != 0) as usize)
                    } else {
                        0
                    };
                }
            }
            _ => (),
        };
        moves
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}
