use crate::{
    Board, Move, Piece, _move::MoveFlags, board::Data, piece_moves::Direction, position::Position,
};

pub(crate) struct MoveGenerator {
    pub(crate) moves: Vec<Move>,
    pub(crate) fields_under_attack: u64,
    pub(crate) attacking_rays: u64, // positions that could block check, if in_check
    pinned: u16,
    pinned_dir: u32, // direction_index for each piece, differentiates only between up, right, diagonal up and diagonal down
    color: i8,
    pub(crate) in_check: bool,
    king_index: u8,
    enemy_king_index: u8,
    in_double_check: bool,
}

impl MoveGenerator {
    pub(crate) fn new() -> MoveGenerator {
        MoveGenerator {
            moves: Vec::new(),
            fields_under_attack: 0,
            attacking_rays: 0,
            pinned: 0,
            pinned_dir: 0,
            color: 0,
            in_check: false,
            king_index: 0,
            enemy_king_index: 0,
            in_double_check: false,
        }
    }

    pub(crate) fn generate_moves(&mut self, board: &Data) {
        self.moves.clear();
        self.moves.reserve(40);

        self.fields_under_attack = 0;
        self.attacking_rays = 0;
        self.pinned = 0;
        self.pinned_dir = 0;
        self.in_check = false;
        self.in_double_check = false;
        self.color = !board.white_turn as i8 * 2 - 1;
        self.king_index = board
            .piece_list
            .from_type((!board.white_turn as i8 * 2 - 1) * 6)[0]
            .1;
        self.enemy_king_index = board
            .piece_list
            .from_type((board.white_turn as i8 * 2 - 1) * 6)[0]
            .1;

        self.calculate_attack_data(board);

        self.color = board.white_turn as i8 * 2 - 1;
        self.king_index = self.enemy_king_index;

        self.generate_king_moves(board);

        // Only king moves are valid in a double check position, so can return early.
        if self.in_double_check {
            return;
        }

        self.generate_sliding_moves(board);
        self.generate_knight_moves(board);
        self.generate_pawn_moves(board);
    }

    fn calculate_attack_data(&mut self, board: &Data) {
        // king
        for i in 0..8 {
            let _move = board.piece_moves.sliding[i as usize];
            let dest = _move * self.color + self.king_index;

            if _move.in_bounds(self.king_index, self.color) {
                self.fields_under_attack |= 1 << dest;
            }
        }
        // knight
        for (_, index) in board.piece_list.from_type(self.color * 3) {
            for i in 0..8 {
                let _move = board.piece_moves.knight_moves[i];
                let dest = _move * self.color + *index;

                if _move.in_bounds(*index, self.color) {
                    self.fields_under_attack |= 1 << dest;

                    if dest == self.enemy_king_index {
                        self.in_double_check = self.in_check;
                        self.in_check = true;
                        self.attacking_rays |= 1 << *index;
                    }
                }
            }
        }
        // pawn
        for (_, index) in board.piece_list.from_type(self.color * 1) {
            for i in 1..3 {
                let _move = board.piece_moves.pawn_moves[i];
                let dest = _move * self.color + *index;

                if _move.in_bounds(*index, self.color) {
                    self.fields_under_attack |= 1 << dest;

                    if dest == self.enemy_king_index {
                        self.in_double_check = self.in_check;
                        self.in_check = true;
                        self.attacking_rays |= 1 << *index;
                    }
                }
            }
        }
        // sliding moves
        for (_, index) in board.piece_list.from_type(self.color * 5) {
            // queen
            self.generate_sliding_attack_moves(board, *index, 0, 8);
        }
        for (_, index) in board.piece_list.from_type(self.color * 4) {
            // rook
            self.generate_sliding_attack_moves(board, *index, 4, 8);
        }
        for (_, index) in board.piece_list.from_type(self.color * 2) {
            // bishop
            self.generate_sliding_attack_moves(board, *index, 0, 4);
        }
    }

    fn generate_sliding_attack_moves(&mut self, board: &Data, index: u8, start: u8, end: u8) {
        for direction_index in start..end {
            let _move = board.piece_moves.sliding[direction_index as usize];
            let attacking_king = self.in_enemy_king_direction(board, index, direction_index);
            let mut pinning_piece = &Piece::empty();

            for n in 1..8 {
                if !_move.in_bounds(index, self.color * n) {
                    break;
                }

                let dest = _move * n * self.color + index;
                let target = &board.board[dest as usize];

                let is_capture = !target.is_empty();
                let enemy_piece = target.is_color(-self.color);

                if dest == self.enemy_king_index && pinning_piece.is_empty() {
                    // direct attack on king
                    self.in_double_check = self.in_check;
                    self.in_check = true;

                    for j in 0..n {
                        self.attacking_rays |= 1
                            << (index as i8
                                + j * board.piece_moves.sliding_offsets[direction_index as usize]
                                    * self.color);
                    }
                    if _move.in_bounds(index, self.color * (n + 1)) {
                        // field directly behind king is also under attack
                        self.fields_under_attack |= 1
                            << (index as i8
                                + (n + 1)
                                    * board.piece_moves.sliding_offsets[direction_index as usize]
                                    * self.color);
                    }
                    break;
                } else if dest == self.enemy_king_index {
                    // indirect attack on king via pinning
                    let uuid = pinning_piece.uuid & 0x0f;
                    self.pinned |= 1 << uuid;
                    self.pinned_dir |= ((direction_index / 2) as u32) << 2 * uuid;
                    break;
                } else if pinning_piece.is_empty() && enemy_piece && attacking_king && is_capture {
                    // pinning piece
                    pinning_piece = target;
                } else if !pinning_piece.is_empty() && is_capture {
                    // second enemy piece in dir
                    break;
                } else if is_capture && pinning_piece.is_empty() {
                    // own piece
                    self.fields_under_attack |= 1 << dest;
                    break;
                } else if !is_capture && pinning_piece.is_empty() {
                    // no piece
                    self.fields_under_attack |= 1 << dest;
                }
            }
        }
    }

    fn generate_king_moves(&mut self, board: &Data) {
        let index = self.king_index;
        for i in 0..8 {
            let _move = board.piece_moves.sliding[i];
            let dest = _move * self.color + index;

            if _move.in_bounds(index, self.color)
                && !board.board[dest as usize].is_color(self.color)
                && !self.fields_under_attack & (1 << dest) != 0
            {
                self.moves.push(Move::from_flags(index, dest, 0));

                if !self.in_check && board.board[dest as usize].is_empty() {
                    // Castle kingside
                    if i == Direction::kingside(self.color)
                        && board.not_able_to_castle & (0x02 << (2 * (!board.white_turn as u8))) == 0
                        && board.board[dest as usize + 1].is_empty()
                        && !self.fields_under_attack & (1 << (dest + 1)) != 0
                    {
                        self.moves.push(Move::from_flags(
                            self.king_index,
                            dest + 1,
                            MoveFlags::KINGSIDE_CASTLING,
                        ));
                    }
                    // Castle queenside
                    else if i == Direction::queenside(self.color)
                        && board.not_able_to_castle & (0x01 << (2 * (!board.white_turn as u8))) == 0
                        && board.board[dest as usize - 1].is_empty()
                        && board.board[dest as usize - 2].is_empty()
                        && !self.fields_under_attack & (1 << (dest - 1)) != 0
                    {
                        self.moves.push(Move::from_flags(
                            self.king_index,
                            dest - 1,
                            MoveFlags::QUEENSIDE_CASTLING,
                        ));
                    }
                }
            }
        }
    }

    fn generate_knight_moves(&mut self, board: &Data) {
        for (_, index) in board.piece_list.from_type(self.color * 3) {
            let uuid = board.board[*index as usize].uuid & 0x0f;
            // Knight cannot move if it is pinned
            if self.pinned & (1 << uuid) != 0 {
                continue;
            }

            for i in 0..8 {
                let _move = board.piece_moves.knight_moves[i];
                let dest = _move * self.color + *index;

                if _move.in_bounds(*index, self.color)
                    && !board.board[dest as usize].is_color(self.color)
                    && (self.attacking_rays & (1 << dest) != 0 || !self.in_check)
                {
                    self.moves.push(Move::from_flags(*index, dest, 0));
                }
            }
        }
    }

    fn generate_sliding_moves(&mut self, board: &Data) {
        for (_, index) in board.piece_list.from_type(self.color * 5) {
            // queen
            self.generate_sliding_piece_moves(board, *index, 0, 8);
        }
        for (_, index) in board.piece_list.from_type(self.color * 4) {
            // rook
            self.generate_sliding_piece_moves(board, *index, 4, 8);
        }
        for (_, index) in board.piece_list.from_type(self.color * 2) {
            // bishop
            self.generate_sliding_piece_moves(board, *index, 0, 4);
        }
    }

    fn generate_sliding_piece_moves(&mut self, board: &Data, index: u8, start: u8, end: u8) {
        let uuid = board.board[index as usize].uuid & 0x0f;
        let pinned = self.pinned & (1 << uuid) != 0;

        // If this piece is pinned, and the king is in check, this piece cannot move
        if pinned && self.in_check {
            return;
        }

        for direction_index in start..end {
            if pinned && !self.moving_along_pinned_dir(uuid, direction_index) {
                // if not moving along pinned direction
                continue;
            }

            let _move = board.piece_moves.sliding[direction_index as usize];

            for n in 1..8 {
                if !_move.in_bounds(index, self.color * n) {
                    break;
                }

                let dest = _move * n * self.color + index;
                let target = &board.board[dest as usize];

                let is_capture = !target.is_empty();

                let preventing_check = self.in_check && self.attacking_rays & (1 << dest) != 0;

                if !target.is_color(self.color) && (!self.in_check || preventing_check) {
                    self.moves.push(Move::from_flags(index, dest, 0));
                }

                if is_capture || preventing_check {
                    break;
                }
            }
        }
    }

    fn generate_pawn_moves(&mut self, board: &Data) {
        let start_file = 1 + (5 * !board.white_turn as u8);
        let penultimate_file = 1 + (5 * board.white_turn as u8);

        for (_, index) in board.piece_list.from_type(self.color * 1) {
            let uuid = board.board[*index as usize].uuid & 0x0f;
            let dest_one_forward = (*index as i8 + 16 * self.color) as u8;
            for i in 0..3 {
                let _move = board.piece_moves.pawn_moves[i];
                let dest = _move * self.color + *index;
                
                if !_move.in_bounds(*index, self.color) {
                    continue;
                }

                let captured = &board.board[dest as usize];

                if self.pinned & (1 << uuid) != 0
                    && !self.moving_along_pinned_dir(
                        uuid,
                        board.piece_moves.pawn_direction_index[i as usize],
                    )
                {
                    // if not moving along pinned direction
                    continue;
                }

                let preventing_check = self.in_check && self.attacking_rays & (1 << dest) != 0;
                let preventing_check_one_forward = self.in_check
                    && dest_one_forward < 64
                    && self.attacking_rays & (1 << dest_one_forward) != 0;

                if i == 0 && captured.is_empty() && (!self.in_check || preventing_check) {
                    if *index / 8 == penultimate_file {
                        self.moves
                            .push(Move::from_flags(*index, dest, MoveFlags::QUEEN_PROMOTION));
                        self.moves
                            .push(Move::from_flags(*index, dest, MoveFlags::ROOK_PROMOTION));
                        self.moves.push(Move::from_flags(
                            *index,
                            dest,
                            MoveFlags::KNIGHT_PROMOTION,
                        ));
                        self.moves.push(Move::from_flags(
                            *index,
                            dest,
                            MoveFlags::BISHOP_PROMOTION,
                        ));
                    } else {
                        self.moves.push(Move::from_flags(*index, dest, 0));
                    }
                }
                if i == 0
                    && captured.is_empty()
                    && *index / 8 == start_file
                    && board.board[dest_one_forward as usize].is_empty()
                    && (!self.in_check || preventing_check_one_forward)
                {
                    self.moves.push(Move::from_flags(
                        *index,
                        dest_one_forward,
                        MoveFlags::TWO_SQUARE_ADVANCE,
                    ));
                } else if i != 0
                    && captured.is_color(-self.color)
                    && (!self.in_check || preventing_check)
                {
                    // regular capture
                    if *index / 8 == penultimate_file {
                        self.moves
                            .push(Move::from_flags(*index, dest, MoveFlags::QUEEN_PROMOTION));
                        self.moves
                            .push(Move::from_flags(*index, dest, MoveFlags::ROOK_PROMOTION));
                        self.moves.push(Move::from_flags(
                            *index,
                            dest,
                            MoveFlags::KNIGHT_PROMOTION,
                        ));
                        self.moves.push(Move::from_flags(
                            *index,
                            dest,
                            MoveFlags::BISHOP_PROMOTION,
                        ));
                    } else {
                        self.moves.push(Move::from_flags(*index, dest, 0));
                    }
                } else if i != 0
                    && dest == board.two_square_advance
                    && dest != 0
                {
                    let captured = (dest as i8 - 8 * self.color) as u8;
                    if !self.in_check_after_en_passant(board, *index, captured) {
                        self.moves
                            .push(Move::from_flags(*index, dest, MoveFlags::EN_PASSANT));
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn moving_along_pinned_dir(&self, uuid: u8, direction_index: u8) -> bool {
        // makes sure that direction_index of for example left and right are the same
        debug_assert!(self.pinned & (1 << uuid) != 0);
        (self.pinned_dir >> (2 * uuid)) as u8 & 0x03 == direction_index / 2
    }

    fn in_check_after_en_passant(&self, board: &Data, start: u8, captured: u8) -> bool {
        if self.in_check {
            // check if to be captured pawn by en passant is the check making piece
            let mut capturing = false;
            for i in 1..=2 {
                let _move = board.piece_moves.pawn_moves[i];
                let dest = _move * self.color + self.king_index;

                if _move.in_bounds(self.king_index, self.color) && dest == captured {
                    // capturing the check making piece
                    capturing = true;
                    break;
                }
            }
            if !capturing {
                // not capturing the pawn
                return true;
            }
        }

        if start / 8 != self.king_index / 8 {
            // not on the same file
            return false;
        }
        let direction = if start % 8 > self.king_index % 8 {
            1
        } else {
            -1
        };
        let step_count = if direction > 0 {
            7 - self.king_index % 8
        } else {
            self.king_index % 8
        };
        if step_count <= 2 {
            // not enough space for enemy
            return false;
        }
        for n in 1..=step_count {
            let dest = (self.king_index as i8 + n as i8 * direction) as u8;
            let piece = &board.board[dest as usize];
            if dest != start && dest != captured && !piece.is_empty() {
                if piece.is_color(-self.color) && (piece.is_rook() || piece.is_queen()) {
                    return true;
                } else {
                    return false;
                }
            }
        }
        false
    }

    #[inline(always)]
    fn in_enemy_king_direction(&self, board: &Data, index: u8, direction_index: u8) -> bool {
        Position::new(
            (self.enemy_king_index as i8 / 8 - index as i8 / 8) * self.color,
            (self.enemy_king_index as i8 % 8 - index as i8 % 8) * self.color,
        )
        .same_direction(board.piece_moves.sliding[direction_index as usize])
    }
}
