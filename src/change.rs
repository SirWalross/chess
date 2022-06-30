use crate::Piece;

pub(crate) struct Change {
    pub(crate) start: u8,
    pub(crate) end: u8,
    pub(crate) captured: Piece,
    pub(crate) captured_position: u8,
    pub(crate) two_square_advance: u8,
    pub(crate) fifty_move_counter: u8,
    pub(crate) promotion: bool,
    pub(crate) not_able_to_castle: u8,
    pub(crate) castling: u8,
}

impl Change {
    pub(crate) fn new(
        start: u8,
        end: u8,
        captured: Piece,
        captured_position: u8,
        two_square_advance: u8,
        fifty_move_counter: u8,
        promotion: bool,
        not_able_to_castle: u8,
        castling: u8,
    ) -> Self {
        Self {
            start,
            end,
            captured,
            captured_position,
            two_square_advance,
            fifty_move_counter,
            promotion,
            not_able_to_castle,
            castling,
        }
    }
}