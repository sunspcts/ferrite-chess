mod psts;
mod pawn_structure;

use psts::*;
use crate::board::Board;

pub fn eval(board: &Board) -> i64 {
    let mut phase = 0;
    for color in 0..2 {
        for (piece, bb) in board.piece_bb[color].iter().enumerate() {
            phase += PIECE_PHASE[piece] * bb.count_ones() as i64;
        }
    }
    let mg_phase = phase.min(MAX_PHASE);

    let mut score = 0;

    for (piece, bb) in board.piece_bb[0].iter().enumerate() {
        score += calc_tapered_score(piece, mg_phase, *bb, 56);
    }

    for (piece, bb) in board.piece_bb[1].iter().enumerate() {
        score -= calc_tapered_score(piece, mg_phase, *bb, 0);
    }

    score * board.side_to_move_multiplier()
}