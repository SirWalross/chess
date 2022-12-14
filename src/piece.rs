use crate::{position::Position, Board};
use std::{
    fmt,
    ops::{self, Range},
};

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
    Bishop = 2,
    Knight = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

impl PieceType {
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

    pub fn from_piece(piece: i8) -> Self {
        match piece.abs() {
            a if a == PieceType::Pawn as i8 => PieceType::Pawn,
            a if a == PieceType::Bishop as i8  => PieceType::Bishop,
            a if a == PieceType::Knight as i8  => PieceType::Knight,
            a if a == PieceType::Rook as i8  => PieceType::Rook,
            a if a == PieceType::Queen as i8  => PieceType::Queen,
            a if a == PieceType::King as i8  => PieceType::King,
            _ => PieceType::None,
        }
    }

    pub fn value(&self) -> i32 {
        match self {
            PieceType::Pawn => 1,
            PieceType::Bishop => 3,
            PieceType::Knight => 3,
            PieceType::Rook => 5,
            PieceType::Queen => 7,
            PieceType::King => 0,
            PieceType::None => 0,
        }
    }
}

#[repr(i8)]
#[derive(Eq, PartialEq, Clone, Debug, Copy)]
pub enum PieceColor {
    White = 1,
    Black = -1,
    None = 0,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub(crate) piece: i8,
    pub(crate) uuid: u8,
}

impl Piece {
    pub fn empty() -> Piece {
        Piece { piece: 0, uuid: 0 }
    }

    pub fn set(&mut self, _type: PieceType, color: PieceColor, uuid: u8) {
        self.piece = (_type as i8) * (color as i8);
        self.uuid = uuid | (0x10 * ((color == PieceColor::Black) as u8));
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.piece == 0
    }

    #[inline(always)]
    pub fn is_pawn(&self) -> bool {
        self.piece.abs() == 1
    }

    #[inline(always)]
    pub fn is_bishop(&self) -> bool {
        self.piece.abs() == 2
    }

    #[inline(always)]
    pub fn is_knight(&self) -> bool {
        self.piece.abs() == 3
    }

    #[inline(always)]
    pub fn is_rook(&self) -> bool {
        self.piece.abs() == 4
    }

    #[inline(always)]
    pub fn is_queen(&self) -> bool {
        self.piece.abs() == 5
    }

    #[inline(always)]
    pub fn is_king(&self) -> bool {
        self.piece.abs() == 6
    }

    pub(crate) fn promotion(&mut self, promotion: u8) {
        self.piece = self.piece.signum() * (promotion as i8 + 1);
        debug_assert!(self.piece != 0);
    }

    #[inline(always)]
    pub fn is_white(&self) -> bool {
        self.piece.signum() == 1
    }

    #[inline(always)]
    pub fn is_black(&self) -> bool {
        self.piece.signum() == -1
    }

    #[inline(always)]
    pub fn color(&self) -> i8 {
        self.piece.signum()
    }

    #[inline(always)]
    pub fn is_color(&self, color: i8) -> bool {
        self.piece.signum() == color
    }

    #[inline(always)]
    pub fn to_piecelist_index(&self) -> usize {
        self.piece.abs() as usize - 1 + 3 * (self.piece.signum() + 1) as usize
    }

    pub fn as_char(&self) -> char {
        let piece_char = match &self.piece.abs() {
            1 => 'p',
            2 => 'b',
            3 => 'n',
            4 => 'r',
            5 => 'q',
            6 => 'k',
            _ => ' ',
        };

        if self.is_white() {
            piece_char.to_ascii_uppercase()
        } else if self.is_black() {
            piece_char
        } else {
            ' '
        }
    }

    pub fn as_unicode_char(&self) -> char {
        match self.piece {
            -1 => '???',
            -2 => '???',
            -3 => '???',
            -4 => '???',
            -5 => '???',
            -6 => '???',
            1 => '???',
            2 => '???',
            3 => '???',
            4 => '???',
            5 => '???',
            6 => '???',
            _ => ' ',
        }
    }

    pub fn as_unicode_char_abs(&self) -> char {
        match -self.piece.abs() {
            -1 => '???',
            -2 => '???',
            -3 => '???',
            -4 => '???',
            -5 => '???',
            -6 => '???',
            _ => ' ',
        }
    }

    pub fn set_empty(&mut self) {
        self.piece = 0;
        self.uuid = 0;
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}
