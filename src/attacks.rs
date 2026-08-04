use crate::bitboard::Bitboard;

// N,S,E,W = 8,-8,1,-1

pub const KING_ATTACKS: [Bitboard; 64] = gen_leaper_attacks(&KING_VECTORS);
pub const KNIGHT_ATTACKS: [Bitboard; 64] = gen_leaper_attacks(&KNIGHT_VECTORS);
pub const PAWN_ATTACKS: [[Bitboard; 64]; 2] = gen_pawn_attacks();

const KING_VECTORS: [i8; 8] = [8, 9, 1, -7, -8, -9, -1, 7];
const KNIGHT_VECTORS: [i8; 8] = [17, 10, -6, -15, -17, -10, 6, 15];

const fn gen_leaper_attacks(vectors: &[i8]) -> [Bitboard; 64] {
    let mut attacks = [Bitboard::zero(); 64];
    let mut sq: usize = 0;

     while sq < 64 {
        let mut bb = 0;
        let rank = (sq / 8) as i8;
        let file = (sq % 8) as i8;

        let mut i = 0;

        while i < 8 {
            let (r, f) = (rank + vectors[i] / 8, file + vectors[i] % 8);

            if r >= 0 && f >= 0 && r < 8 && f < 8 {
                bb |= 1u64 << (r * 8 + f);
            }

            i += 1
        }

        attacks[sq] = Bitboard::new(bb);
        sq += 1;
    }

    attacks
}

const fn gen_pawn_attacks() -> [[Bitboard; 64]; 2] { 
    let mut attacks = [[Bitboard::zero(); 64]; 2];
    let mut sq = 0;
    while sq < 64 {
        let bb = 1u64 << sq;

        let white_attacks = Bitboard::new(((bb & !0x0101010101010101) << 7) | ((bb & !0x8080808080808080) << 9));
        attacks[0][sq] = white_attacks;
        
        let black_attacks = Bitboard::new(((bb & !0x8080808080808080) >> 7) | ((bb & !0x0101010101010101) >> 9));
        attacks[1][sq] = black_attacks;
        
        sq += 1;
    }
    
    attacks
}