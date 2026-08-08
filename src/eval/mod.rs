mod psts;
mod pawn_structure;

use psts::*;
use pawn_structure::*;
use crate::{board::Board, piece::Piece};

const ISOLATED_PAWN_MG: i64 = -10;
const ISOLATED_PAWN_EG: i64 = -20;

pub fn eval(board: &Board) -> i64 {
    let mut phase = 0;
    for color in 0..2 {
        for (piece, bb) in board.piece_bb[color].iter().enumerate() {
            phase += PIECE_PHASE[piece] * bb.count_ones() as i64;
        }
    }
    let mg_phase = phase.min(MAX_PHASE);
    let eg_phase = MAX_PHASE - mg_phase;

    let mut score = 0;

    for (piece, bb) in board.piece_bb[0].iter().enumerate() {
        score += calc_tapered_score(piece, mg_phase, *bb, 56);
    }

    for (piece, bb) in board.piece_bb[1].iter().enumerate() {
        score -= calc_tapered_score(piece, mg_phase, *bb, 0);
    }

    // Isolated pawn penalties
    let white_pawns = board.piece_bb[0][Piece::Pawn as usize];
    for sq in white_pawns {
        if is_isolated(sq, white_pawns) {
            score += (ISOLATED_PAWN_MG * mg_phase + ISOLATED_PAWN_EG * eg_phase) / MAX_PHASE;
        }
    }

    let black_pawns = board.piece_bb[1][Piece::Pawn as usize];
    for sq in black_pawns {
        if is_isolated(sq, black_pawns) {
            score -= (ISOLATED_PAWN_MG * mg_phase + ISOLATED_PAWN_EG * eg_phase) / MAX_PHASE;
        }
    }

    score * board.side_to_move_multiplier()
}