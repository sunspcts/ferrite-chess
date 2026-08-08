mod psts;

use psts::*;

use crate::{board::Board, piece::Piece};

const PAWN_VALUE: i64 = 82;
const KNIGHT_VALUE: i64 = 337;
const BISHOP_VALUE: i64 = 365;
const ROOK_VALUE: i64 = 477;
const QUEEN_VALUE: i64 = 1025;

pub fn eval(board: &Board) -> i64 {
    let mut score = 0;

    for sq in board.piece_bb[0][Piece::Pawn as usize] {
        score += PAWN_VALUE + MG_PAWN_PST[(sq ^ 56) as usize];
    }
    for sq in board.piece_bb[0][Piece::Knight as usize] {
        score += KNIGHT_VALUE + MG_KNIGHT_PST[(sq ^ 56) as usize];
    }
    for sq in board.piece_bb[0][Piece::Bishop as usize] {
        score += BISHOP_VALUE + MG_BISHOP_PST[(sq ^ 56) as usize];
    }
    for sq in board.piece_bb[0][Piece::Rook as usize] {
        score += ROOK_VALUE + MG_ROOK_PST[(sq ^ 56) as usize];
    }
    for sq in board.piece_bb[0][Piece::Queen as usize] {
        score += QUEEN_VALUE + MG_QUEEN_PST[(sq ^ 56) as usize];
    }
    for sq in board.piece_bb[0][Piece::King as usize] {
        score += MG_KING_PST[(sq ^ 56) as usize];
    }

    for sq in board.piece_bb[1][Piece::Pawn as usize] {
        score -= PAWN_VALUE + MG_PAWN_PST[sq as usize];
    }
    for sq in board.piece_bb[1][Piece::Knight as usize] {
        score -= KNIGHT_VALUE + MG_KNIGHT_PST[sq as usize];
    }
    for sq in board.piece_bb[1][Piece::Bishop as usize] {
        score -= BISHOP_VALUE + MG_BISHOP_PST[sq as usize];
    }
    for sq in board.piece_bb[1][Piece::Rook as usize] {
        score -= ROOK_VALUE + MG_ROOK_PST[sq as usize];
    }
    for sq in board.piece_bb[1][Piece::Queen as usize] {
        score -= QUEEN_VALUE + MG_QUEEN_PST[sq as usize];
    }
    for sq in board.piece_bb[1][Piece::King as usize] {
        score -= MG_KING_PST[sq as usize];
    }

    score * board.side_to_move_multiplier()
}