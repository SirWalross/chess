use crate::position::Position;

#[allow(non_snake_case)]
pub(crate) mod Direction {
    #[inline(always)]
    pub(crate) fn queenside(color: i8) -> usize {
        7 - (color == -1) as usize
    }

    #[inline(always)]
    pub(crate) fn kingside(color: i8) -> usize {
        6 + (color == -1) as usize
    }
}

/// holds all possible moves for all piece types
pub(crate) struct PieceMoves {
    pub(crate) pawn_moves: [Position; 3],
    pub(crate) knight_moves: [Position; 8],
    pub(crate) sliding: [Position; 8],
    pub(crate) sliding_offsets: [i8; 8],
    pub(crate) pawn_direction_index: [u8; 3],
}

impl PieceMoves {
    pub fn new() -> PieceMoves {
        PieceMoves {
            pawn_moves: [
                Position::new(1, 0),
                Position::new(1, -1),
                Position::new(1, 1),
            ],
            knight_moves: [
                Position::new(2, 1),
                Position::new(2, -1),
                Position::new(-2, 1),
                Position::new(-2, -1),
                Position::new(1, 2),
                Position::new(1, -2),
                Position::new(-1, 2),
                Position::new(-1, -2),
            ],
            sliding: [
                Position::new(1, 1),
                Position::new(-1, -1),
                Position::new(1, -1),
                Position::new(-1, 1),
                Position::new(1, 0),
                Position::new(-1, 0),
                Position::new(0, 1),
                Position::new(0, -1),
            ],
            sliding_offsets: [9, -9, 7, -7, 8, -8, 1, -1],
            pawn_direction_index: [4, 2, 0],
        }
    }
}
