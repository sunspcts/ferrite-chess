use std::{fmt, ops::{Add, AddAssign}};
use rand::{Rng, rngs::StdRng, SeedableRng};
const MAX_GAME_MOVES: usize = 1024; //sure. 
const PIECE_CHARS: &str = "kqrbnpKQRBNP";

#[derive(Clone, Copy)]
pub struct Bitboard(pub u64);

impl Add for Bitboard {
    type Output = Bitboard;

    fn add(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 + rhs.0)
    }
}

impl AddAssign for Bitboard {
    fn add_assign(&mut self, rhs: Bitboard) {
        *self = *self + rhs
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut display: Vec<char> = vec![];

        for rank in 0..8 {
            for file in (0..8).rev() {
                let mask = 1u64 << (63 - (rank * 8) - file);
                let chr = if &self.0 & mask != 0 {'1'} else {'0'};
                display.push(chr); display.push(' ');
            }
            display.push('\n')
        }
        write!(f, "{}", display.into_iter().collect::<String>())
    }
} 

impl Default for Bitboard {
    fn default() -> Self {
        Bitboard(0)
    }
}

impl Bitboard {
    pub fn serialize(&self) -> Vec<u8> {
        let mut bb = self.0;
        std::iter::from_fn(move || {
            if bb == 0 {None} else {
                let idx = bb.trailing_zeros();
                bb -= 1 << idx; Some(idx as u8)
            }
        }).collect()
    }
}

// This is a placeholder! This will have associated logic eventually.
#[derive(Clone, Copy)]
pub struct Move;

#[derive(Clone, Copy, Debug)]
pub enum Piece {
    Pawn, // 0
    Knight, // 1
    Bishop, // ..
    Rook,
    Queen,
    King,
    None // 6
}

#[derive(Clone, Copy)]
pub enum Side {
    White,
    Black
}

pub struct Board {
    pub piece_bitboards: [[Bitboard; 6]; 2],
    pub side_bitboards: [Bitboard; 2],
    pub game_state: GameState,
    // pub history: StateHistory, // I need to implement some defaults to fill the history array!
    pub zobrist_randoms: [u64; 781],
}

pub struct BoardBuilder {
    pub piece_bitboards: [[Bitboard; 6]; 2],
    pub side_bitboards: [Bitboard; 2],
    pub game_state: GameState,
}

impl BoardBuilder {
    pub fn init(&self) -> Board {
        let zobrist_randoms = gen_zobrist_random_array();
        let key = self.initialize_zobrist_key(&zobrist_randoms);
        let mut game_state = self.game_state;

        game_state.curr_zobrist_key = key;

        Board {
            piece_bitboards: self.piece_bitboards,
            side_bitboards: self.side_bitboards,
            game_state: game_state,
            zobrist_randoms,
        }
    }

    fn initialize_zobrist_key(&self, zobrist_randoms: &[u64; 781]) -> u64 {
        let mut hash = 0;
        let bitboards = [self.piece_bitboards[0], self.piece_bitboards[1]];
        for (color, boards) in bitboards.into_iter().enumerate() {
            for (piece, board) in boards.into_iter().enumerate() {
                let mut board_mut = board.0;
                let offset = (color * 6) + piece;

                while board_mut != 0 {
                    let sq_idx = board_mut.trailing_zeros() as usize;
                    let rand_idx = sq_idx * 12 + offset;
                    hash ^= zobrist_randoms[rand_idx];
                    board_mut ^= 1 << sq_idx;
                }
            }
        }
        hash
    }
}

#[derive (Clone, Copy)]
pub struct GameState {
    pub active_side: Side,
    pub castling: u8,
    pub half_moves: u8,
    pub move_counter: u8,
    pub en_passant_square: Option<u8>, //unfortunately, it's unprofessional to call this the holy_hell_square.
    pub curr_zobrist_key: u64,
    pub next_move: Move
}

pub struct StateHistory {
    history: [GameState; MAX_GAME_MOVES],
    count: usize,
}

fn gen_zobrist_random_array() -> [u64; 781] {
    let mut rng = StdRng::seed_from_u64(0b0101000101000010100101001111000011101101000111011011001111111100);
    let mut values: Vec<u64> = (0..781).map(|_| rng.random::<u64>()).collect();
    values.try_into().expect("incorrect length")
}

//obviously this can be done far, far, better. don't care rn!
fn index_to_piece (idx: usize) -> Piece {
    match idx {
        0 => Piece::Pawn,
        1 => Piece::Knight,
        2 => Piece::Bishop,
        3 => Piece::Rook,
        4 => Piece::Queen,
        5 => Piece::King,
        _ => Piece::None
    }
}

pub fn init_boardbuilder_from_fen(fen: &str) -> BoardBuilder { 
    // if you give it a malformed FEN it will fucking die, so uhhh don't.
    let mut rank = 7;
    let mut file = 0;
    let mut white_bitboards = [Bitboard::default(); 6];
    let mut black_bitboards = [Bitboard::default(); 6];
    
    let mut split = fen.split_ascii_whitespace();
    let pieces = split.next().unwrap();
    for char in pieces.chars() {
        let sq = (rank * 8) + file;

        match char {
            'p' => black_bitboards[0].0 += 1 << sq,
            'P' => white_bitboards[0].0 += 1 << sq,
            'n' => black_bitboards[1].0 += 1 << sq,
            'N' => white_bitboards[1].0 += 1 << sq,
            'b' => black_bitboards[2].0 += 1 << sq,
            'B' => white_bitboards[2].0 += 1 << sq,
            'r' => black_bitboards[3].0 += 1 << sq,
            'R' => white_bitboards[3].0 += 1 << sq,
            'q' => black_bitboards[4].0 += 1 << sq,
            'Q' => white_bitboards[4].0 += 1 << sq,
            'k' => black_bitboards[5].0 += 1 << sq,
            'K' => white_bitboards[5].0 += 1 << sq,
            '1'..='8' => {
                if let Some(x) = char.to_digit(10) {
                    file += x as u8;
                }
            }
            '/' => { rank -= 1; file = 0 }
            _ => panic!("unsupported character {} in FEN string!", char) // fix this, please dont just fucking panic
        }

        if PIECE_CHARS.contains(char) {
            file += 1
        }
    }

    let color_to_play = match split.next().unwrap() {
        "w" => Side::White,
        "b" => Side::Black,
        _ => Side::White
    };

    let castling_rights_str = split.next().unwrap();
    let mut castling_rights_u8 = 0;

    for c in castling_rights_str.chars() {
        castling_rights_u8 += match c {
            'K' => 1,
            'Q' => 2,
            'k' => 4,
            'q' => 8,
            _ => 0,
        }
    }

    let ep_sqr_str = split.next().unwrap();
    //TODO: Fix once I have an algebraic notation parser.

    let halfmove_clock: u8 = split.next().unwrap().parse().unwrap();
    let fullmove_clock: u8 = split.next().unwrap().parse().unwrap();

    let white_bitboard: Bitboard = white_bitboards.into_iter().fold(Bitboard(0), |acc, x| acc + x);
    let black_bitboard: Bitboard = black_bitboards.into_iter().fold(Bitboard(0), |acc, x| acc + x);

    let board_builder: BoardBuilder = BoardBuilder {
        piece_bitboards: [white_bitboards, black_bitboards],
        side_bitboards: [white_bitboard, black_bitboard],
        game_state: GameState {
            active_side: color_to_play,
            castling: castling_rights_u8,
            half_moves: halfmove_clock,
            move_counter: fullmove_clock,
            en_passant_square: None,
            curr_zobrist_key: 0,
            next_move: Move
        }
    };

    board_builder
}