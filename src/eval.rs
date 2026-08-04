use crate::{board::Board, piece::Piece};

const PAWN_VALUE: i64 = 100;
const KNIGHT_VALUE: i64 = 300;
const BISHOP_VALUE: i64 = 300;
const ROOK_VALUE: i64 = 500;
const QUEEN_VALUE: i64 = 900;

// Naive, but good enough to implement search.
pub fn eval(board: &Board) -> i64 {
    let mut score = 0;

    score += board.piece_bb[0][Piece::Pawn as usize].count_ones() as i64 * PAWN_VALUE;
    score += board.piece_bb[0][Piece::Knight as usize].count_ones() as i64 * KNIGHT_VALUE;
    score += board.piece_bb[0][Piece::Bishop as usize].count_ones() as i64 * BISHOP_VALUE;
    score += board.piece_bb[0][Piece::Rook as usize].count_ones() as i64 * ROOK_VALUE;
    score += board.piece_bb[0][Piece::Queen as usize].count_ones() as i64 * QUEEN_VALUE;

    score -= board.piece_bb[1][Piece::Pawn as usize].count_ones() as i64 * PAWN_VALUE;
    score -= board.piece_bb[1][Piece::Knight as usize].count_ones() as i64 * KNIGHT_VALUE;
    score -= board.piece_bb[1][Piece::Bishop as usize].count_ones() as i64 * BISHOP_VALUE;
    score -= board.piece_bb[1][Piece::Rook as usize].count_ones() as i64 * ROOK_VALUE;
    score -= board.piece_bb[1][Piece::Queen as usize].count_ones() as i64 * QUEEN_VALUE;

    score * board.side_to_move_multiplier()
}