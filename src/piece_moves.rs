use crate::position::Position;

/// holds all possible moves for all piece types
pub(crate) struct PieceMoves {
    pub(crate) pawn_moves: [Position; 3],
    pub(crate) knight_moves: [Position; 8],
    pub(crate) sliding: Vec<Position>,
    pub(crate) pawn_direction_index: [u8; 3],
}

impl PieceMoves {
    pub fn new() -> PieceMoves {
        let mut diag: Vec<Position> = (1..2 as i8).map(|i| Position::new(i, i)).collect();
        diag.extend(diag.clone().iter().map(|p| p.rotate_180()));
        diag.extend(diag.clone().iter().map(|p| p.rotate_90()));

        let mut horiz: Vec<Position> = (1..2 as i8).map(|i| Position::new(i, 0)).collect();
        horiz.extend(horiz.clone().iter().map(|p| p.rotate_180()));
        horiz.extend(horiz.clone().iter().map(|p| p.rotate_90()));

        // TODO: ensure that diag[0] == pawn_moves[0] and diag[1] == pawn_moves[1]
        let mut piece_moves = PieceMoves {
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
            sliding: Vec::new(),
            pawn_direction_index: [4, 0, 2],
        };
        piece_moves.sliding.extend(diag.iter());
        piece_moves.sliding.extend(horiz.iter());

        piece_moves
    }
}
