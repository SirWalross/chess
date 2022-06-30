use crate::{Board, Piece};

pub(crate) struct PieceList {
    map: [u8; 32], // index of piece in pieces list from uuid of piece
    pieces: [Vec<u8>; 12],
}

impl PieceList {
    pub fn new() -> PieceList {
        PieceList {
            map: [0; 32],
            pieces: [
                Vec::with_capacity(1),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
                Vec::with_capacity(1),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
                Vec::with_capacity(8),
            ],
        }
    }

    pub fn populate(&mut self, board: &[Piece; 8 * 8]) {
        board
            .iter()
            .filter(|p| !p.is_empty())
            .enumerate()
            .for_each(|(i, p)| {
                self.pieces[p.to_piecelist_index()]
                    .push(i as u8)
            })
    }

    pub fn remove(&mut self, piece: &Piece) {
        debug_assert!(!piece.is_empty());
        let piece_index = piece.to_piecelist_index();
        let index = self.map[piece.uuid as usize];
        self.pieces[piece_index][index as usize] = self.pieces[piece_index].pop().unwrap();
        self.map[self.pieces[piece_index][index as usize] as usize] = index;
    }

    pub fn add(&mut self, piece: &Piece, pos: u8) {
        let index = piece.to_piecelist_index();
        self.pieces[index].push(pos);
        self.map[piece.uuid as usize] = self.pieces[index].len() as u8 - 1;
    }

    pub fn from_piece(&self, piece: &Piece) -> &Vec<u8> {
        &self.pieces[piece.to_piecelist_index()]
    }

    pub fn from_type(&self, piece: i8) -> &Vec<u8> {
        &self.pieces[piece.abs() as usize + 3 * (piece.signum() + 1) as usize]
    }

    pub fn _move(&mut self, piece: &Piece, target: u8) {
        debug_assert!(!piece.is_empty());
        self.pieces[piece.to_piecelist_index()][self.map[piece.uuid as usize] as usize] = target;
    }
}
