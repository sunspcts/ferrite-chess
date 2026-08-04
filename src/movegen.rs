use crate::{attacks::*, board::Board, moves::*, piece::Piece};

//Pseudolegal move generation. Legality checking is done at the make_move() stage.

impl Board {
    pub fn generate_pseudolegal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(256); 
        let side = self.game_state.active_side;

        for knight in self.piece_bb[side as usize][Piece::Knight as usize] {
            self.generate_leaper_moves(knight, &mut moves, Piece::Knight);
        }

        for king in self.piece_bb[side as usize][Piece::King as usize] {
            self.generate_leaper_moves(king, &mut moves, Piece::King);
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

}