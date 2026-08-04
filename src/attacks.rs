use crate::bitboard::Bitboard;

pub const KING_ATTACKS: [Bitboard; 64] = gen_leaper_attacks(&DIR_OFFSETS);
pub const KNIGHT_ATTACKS: [Bitboard; 64] = gen_leaper_attacks(&KNIGHT_OFFSETS);
pub const PAWN_ATTACKS: [[Bitboard; 64]; 2] = gen_pawn_attacks();
pub const RAYS: [[Bitboard; 64]; 8] = gen_ray_lookup();

const DIR_OFFSETS: [(i8, i8); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

const KNIGHT_OFFSETS: [(i8, i8); 8] = [
    (2, 1),
    (1, 2),
    (-1, 2),
    (-2, 1),
    (-2, -1),
    (-1, -2),
    (1, -2),
    (2, -1),
];

const A_FILE: u64 = 0x0101010101010101;
const H_FILE: u64 = 0x8080808080808080;

const fn gen_leaper_attacks(offsets: &[(i8, i8)]) -> [Bitboard; 64] {
    let mut attacks = [Bitboard::zero(); 64];
    let mut sq: usize = 0;

    while sq < 64 {
        let mut bb = 0;
        let rank = (sq / 8) as i8;
        let file = (sq % 8) as i8;

        let mut i = 0;
        while i < offsets.len() {
            let (dr, df) = offsets[i];
            let (r, f) = (rank + dr, file + df);

            if r >= 0 && f >= 0 && r < 8 && f < 8 {
                bb |= 1u64 << (r * 8 + f);
            }

            i += 1;
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

        let white_attacks = Bitboard::new(((bb & !A_FILE) << 7) | ((bb & !H_FILE) << 9));
        attacks[0][sq] = white_attacks;

        let black_attacks = Bitboard::new(((bb & !H_FILE) >> 7) | ((bb & !A_FILE) >> 9));
        attacks[1][sq] = black_attacks;

        sq += 1;
    }

    attacks
}

const fn gen_ray_lookup() -> [[Bitboard; 64]; 8] {
    let mut rays = [[Bitboard::zero(); 64]; 8];
    let mut sq = 0;
    while sq < 64 {
        let rank = (sq / 8) as i8;
        let file = (sq % 8) as i8;
        let mut dir = 0;

        while dir < 8 {
            let mut ray_bb = 0;
            let (dr, df) = DIR_OFFSETS[dir];
            let (mut r, mut f) = (rank + dr, file + df);

            while r >= 0 && f >= 0 && r < 8 && f < 8 {
                ray_bb |= 1u64 << (r * 8 + f);
                r += dr;
                f += df;
            }

            rays[dir][sq] = Bitboard::new(ray_bb);
            dir += 1;
        }
        sq += 1;
    }
    rays
}