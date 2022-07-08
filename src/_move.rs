use std::fmt;

use crate::position::Position;

#[allow(non_snake_case)]
pub(crate) mod MoveFlags {
    pub(crate) const KINGSIDE_CASTLING: u8 = 1 << 0;
    pub(crate) const QUEENSIDE_CASTLING: u8 = 1 << 1;
    pub(crate) const QUEEN_PROMOTION: u8 = 4 << 2;
    pub(crate) const ROOK_PROMOTION: u8 = 3 << 2;
    pub(crate) const KNIGHT_PROMOTION: u8 = 2 << 2;
    pub(crate) const BISHOP_PROMOTION: u8 = 1 << 2;
    pub(crate) const TWO_SQUARE_ADVANCE: u8 = 1 << 5;
    pub(crate) const EN_PASSANT: u8 = 1 << 6;
}

#[derive(Clone, Debug)]
pub struct Move {
    pub start: u8,
    pub end: u8,
    pub(crate) flags: u8, 
    // bit 0 castling kingside, bit 1 castling queenside, bit 2, 3 & 4 promotion
    // bit 5 two square advance, bit 6 en passant
}

impl Move {
    pub fn new(
        start: u8,
        end: u8,
        castling: u8,
        promotion: u8,
        two_square_advance: bool,
        en_passant: bool,
    ) -> Move {
        Move {
            start: start,
            end: end,
            flags: castling
                & (promotion << 2)
                & ((two_square_advance as u8) << 5)
                & ((en_passant as u8) << 6),
        }
    }

    pub(crate) fn from_flags(start: u8, end: u8, flags: u8) -> Move {
        Move { start, end, flags }
    }

    pub fn from_positions(
        start: Position,
        end: Position,
        castling: u8,
        promotion: u8,
        two_square_advance: bool,
        en_passant: bool,
    ) -> Move {
        Move {
            start: start.index() as u8,
            end: end.index() as u8,
            flags: castling
                | (promotion << 2)
                | ((two_square_advance as u8) << 5)
                | ((en_passant as u8) << 6),
        }
    }

    pub fn castling(&self) -> u8 {
        (self.flags & 0x3) as u8
    }

    pub fn promotion(&self) -> u8 {
        ((self.flags & 0x1c) >> 2) as u8
    }

    pub fn two_square_advance(&self) -> bool {
        (self.flags & 0x20) != 0
    }

    pub fn en_passant(&self) -> bool {
        (self.flags & 0x40) != 0
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{},{}}}", Position::from_index(self.start).to_string(), Position::from_index(self.end).to_string())
    }
}
