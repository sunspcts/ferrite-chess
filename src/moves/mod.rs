use crate::{board::*, heuristics::*, piece::Piece};
#[cfg(test)]
mod tests;

mod make;
mod movelist;
pub mod format;

pub use movelist::MoveList;
// Data field is structured as follows:
// First 4 bits encode any flags that make_move needs to know.
// Next 6 bits represent the square the piece is moving to.
// Lowest 6 bits represent the square the piece is moving from.

// Heuristics are calculated at movegen, which I might change.

// flags from https://www.chessprogramming.org/Encoding_Moves.
#[allow(dead_code)]
pub mod move_flags {
    pub const QUIET: u16               = 0b0000;
    pub const DOUBLE_PAWN_PUSH: u16    = 0b0001;
    pub const KING_CASTLE: u16         = 0b0010;
    pub const QUEEN_CASTLE: u16        = 0b0011;

    pub const CAPTURE: u16             = 0b0100;
    pub const EP_CAPTURE: u16          = 0b0101;

    pub const KNIGHT_PROMO: u16        = 0b1000;
    pub const BISHOP_PROMO: u16        = 0b1001;
    pub const ROOK_PROMO: u16          = 0b1010;
    pub const QUEEN_PROMO: u16         = 0b1011;

    pub const KNIGHT_PROMO_CAPTURE: u16 = 0b1100;
    pub const BISHOP_PROMO_CAPTURE: u16 = 0b1101;
    pub const ROOK_PROMO_CAPTURE: u16   = 0b1110;
    pub const QUEEN_PROMO_CAPTURE: u16  = 0b1111;
}

#[derive(Clone, Copy, Debug)]
pub struct Move {data: u16, ordering_score: i16}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for Move {}

impl Move {
    // Packs arguments, and calculates heuristics (Only mvv_lva for now.)
    pub fn new(board: &Board, from: u16, to: u16, flags: u16, piece: Piece) -> Self {
        let mut score = 0;

        // Is a capture
        if flags & move_flags::CAPTURE != 0 {
            let enemy_piece;

            // Is an en-passant capture
            if flags & move_flags::EP_CAPTURE != 0 {
                enemy_piece = Piece::Pawn;
            }  else {
                enemy_piece = board[to]
            }

            score += calc_mvv_lva_heuristic(piece, enemy_piece)
        }

        Move {
            data: {
                (from) | (to << 6) | (flags << 12)
            },
            ordering_score: score
        }
    }

    pub fn data(&self) -> u16 {
        self.data
    }

    // Mostly used for initializing non-moves in the movelist, and for transposition tables.
    pub fn new_without_score(data: u16) -> Self {
        Move {
            data,
            ordering_score: 0,
        }
    }

    // Helpers for unpacking data field
    pub fn from_sq(self) -> u16 {
        self.data & 0x3F
    }

    pub fn to_sq(self) -> u16 {
        (self.data >> 6) & 0x3F
    }

    pub fn flags(self) -> u16 {
        (self.data >> 12) & 0x3F
    }

    pub fn is_capture(self) -> bool {
        self.flags() & 0b0100 != 0
    }

    pub fn is_promo(self) -> bool {
        self.flags() & 0b1000 != 0
    }

    pub fn captured_piece(self, board: &Board) -> Piece {
        if self.flags() & move_flags::EP_CAPTURE != 0 {
            Piece::Pawn
        } else {
            board[self.to_sq()]
        }
    }

    pub fn score(self) -> i16 {
        self.ordering_score
    }
}