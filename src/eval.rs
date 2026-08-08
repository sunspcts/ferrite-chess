mod psts;

use psts::*;

use crate::board::Board;

const PAWN_VALUE: i64 = 82;
const KNIGHT_VALUE: i64 = 337;
const BISHOP_VALUE: i64 = 365;
const ROOK_VALUE: i64 = 477;
const QUEEN_VALUE: i64 = 1025;

const PIECE_VALUES: [i64; 6] = [PAWN_VALUE, KNIGHT_VALUE, BISHOP_VALUE, ROOK_VALUE, QUEEN_VALUE, 0];

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
        score += calc_tapered_score(piece, mg_phase, *bb, 56, PIECE_VALUES[piece]);
    }

    for (piece, bb) in board.piece_bb[1].iter().enumerate() {
        score -= calc_tapered_score(piece, mg_phase, *bb, 0, PIECE_VALUES[piece]);
    }

    score * board.side_to_move_multiplier()
}