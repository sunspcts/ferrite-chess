use crate::{board::Board, heuristics::*, piece::Piece};

// Data field is structured as follows: 
// First 4 bits encode any flags that make_move needs to know.
// Next 6 bits represent the square the piece is moving to.
// Lowest 6 bits represent the square the piece is moving from.

// Heuristics are calculated at movegen, which I might change.

#[derive(Clone, Copy)]
pub struct Move {data: u16, ordering_score: i32}

impl Move {
    pub fn new(board: &Board, from: u16, to: u16, flags: u16, piece: Piece) -> Self {
        let mut score = 0;

        if flags & 0b0100 != 0 {
            let enemy_piece;
            if flags & 0b0001 != 0 {
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

    pub fn score(self) -> i32 {
        self.ordering_score
    }
}

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

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from_file = (b'a' + (self.from_sq() % 8) as u8) as char;
        let from_rank = (b'1' + (self.from_sq() / 8) as u8) as char;
        let to_file = (b'a' + (self.to_sq() % 8) as u8) as char;
        let to_rank = (b'1' + (self.to_sq() / 8) as u8) as char;

        if self.is_promo() {
            let promo_char = match self.flags() & 0b0011 {
                0 => 'n',
                1 => 'b',
                2 => 'r',
                3 => 'q',
                _ => unreachable!(),
            };
            write!(f, "{from_file}{from_rank}{to_file}{to_rank}{promo_char}")?;
        } else {
            write!(f, "{from_file}{from_rank}{to_file}{to_rank}")?;
        }

        Ok(())
    }
}

impl Board {
    pub fn make_move(&mut self, mv: Move) {

    }
}