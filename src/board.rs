// IMPORTS

// CONSTANT VALUES

use crate::{attacks::{KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS}, bitboard::Bitboard, piece::Piece};

pub const ZOBRIST_RANDOMS: [u64; 793] = init_zobrist_random_array();

const MAX_GAME_MOVES: usize = 1024; // pretty arbitrary. 
const PIECE_CHARS: &str = "kqrbnpKQRBNP";

//evaluated at compile time for quick accesses. we will be using this a lot!
const fn init_zobrist_random_array() -> [u64; 793] {
    let mut arr = [0; 793];
    let mut i = 0;
    let mut state = 0x13371337;

    while i < 100 {
        state = xorshift(state); // quick mix
        i += 1
    }
    i = 0;
    while i < 793 {
        let new_state = xorshift(state);
        arr[i] = new_state;
        state = new_state;
        i += 1
    }

    arr
}

const fn xorshift(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 5;
    state
}

// STATE STRUCT

#[derive (Clone, Copy)]
pub struct GameState {
    pub active_side: Side,
    pub castling: u8,
    half_moves: u8,
    move_counter: u16,
    pub en_passant_square: Option<u8>, //unfortunately, it's unprofessional to call this the holy_hell_square.
    pub curr_zobrist_key: u64,
}

impl GameState {
    pub fn inc_halfmoves(&mut self) {
        self.half_moves += 1
    }

    pub fn reset_halfmoves(&mut self) {
        self.half_moves = 0
    }

    pub fn inc_count(&mut self) {
        self.move_counter += 1
    }
}

// SIDE ENUM

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    White = 0,
    Black = 1
}

impl Side {
    pub fn flip(self) -> Self {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}


// BOARD STRUCT

#[derive(Copy, Clone)]
pub struct Board {
    pub piece_bb: [[Bitboard; 6]; 2],
    pub side_bb: [Bitboard; 2],
    pub game_state: GameState,
    mailbox: [Piece; 64]
}

impl std::ops::Index<u16> for Board {
    type Output = Piece;

    fn index(&self, sq: u16) -> &Self::Output {
        &self.mailbox[sq as usize] 
    }
}

impl std::ops::IndexMut<u16> for Board {
    fn index_mut(&mut self, sq: u16) -> &mut Self::Output {
        &mut self.mailbox[sq as usize]
    }
}

impl Board {
    pub fn new_from_fen(fen: &str) -> Self {
        let fen_parts: Vec<&str> = fen.split_ascii_whitespace().collect();
        let (piece_bb, side_bb, mailbox) = init_bb_mb_fen(fen_parts[0]);

        let game_state = GameState { 
            active_side: init_active_side(fen_parts[1]), 
            castling: init_castling_rights(fen_parts[2]), 
            en_passant_square: init_ep_square(fen_parts[3]),
            half_moves: init_halfmoves(fen_parts[4]),
            move_counter: init_move_counter(fen_parts[5]),
            curr_zobrist_key: 0 
        };

        let mut board = Board {
            piece_bb,
            side_bb,
            game_state,
            mailbox
        };

        board.recompute_zobrist_hash();

        board
    }

    fn get_side_at(&self, square: u16) -> Option<Side> {
        let mask = Bitboard::new(1 << square);
        if (self.side_bb[0] & mask) != Bitboard::zero() {
            return Some(Side::White) 
        } else if (self.side_bb[1] & mask) != Bitboard::zero() {
            return Some(Side::Black) 
        }
        None
    }

    pub fn is_attacked(&self, square: u16, attacker_side: Side) -> bool {
        let attacker = attacker_side as usize;
        let defender = (attacker_side as usize) ^ 1;

        let enemy_knights = self.piece_bb[attacker][Piece::Knight as usize];
        if (KNIGHT_ATTACKS[square as usize] & enemy_knights) != Bitboard::zero() {
            return true;
        }

        let enemy_king = self.piece_bb[attacker][Piece::King as usize];
        if (KING_ATTACKS[square as usize] & enemy_king) != Bitboard::zero() {
            return true;
        }

        let enemy_pawns = self.piece_bb[attacker][Piece::Pawn as usize];
        if (PAWN_ATTACKS[defender][square as usize] & enemy_pawns) != Bitboard::zero() {
            return true;
        }

        let diagonal_attackers = self.piece_bb[attacker][Piece::Bishop as usize] | self.piece_bb[attacker][Piece::Queen as usize];
        if (self.get_bishop_attacks(square, defender) & diagonal_attackers) != Bitboard::zero() {
            return true;
        }

        let orthogonal_attackers = self.piece_bb[attacker][Piece::Rook as usize] | self.piece_bb[attacker][Piece::Queen as usize];
        if (self.get_rook_attacks(square, defender) & orthogonal_attackers) != Bitboard::zero() {
            return true;
        }
        false
    }

    fn recompute_zobrist_hash(&mut self) {
        let mut key = 0;
        for sq in 0..64 {
            let piece = self[sq as u16];
            if piece != Piece::None {
                if let Some(side) = self.get_side_at(sq as u16) {
                    key ^= ZOBRIST_RANDOMS[get_piece_zobrist_index(piece, side, sq)];
                }
            }
        }

        key ^= ZOBRIST_RANDOMS[768 + self.game_state.castling as usize];

        if let Some(x) = self.game_state.en_passant_square {
            let file = x % 8;
            key ^= ZOBRIST_RANDOMS[768 + 16 + file as usize];
        }

        if self.game_state.active_side == Side::Black {
            key ^= ZOBRIST_RANDOMS[768 + 16 + 8];
        }

        self.game_state.curr_zobrist_key = key
    }

    
    pub fn update_castling_rights(&mut self, from_sq: u16, to_sq: u16) {
        let old_castling = self.game_state.castling;
        match from_sq {
            4 => self.game_state.castling &= !3,
            60 => self.game_state.castling &= !12,
            7 => self.game_state.castling &= !1,
            0 => self.game_state.castling &= !2,
            63 => self.game_state.castling &= !4,
            56 => self.game_state.castling &= !8,
            _ => ()
        }
        match to_sq {
            7 => self.game_state.castling &= !1,
            0 => self.game_state.castling &= !2,
            63 => self.game_state.castling &= !4,
            56 => self.game_state.castling &= !8,
            _ => ()
        }
        if self.game_state.castling != old_castling {
            self.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[768 + old_castling as usize];
            self.game_state.curr_zobrist_key ^= ZOBRIST_RANDOMS[768 + self.game_state.castling as usize];
        }
    }

    pub fn is_in_check(&self) -> bool {
        let side = self.game_state.active_side;
        let king_bb = self.piece_bb[side as usize][Piece::King as usize];
        let king_square = king_bb.trailing_zeros() as u16;

        self.is_attacked(king_square, side.flip())
    }

    pub fn side_to_move_multiplier(&self) -> i64 {
        match self.game_state.active_side {
            Side::White => 1,
            Side::Black => -1
        }
    }
}

// FEN PARSING

fn init_bb_mb_fen(fen_part_1: &str) -> ([[Bitboard; 6]; 2], [Bitboard; 2], [Piece; 64]) {
    let mut piece_bb = [[Bitboard::default(); 6]; 2];
    let mut side_bb = [Bitboard::default(); 2];
    let mut mailbox: [Piece; 64] = [Piece::None; 64];

    let mut rank = 7; let mut file = 0;
    for char in fen_part_1.chars() {
        let sq = (rank * 8) + file;

        match char {
            'p' => {piece_bb[1][0] |= Bitboard::one() << sq; mailbox[sq] = Piece::Pawn},
            'P' => {piece_bb[0][0] |= Bitboard::one() << sq; mailbox[sq] = Piece::Pawn},
            'n' => {piece_bb[1][1] |= Bitboard::one() << sq; mailbox[sq] = Piece::Knight},
            'N' => {piece_bb[0][1] |= Bitboard::one() << sq; mailbox[sq] = Piece::Knight},
            'b' => {piece_bb[1][2] |= Bitboard::one() << sq; mailbox[sq] = Piece::Bishop},
            'B' => {piece_bb[0][2] |= Bitboard::one() << sq; mailbox[sq] = Piece::Bishop},
            'r' => {piece_bb[1][3] |= Bitboard::one() << sq; mailbox[sq] = Piece::Rook},
            'R' => {piece_bb[0][3] |= Bitboard::one() << sq; mailbox[sq] = Piece::Rook},
            'q' => {piece_bb[1][4] |= Bitboard::one() << sq; mailbox[sq] = Piece::Queen},
            'Q' => {piece_bb[0][4] |= Bitboard::one() << sq; mailbox[sq] = Piece::Queen},
            'k' => {piece_bb[1][5] |= Bitboard::one() << sq; mailbox[sq] = Piece::King},
            'K' => {piece_bb[0][5] |= Bitboard::one() << sq; mailbox[sq] = Piece::King},
            '1'..='8' => {
                if let Some(x) = char.to_digit(10) {
                    file += x as usize;
                }
            }
            '/' => { rank -= 1; file = 0 }
            _ => panic!("unsupported character {} in FEN string!", char) // fix this, please dont just fucking panic
        }

        if PIECE_CHARS.contains(char) {
            file += 1
        }
    }

    //initializing side bitboards
    for side in 0..=1 {
        let piece_bbs = piece_bb[side];
        for bb in piece_bbs {
            side_bb[side] |= bb
        }
    }

    (piece_bb, side_bb, mailbox)
}

fn init_active_side(fen_part_2: &str) -> Side {
    match fen_part_2 {
        "w" => Side::White,
        "b" => Side::Black,
        _ => panic!("unsupported field {} in side_to_play component of FEN string!", fen_part_2)
    }
}

fn init_castling_rights(fen_part_3: &str) -> u8 {
    let mut castling_rights = 0;
    for c in fen_part_3.chars() {
        castling_rights += match c {
            'K' => 1,
            'Q' => 2,
            'k' => 4,
            'q' => 8,
            _ => 0,
        }
    }
    castling_rights
}

fn init_ep_square(fen_part_4: &str) -> Option<u8> {
    if fen_part_4 == "-" {
        None 
    } else {
        let mut chars = fen_part_4.chars();
        let file_char = chars.next().unwrap();
        let file = file_char as u8 - b'a';
        let rank_char = chars.next().unwrap();
        let rank = rank_char as u8 - b'1';
        Some(rank * 8 + file)
    }
}

fn init_halfmoves(fen_part_5: &str) -> u8 {
    fen_part_5.parse::<u8>().unwrap()
}

fn init_move_counter(fen_part_6: &str) -> u16 {
    fen_part_6.parse::<u16>().unwrap()
}

// ZOBRIST HASHING

pub fn get_piece_zobrist_index(piece: Piece, side: Side, sq: usize) -> usize {
    ((piece as usize + side as usize * 6)) * 64 + sq
}

