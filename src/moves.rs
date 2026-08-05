use crate::{bitboard::Bitboard, board::*, heuristics::*, piece::Piece};

// Data field is structured as follows: 
// First 4 bits encode any flags that make_move needs to know.
// Next 6 bits represent the square the piece is moving to.
// Lowest 6 bits represent the square the piece is moving from.

// Heuristics are calculated at movegen, which I might change.

#[derive(Clone, Copy)]
pub struct Move {data: u16, ordering_score: i16}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl Eq for Move {}

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

    pub fn data(&self) -> u16 {
        self.data
    }

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

    pub fn score(self) -> i16 {
        self.ordering_score
    }

    pub fn from_uci(board: &Board, uci_str: &str) -> Option<Move> {
        let moves = board.generate_pseudolegal_moves();
        for mv in moves {
            if mv.to_string() == uci_str {
                if board.make(mv).is_some() {
                    return Some(mv);
                }
            }
        }
        None
    }
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

#[derive(Clone, Copy)]
pub struct MoveList {
    moves: [Move; 256],
    len: usize,
}

impl MoveList {
    pub fn new() -> Self {
        MoveList {
            moves: [Move::new_without_score(0); 256], 
            len: 0,
        }
    }

    pub fn push(&mut self, mv: Move) {
        self.moves[self.len] = mv;
        self.len += 1
    }
}
impl Board {
    pub fn make(&self, mv: Move) -> Option<Board> {
        let mut board = *self;
        let side = board.game_state.active_side;
        let enemy = side.flip();
        let from = mv.from_sq();
        let to = mv.to_sq();
        let piece = board[from];
        let flags = mv.flags();

        if flags & 0b1110 == 0b0010 {
            let transit_sq = match to {
                2 => 3,   
                6 => 5,   
                58 => 59, 
                62 => 61,
                _ => unreachable!(),
            };
            if self.is_attacked(from, enemy) || self.is_attacked(transit_sq, enemy) {
                return None;
            }
        }
        
        board.game_state.inc_halfmoves();
        if let Some(old_ep_square) = board.game_state.en_passant_square {
            board.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[768 + 16 + (old_ep_square % 8) as usize];
            board.game_state.en_passant_square = None;
        }

        if mv.is_capture() {
            let captured_piece = board[to];
            if captured_piece != Piece::None {
                board.remove_piece(enemy, captured_piece, to);
            }
            board.game_state.reset_halfmoves();
            if captured_piece == Piece::Rook {
                board.update_castling_rights(255, to); //255 is a dummy value here!
            }
        }

        if piece != Piece::Pawn {
            board.move_piece(piece, side, from, to)
        } else {
            board.remove_piece(side, piece, from);
            let piece_to_place = if mv.is_promo() {
                promo_flag_parser(flags)
            } else {
                    Piece::Pawn
            };

            board.place_piece(side, piece_to_place, to);
            board.game_state.reset_halfmoves();

            if flags == move_flags::EP_CAPTURE {
                board.remove_piece(enemy, Piece::Pawn, (to as u8 ^ 8) as u16);
            }

            if flags == move_flags::DOUBLE_PAWN_PUSH {
                let ep_square = ((from + to) / 2) as u8;
                board.game_state.en_passant_square = Some(ep_square);
                board.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[768 + 16 + (ep_square % 8) as usize];
            }
        }

        if piece == Piece::King || piece == Piece::Rook {
            board.update_castling_rights(from, to);
        } 

        if flags & 0b1110 == 0b0010 {
            match to {
                2 => board.move_piece(Piece::Rook, side, 0, 3),
                6 => board.move_piece(Piece::Rook, side, 7, 5),
                58 => board.move_piece(Piece::Rook, side, 56, 59),
                62 => board.move_piece(Piece::Rook, side, 63, 61),
                _ => unreachable!()
            }
        }

        if side == Side::Black {
            board.game_state.inc_count();
        }

        board.game_state.active_side = enemy;
        board.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[768 + 16 + 8];
        
        let king_square = board.piece_bb[side as usize][Piece::King as usize].trailing_zeros() as u16;
        let is_legal = !board.is_attacked(king_square, enemy);

        if !is_legal {
            return None
        }

        Some(board)
    }

    fn remove_piece(&mut self, side: Side, piece: Piece, sq: u16) {
        let mask = Bitboard::one() << sq as usize;
        let side_idx = side as usize;
        let piece_idx = piece as usize;

        self.piece_bb[side_idx][piece_idx] ^= mask;
        self.side_bb[side_idx] ^= mask;
        self[sq] = Piece::None;

        let zobrist_idx = get_piece_zobrist_index(piece, side, sq as usize);
        self.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[zobrist_idx];

    }

    fn place_piece(&mut self, side: Side, piece: Piece, sq: u16) {
        let mask = Bitboard::one() << sq as usize;
        let side_idx = side as usize;
        let piece_idx = piece as usize;
        self.piece_bb[side_idx][piece_idx] |= mask;
        self.side_bb[side_idx] |= mask;
        self[sq] = piece;

        let zobrist_idx = get_piece_zobrist_index(piece, side, sq as usize);
        self.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[zobrist_idx];
    }

    fn move_piece(&mut self, piece: Piece, side: Side, from: u16, to: u16) {
        self.remove_piece(side, piece, from);
        self.place_piece(side, piece, to);
    }

    #[cfg(test)]
    pub fn perft(&self, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut nodes = 0;
        let moves = self.generate_pseudolegal_moves();

        for m in moves {
            if let Some(next_board) = self.make(m) {
                nodes += next_board.perft(depth - 1);
            }
        }

        nodes
    }

}

fn promo_flag_parser(flag: u16) -> Piece {
    match flag & 0b0011 {
        0 => Piece::Knight,
        1 => Piece::Bishop,
        2 => Piece::Rook,
        3 => Piece::Queen,
        _ => unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft_startpos() {
        let board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(board.perft(1), 20);
        assert_eq!(board.perft(2), 400);
        assert_eq!(board.perft(3), 8902);
        assert_eq!(board.perft(4), 197281);
        assert_eq!(board.perft(5), 4865609);
    }
}