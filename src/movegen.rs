use crate::{attacks::*, bitboard::Bitboard, board::{Board, Side}, moves::*, piece::Piece};
const PROMOTION_FLAGS: [u16; 4] = [move_flags::KNIGHT_PROMO, move_flags::BISHOP_PROMO, move_flags::ROOK_PROMO, move_flags::QUEEN_PROMO];

const A_FILE_BB: Bitboard = Bitboard::new(0x0101010101010101);
const H_FILE_BB: Bitboard = Bitboard::new(0x8080808080808080);
const RANK_3_BB: Bitboard = Bitboard::new(0x0000000000FF0000);
const RANK_6_BB: Bitboard = Bitboard::new(0x0000FF0000000000);
const PROMOTION_RANKS_BB: Bitboard = Bitboard::new(0xFF000000000000FF);

//Pseudolegal move generation. Legality checking is done at the make_move() stage.

impl Board {
    pub fn generate_pseudolegal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(256); 
        let side = self.game_state.active_side;
        self.generate_pawn_moves(&mut moves);
        self.generate_castling_moves(&mut moves);

        for king in self.piece_bb[side as usize][Piece::King as usize] {
            self.generate_leaper_moves(king, &mut moves, Piece::King);
        }

        for knight in self.piece_bb[side as usize][Piece::Knight as usize] {
            self.generate_leaper_moves(knight, &mut moves, Piece::Knight);
        }

        for bishop in self.piece_bb[side as usize][Piece::Bishop as usize] {
            self.generate_slider_moves(bishop, &mut moves, Piece::Bishop);
        }

        for rook in self.piece_bb[side as usize][Piece::Rook as usize] {
            self.generate_slider_moves(rook, &mut moves, Piece::Rook);
        }

        for queen in self.piece_bb[side as usize][Piece::Queen as usize] {
            self.generate_slider_moves(queen, &mut moves, Piece::Queen);
        }

        moves
    }

    pub fn generate_leaper_moves(
        &self,
        from_sq: u16, 
        moves: &mut Vec<Move>,
        piece: Piece,
    ) {
        let side = self.game_state.active_side;
        let friendly_pieces = self.side_bb[side as usize];
        let enemy_pieces = self.side_bb[(side as usize) ^ 1];
        
        let raw_attacks = match piece {
            Piece::Knight => KNIGHT_ATTACKS[from_sq as usize],
            Piece::King => KING_ATTACKS[from_sq as usize],
            _ => panic!(),
        };

        let valid_moves = raw_attacks & !friendly_pieces;
        let captures = valid_moves & enemy_pieces;
        let quiets = valid_moves ^ captures;
        
        for to_sq in captures {
            moves.push(Move::new(self, from_sq, to_sq, move_flags::CAPTURE, piece)); 
        }

        for to_sq in quiets {
            moves.push(Move::new(self, from_sq, to_sq, move_flags::QUIET, piece)); 
        }
    }

    pub fn generate_slider_moves(
        &self,
        from_sq: u16,
        moves: &mut Vec<Move>,
        piece: Piece,
    ) {
        let side = self.game_state.active_side;
        let friendly_pieces = self.side_bb[side as usize];
        let enemy_pieces = self.side_bb[(side as usize) ^ 1]; 
        let occupancy = friendly_pieces | enemy_pieces;

        let dirs: &[usize] = match piece {
            Piece::Rook => &[0,1,2,3],
            Piece::Bishop => &[4,5,6,7],
            Piece::Queen => &[0,1,2,3,4,5,6,7], 
            _ => panic!("Piece passed to generate_slider_moves is not a slider!"),
        };

        let mut raw_attacks: Bitboard = Bitboard::zero();

        for &dir in dirs {
            raw_attacks |= get_ray_attacks(from_sq, dir, occupancy)
        }

        let valid_moves = raw_attacks & !friendly_pieces;
        let captures = valid_moves & enemy_pieces;
        let quiets = valid_moves ^ captures;
        
        for to_sq in captures {
            moves.push(Move::new(self, from_sq, to_sq, move_flags::CAPTURE, piece)); 
        }

        for to_sq in quiets {
            moves.push(Move::new(self, from_sq, to_sq, move_flags::QUIET, piece)); 
        }
    }

    // directly lifted from old implementation, not ideal but it's fine.
    pub fn generate_pawn_moves(
        &self,
        moves: &mut Vec<Move>,
    ) {
        let side = self.game_state.active_side;
        let pawns = self.piece_bb[side as usize][Piece::Pawn as usize];
        let enemy_pieces = self.side_bb[(side as usize) ^ 1]; //this is disgusting but it's kinda a funny way!
        let empty = !(self.side_bb[side as usize] | enemy_pieces);

        let ep_square = self.game_state.en_passant_square;
        let ep_square_bb = Bitboard::new(ep_square.map_or(0, |x| 1u64 << x));

        let attackables = enemy_pieces | ep_square_bb;
        let single_pushes: Bitboard;
        let double_pushes: Bitboard;
        let captures_left: Bitboard;
        let captures_right: Bitboard;

        if side == Side::White {
            single_pushes = (pawns << 8) & empty;
            double_pushes = ((single_pushes & RANK_3_BB) << 8) & empty;
            captures_left = ((pawns & !A_FILE_BB) << 7) & attackables;
            captures_right = ((pawns & !H_FILE_BB) << 9) & attackables;
        } else {
            single_pushes = (pawns >> 8) & empty;
            double_pushes = ((single_pushes & RANK_6_BB) >> 8) & empty;
            captures_left = ((pawns & !A_FILE_BB) >> 7) & attackables;
            captures_right = ((pawns & !H_FILE_BB) >> 9) & attackables;
        }

        let promotion_bb = PROMOTION_RANKS_BB;

        let promo_pushes = single_pushes & promotion_bb;
        let single_pushes = single_pushes & !promotion_bb;

        let promo_caps_left = captures_left & promotion_bb;
        let ep_capture_left = captures_left & ep_square_bb;
        let captures_left = captures_left & !promotion_bb & !ep_square_bb;

        let promo_caps_right = captures_right & promotion_bb;
        let ep_capture_right = captures_right & ep_square_bb;
        let captures_right = captures_right & !promotion_bb & !ep_square_bb;


        let (offset_push, offset_cap_left, offset_cap_right) = if side == Side::White {
            (8, 7, 9)
        } else {
            (-8, -7, -9)
        };

        //Lots of cases!
        pawn_move_helper(self, single_pushes, offset_push, move_flags::QUIET, false, moves);
        pawn_move_helper(self, double_pushes, offset_push * 2, move_flags::DOUBLE_PAWN_PUSH, false, moves);
        pawn_move_helper(self, captures_left, offset_cap_left, move_flags::CAPTURE, false, moves);
        pawn_move_helper(self, captures_right, offset_cap_right, move_flags::CAPTURE, false, moves);
        pawn_move_helper(self, promo_pushes, offset_push, move_flags::QUIET, true, moves);
        pawn_move_helper(self, promo_caps_left, offset_cap_left, move_flags::CAPTURE, true, moves);
        pawn_move_helper(self, promo_caps_right, offset_cap_right, move_flags::CAPTURE, true, moves);
        pawn_move_helper(self, ep_capture_left, offset_cap_left, move_flags::EP_CAPTURE, false, moves);
        pawn_move_helper(self, ep_capture_right, offset_cap_right, move_flags::EP_CAPTURE, false, moves);
    }

    pub fn generate_castling_moves(
        &self,
        moves: &mut Vec<Move>, 
    ) {
        let side = self.game_state.active_side;
        let all_pieces = self.side_bb[0] | self.side_bb[1];

        if side == Side::White {
            if self.game_state.castling & 1 != 0 {
                if (all_pieces & Bitboard::new(0x60)) == Bitboard::zero() {
                    moves.push(Move::new(self, 4, 6, move_flags::KING_CASTLE, Piece::King)); // E1 to G1
                }
            }
            if self.game_state.castling & 2 != 0 {
                if (all_pieces & Bitboard::new(0x0E)) == Bitboard::zero() {
                    moves.push(Move::new(self, 4, 2, move_flags::QUEEN_CASTLE, Piece::King)); // E1 to C1
                }
            }
        } else {
            if self.game_state.castling & 4 != 0 {
                if (all_pieces & Bitboard::new(0x6000000000000000)) == Bitboard::zero() {
                    moves.push(Move::new(self, 60, 62, move_flags::KING_CASTLE, Piece::King)); // E8 to G8
                }
            }
            if self.game_state.castling & 8 != 0 {
                if (all_pieces & Bitboard::new(0x0E00000000000000)) == Bitboard::zero() {
                    moves.push(Move::new(self, 60, 58, move_flags::QUEEN_CASTLE, Piece::King)); // E8 to C8
                }
            }
        }
    }
}

fn pawn_move_helper(board: &Board, dest_bb: Bitboard, offset: i16, flag: u16, is_promotion: bool, moves: &mut Vec<Move>) {
    if is_promotion {
        for to_sq in dest_bb {
            let from_sq = (to_sq as i16 - offset) as u16;
            for pflag in PROMOTION_FLAGS {
                moves.push(Move::new(board, from_sq, to_sq, flag | pflag, Piece::Pawn));
            }
        }
    } else {
        for to_sq in dest_bb {
            let from_sq = (to_sq as i16 - offset) as u16;
            moves.push(Move::new(board, from_sq, to_sq, flag, Piece::Pawn));
        }
    }
}