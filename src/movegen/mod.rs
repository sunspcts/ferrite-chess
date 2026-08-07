mod pawns;

use crate::{attacks::*, bitboard::Bitboard, moves::*, piece::Piece};

use crate::board::{Board, Side};

//Pseudolegal move generation. Legality checking is done at the make_move() stage.

impl Board {
    pub fn generate_pseudolegal_moves(&self, moves: &mut MoveList) {
        moves.clear();
        let side = self.game_state.active_side;
        self.generate_pawn_moves(moves);
        self.generate_castling_moves(moves);

        // Might eventually move these loops into the methods.
        for king in self.piece_bb[side as usize][Piece::King as usize] {
            self.generate_leaper_moves(king, moves, Piece::King);
        }

        for knight in self.piece_bb[side as usize][Piece::Knight as usize] {
            self.generate_leaper_moves(knight, moves, Piece::Knight);
        }

        for bishop in self.piece_bb[side as usize][Piece::Bishop as usize] {
            self.generate_slider_moves(bishop, moves, Piece::Bishop);
        }

        for rook in self.piece_bb[side as usize][Piece::Rook as usize] {
            self.generate_slider_moves(rook, moves, Piece::Rook);
        }

        for queen in self.piece_bb[side as usize][Piece::Queen as usize] {
            self.generate_slider_moves(queen, moves, Piece::Queen);
        }
    }

    pub fn generate_pseudolegal_moves_list(&self) -> MoveList {
        let mut moves = MoveList::default();
        self.generate_pseudolegal_moves(&mut moves);
        moves
    }

    pub fn generate_leaper_moves(
        &self,
        from_sq: u16,
        moves: &mut MoveList,
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
        moves: &mut MoveList,
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

    pub fn generate_castling_moves(
        &self,
        moves: &mut MoveList,
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