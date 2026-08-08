use super::*;

use crate::{attacks::get_ray_attacks, bitboard::Bitboard, moves::MoveList, piece::Piece};

impl Board {
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

    pub fn generate_slider_captures(
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
            _ => panic!("Piece passed to generate_slider_captures is not a slider!"),
        };

        let mut raw_attacks: Bitboard = Bitboard::zero();

        for &dir in dirs {
            raw_attacks |= get_ray_attacks(from_sq, dir, occupancy);
        }

        let captures = raw_attacks & enemy_pieces;

        for to_sq in captures {
            moves.push(Move::new(self, from_sq, to_sq, move_flags::CAPTURE, piece));
        }
    }
}