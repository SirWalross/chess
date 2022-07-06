use crate::{Board, Piece};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PieceList {
    map: [u8; 32],               // index of piece in pieces list from uuid of piece
    pieces: [Vec<(u8, u8)>; 12], // uuid and position of pieces
}

impl PieceList {
    pub fn new() -> PieceList {
        PieceList {
            map: [0; 32],
            pieces: [
                Vec::with_capacity(8), // pawn
                Vec::with_capacity(8), // bishop
                Vec::with_capacity(8), // knight
                Vec::with_capacity(8), // rook
                Vec::with_capacity(8), // queen
                Vec::with_capacity(1), // king
                Vec::with_capacity(8), // pawn
                Vec::with_capacity(8), // bishop
                Vec::with_capacity(8), // knight
                Vec::with_capacity(8), // rook
                Vec::with_capacity(8), // queen
                Vec::with_capacity(1), // king
            ],
        }
    }

    pub fn populate(&mut self, board: &[Piece; 8 * 8]) {
        for m in &mut self.map {
            *m = 0;
        }
        for p in &mut self.pieces {
            p.clear();
        }
        board
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.is_empty())
            .for_each(|(i, p)| {
                self.map[p.uuid as usize] = self.pieces[p.to_piecelist_index()].len() as u8;
                self.pieces[p.to_piecelist_index()].push((p.uuid, i as u8));
            });
        print!("");
    }

    pub fn remove(&mut self, piece: &Piece) {
        debug_assert!(!piece.is_empty());
        let piece_index = piece.to_piecelist_index();
        let index = self.map[piece.uuid as usize];
        if self.pieces[piece_index].len() as u8 - 1 != index {
            self.pieces[piece_index][index as usize] = self.pieces[piece_index].pop().unwrap();
            self.map[self.pieces[piece_index][index as usize].0 as usize] = index;
        } else {
            self.pieces[piece_index].pop();
        }
    }

    pub fn add(&mut self, piece: &Piece, pos: u8) {
        let index = piece.to_piecelist_index();
        self.pieces[index].push((piece.uuid, pos));
        self.map[piece.uuid as usize] = self.pieces[index].len() as u8 - 1;
    }

    pub fn from_piece(&self, piece: &Piece) -> &Vec<(u8, u8)> {
        &self.pieces[piece.to_piecelist_index()]
    }

    pub fn from_type(&self, piece: i8) -> &Vec<(u8, u8)> {
        &self.pieces[piece.abs() as usize - 1 + 3 * (piece.signum() + 1) as usize]
    }

    pub fn _move(&mut self, piece: &Piece, target: u8) {
        debug_assert!(!piece.is_empty());
        self.pieces[piece.to_piecelist_index()][self.map[piece.uuid as usize] as usize] =
            (piece.uuid, target);
    }

    pub fn pawn_count(&self) -> usize {
        self.pieces[0].len() + self.pieces[6].len()
    }

    pub fn bishop_count(&self) -> usize {
        self.pieces[1].len() + self.pieces[7].len()
    }

    pub fn knight_count(&self) -> usize {
        self.pieces[2].len() + self.pieces[8].len()
    }

    pub fn rook_count(&self) -> usize {
        self.pieces[3].len() + self.pieces[9].len()
    }

    pub fn queen_count(&self) -> usize {
        self.pieces[4].len() + self.pieces[10].len()
    }
}
