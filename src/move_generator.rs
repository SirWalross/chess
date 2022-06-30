use crate::{Board, Move, Piece, _move::MoveFlags, board::Data};

pub(crate) struct MoveGenerator {
    pub(crate) moves: Vec<Move>,
    pub(crate) fields_under_attack: [bool; 8 * 8],
    pub(crate) attacking_rays: [bool; 8 * 8], // positions that could block check, if in_check
    pinned: u16,
    pinned_dir: u32, // direction_index for each piece, differentiates only between up, right, diagonal up and diagonal down
    color: i8,
    in_check: bool,
    king_index: u8,
    enemy_king_index: u8,
    in_double_check: bool,
}

impl MoveGenerator {
    pub(crate) fn new() -> MoveGenerator {
        MoveGenerator {
            moves: Vec::new(),
            fields_under_attack: [false; 8 * 8],
            attacking_rays: [false; 8 * 8],
            pinned: 0,
            pinned_dir: 0,
            color: 0,
            in_check: false,
            king_index: 0,
            enemy_king_index: 0,
            in_double_check: false,
        }
    }

    fn calculate_attack_data(&mut self, board: &Data) {
        for i in 0..8 {
            let _move = board.piece_moves.sliding[i];
            let dest = _move * self.color + self.king_index;

            if _move.in_bounds(self.king_index, self.color)
            {
                self.fields_under_attack[dest as usize] = true;
                self.attacking_rays[dest as usize] = true;
            }
        }
    }

    pub(crate) fn generate_moves(&mut self, board: &Data) {
        self.moves.clear();
        self.moves.reserve(40);
        for f in &mut self.fields_under_attack {
            *f = false;
        }
        for r in &mut self.attacking_rays {
            *r = false;
        }

        self.in_check = false;
        self.in_double_check = false;
        self.color = !board.white_turn as i8 * 2 - 1;
        self.king_index = board
            .piece_list
            .from_type((!board.white_turn as i8 * 2 - 1) * 6)[0];
        self.enemy_king_index = board
            .piece_list
            .from_type((board.white_turn as i8 * 2 - 1) * 6)[0];

        self.calculate_attack_data(board);

        self.color = board.white_turn as i8 * 2 - 1;
        self.enemy_king_index = self.king_index;

        self.generate_king_moves(board);

        // Only king moves are valid in a double check position, so can return early.
        if self.in_double_check {
            return;
        }

        self.generate_sliding_moves(board);
        self.generate_knight_moves(board);
        self.generate_pawn_moves(board);
    }

    fn generate_king_moves(&mut self, board: &Data) {
        let index = self.king_index;
        for i in 0..8 {
            let _move = board.piece_moves.sliding[i];
            let dest = _move * self.color + index;

            if _move.in_bounds(index, self.color)
                && !board.board[dest as usize].is_color(self.color)
                && !self.fields_under_attack[dest as usize]
            {
                self.moves.push(Move::from_flags(index, dest, 0));

                if !self.in_check && board.board[dest as usize].is_empty() {
                    // Castle kingside
                    if i == 5
                        && board.not_able_to_castle & 0x01 << (2 * (!board.white_turn as u8)) == 0
                        && board.board[dest as usize + 1].is_empty()
                        && !self.fields_under_attack[dest as usize + 1]
                    {
                        self.moves.push(Move::from_flags(
                            self.king_index,
                            dest + 1,
                            MoveFlags::KINGSIDE_CASTLING,
                        ));
                    }
                }
                // Castle queenside
                else if i == 7
                    && board.not_able_to_castle & 0x02 << (2 * (!board.white_turn as u8)) == 0
                    && board.board[dest as usize - 1].is_empty()
                    && board.board[dest as usize - 2].is_empty()
                    && !self.fields_under_attack[dest as usize - 1]
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

    fn generate_knight_moves(&mut self, board: &Data) {
        for index in board.piece_list.from_type(self.color * 3) {
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
                    && (self.attacking_rays[*index as usize] || !self.in_check)
                {
                    self.moves.push(Move::from_flags(*index, dest, 0));
                }
            }
        }
    }
    fn generate_sliding_moves(&mut self, board: &Data) {
        for index in board.piece_list.from_type(self.color * 5) {
            // queen
            self.generate_sliding_piece_moves(board, *index, 0, 8);
        }
        for index in board.piece_list.from_type(self.color * 4) {
            // rook
            self.generate_sliding_piece_moves(board, *index, 4, 8);
        }
        for index in board.piece_list.from_type(self.color * 2) {
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

        for direction_index in start..=end {
            if pinned && !self.moving_along_pinned_dir(uuid, direction_index) {
                // if not moving along pinned direction
                continue;
            }

            let _move = board.piece_moves.sliding[direction_index as usize];

            for n in 1..8 {
                if !_move.in_bounds(index, self.color) {
                    break;
                }

                let dest = _move * n + index;
                let target = &board.board[dest as usize];

                let is_capture = !target.is_empty();

                let preventing_check =
                    self.in_check && self.attacking_rays[index as usize];

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

        for index in board.piece_list.from_type(self.color * 1) {
            let uuid = board.board[*index as usize].uuid & 0x0f;
            for i in 0..3 {
                let _move = board.piece_moves.pawn_moves[i];
                let dest = _move * self.color + *index;
                let captured = &board.board[dest as usize];
                let dest_one_forward = (dest as i8 + 8 * self.color) as u8;

                if !_move.in_bounds(*index, self.color) {
                    continue;
                }

                if self.pinned & (1 << uuid) != 0
                    && !self.moving_along_pinned_dir(
                        uuid,
                        board.piece_moves.pawn_direction_index[i as usize],
                    )
                {
                    // if not moving along pinned direction
                    continue;
                }

                let preventing_check =
                    self.in_check && self.attacking_rays[*index as usize];

                if i == 0 && captured.is_empty() && (!self.in_check || preventing_check) {
                    if dest / 8 == penultimate_file {
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

                        if dest / 8 == start_file
                            && board.board[dest_one_forward as usize].is_empty()
                        {
                            self.moves.push(Move::from_flags(
                                *index,
                                dest_one_forward,
                                MoveFlags::TWO_SQUARE_ADVANCE,
                            ));
                        }
                    }
                } else if i != 0 && captured.is_color(-self.color) {
                    // regular capture
                    if dest / 8 == penultimate_file {
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
                } else if i != 0 && dest == board.two_square_advance {
                    let captured = (dest as i8 - 8 * self.color) as u8;
                    if !self.in_check_after_en_passant(*index, dest, captured) {
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
        (self.pinned_dir >> (30 - 2 * uuid)) as u8 & 0x03 != direction_index / 2
    }

    fn in_check_after_en_passant(&self, start: u8, dest: u8, captured: u8) -> bool {
        // TODO:
        false
    }
}
