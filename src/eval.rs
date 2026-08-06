use crate::{board::Board, piece::Piece};

const PAWN_VALUE: i64 = 82;
const KNIGHT_VALUE: i64 = 337;
const BISHOP_VALUE: i64 = 365;
const ROOK_VALUE: i64 = 477;
const QUEEN_VALUE: i64 = 1025;

pub fn piece_value_i64(piece: Piece) -> i64 {
    match piece {
        Piece::Pawn => PAWN_VALUE,
        Piece::Knight => KNIGHT_VALUE,
        Piece::Bishop => BISHOP_VALUE,
        Piece::Rook => ROOK_VALUE,
        Piece::Queen => QUEEN_VALUE,
        _ => 0,
    }
}

//PeSTO middlegame tables.
const PAWN_PST: [i64; 64] = [
      0,   0,   0,   0,   0,   0,  0,   0, // 8
     98, 134,  61,  95,  68, 126, 34, -11,
     -6,   7,  26,  31,  65,  56, 25, -20,
    -14,  13,   6,  21,  23,  12, 17, -23,
    -27,  -2,  -5,  12,  17,   6, 10, -25,
    -26,  -4,  -4, -10,   3,   3, 33, -12,
    -35,  -1, -20, -23, -15,  24, 38, -22,
      0,   0,   0,   0,   0,   0,  0,   0, // 1
];

const KNIGHT_PST: [i64; 64] = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
];

const BISHOP_PST: [i64; 64] = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
];

const ROOK_PST: [i64; 64] = [
     32,  42,  32,  51, 63,  9,  31,  43,
     27,  32,  58,  62, 80, 67,  26,  44,
     -5,  19,  26,  36, 17, 45,  61,  16,
    -24, -11,   7,  26, 24, 35,  -8, -20,
    -36, -26, -12,  -1,  9, -7,   6, -23,
    -45, -25, -16, -17,  3,  0,  -5, -33,
    -44, -16, -20,  -9, -1, 11,  -6, -71,
    -19, -13,   1,  17, 16,  7, -37, -26,
];

const QUEEN_PST: [i64; 64] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,
];

const KING_PST: [i64; 64] = [
    -50, -40, -40, -50, -50, -40, -40, -50,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -30, -40, -40, -50, -50, -40, -40, -30,
    -20, -30, -30, -40, -40, -30, -30, -20,
    -10, -20, -20, -20, -20, -20, -20, -10,
     20,  20,   0,   0,   0,   0,  20,  20,
     20,  30,  10,   0,   0,  10,  30,  20,
];

pub fn eval(board: &Board) -> i64 {
    let mut score = 0;

    for sq in board.piece_bb[0][Piece::Pawn as usize] {
        score += PAWN_VALUE + PAWN_PST[(sq ^ 56) as usize];
    }
    for sq in board.piece_bb[0][Piece::Knight as usize] {
        score += KNIGHT_VALUE + KNIGHT_PST[(sq ^ 56) as usize];
    }
    for sq in board.piece_bb[0][Piece::Bishop as usize] {
        score += BISHOP_VALUE + BISHOP_PST[(sq ^ 56) as usize];
    }
    for sq in board.piece_bb[0][Piece::Rook as usize] {
        score += ROOK_VALUE + ROOK_PST[(sq ^ 56) as usize];
    }
    for sq in board.piece_bb[0][Piece::Queen as usize] {
        score += QUEEN_VALUE + QUEEN_PST[(sq ^ 56) as usize];
    }
    for sq in board.piece_bb[0][Piece::King as usize] {
        score += KING_PST[(sq ^ 56) as usize];
    }

    for sq in board.piece_bb[1][Piece::Pawn as usize] {
        score -= PAWN_VALUE + PAWN_PST[sq as usize];
    }
    for sq in board.piece_bb[1][Piece::Knight as usize] {
        score -= KNIGHT_VALUE + KNIGHT_PST[sq as usize];
    }
    for sq in board.piece_bb[1][Piece::Bishop as usize] {
        score -= BISHOP_VALUE + BISHOP_PST[sq as usize];
    }
    for sq in board.piece_bb[1][Piece::Rook as usize] {
        score -= ROOK_VALUE + ROOK_PST[sq as usize];
    }
    for sq in board.piece_bb[1][Piece::Queen as usize] {
        score -= QUEEN_VALUE + QUEEN_PST[sq as usize];
    }
    for sq in board.piece_bb[1][Piece::King as usize] {
        score -= KING_PST[sq as usize];
    }

    score * board.side_to_move_multiplier()
}