use std::fmt;

pub struct Bitboard(u64);

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:b}", self.0)
    }
} 
pub struct ChessBoard {
    white_pawns: Bitboard,
    white_rooks: Bitboard,
    white_knights: Bitboard,
    white_bishops: Bitboard,
    white_queens: Bitboard,
    white_king: Bitboard,

    black_pawns: Bitboard,
    black_rooks: Bitboard,
    black_knights: Bitboard,
    black_bishops: Bitboard,
    black_queens: Bitboard,
    black_king: Bitboard,
}

impl Default for Bitboard {
    fn default() -> Self {
        Bitboard(0)
    }
}
impl Default for ChessBoard {
    fn default() -> Self {
        //yes im hardcoding the start position. Cope. I'll add a PGN importer eventually, and store the default initial position as a PGN.
        ChessBoard {
            white_pawns: Bitboard(0x000000000000FF00),
            //the position of every piece of each type is represented by a binary bit. in this case, the last 8 bits are set.
            white_rooks: Bitboard(0x0000000000000081),
            white_knights: Bitboard(0x0000000000000042),
            white_bishops: Bitboard(0x0000000000000024),
            white_queens: Bitboard(0x0000000000000008),
            white_king: Bitboard(0x0000000000000010),
            
            black_pawns: Bitboard(0x00FF000000000000),
            black_rooks: Bitboard(0x8100000000000000),
            black_knights: Bitboard(0x4200000000000000),
            black_bishops: Bitboard(0x2400000000000000),
            black_queens: Bitboard(0x0800000000000000),
            black_king: Bitboard(0x1000000000000000)
        }
    }
}