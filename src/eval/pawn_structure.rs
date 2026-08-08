use crate::bitboard::Bitboard;

const FILE_A: u64 = 0x0101010101010101;
const FILE_B: u64 = 0x0202020202020202;
const FILE_C: u64 = 0x0404040404040404;
const FILE_D: u64 = 0x0808080808080808;
const FILE_E: u64 = 0x1010101010101010;
const FILE_F: u64 = 0x2020202020202020;
const FILE_G: u64 = 0x4040404040404040;
const FILE_H: u64 = 0x8080808080808080;

const ADJACENT_FILE_MASKS: [Bitboard; 8] = [
    Bitboard::new(FILE_B),
    Bitboard::new(FILE_A | FILE_C),
    Bitboard::new(FILE_B | FILE_D),
    Bitboard::new(FILE_C | FILE_E),
    Bitboard::new(FILE_D | FILE_F),
    Bitboard::new(FILE_E | FILE_G),
    Bitboard::new(FILE_F | FILE_H),
    Bitboard::new(FILE_G)
];

#[inline]
pub fn is_isolated(sq: u16, friendly_pawns: Bitboard) -> bool {
    let file = (sq & 0b0111) as usize;
    (friendly_pawns & ADJACENT_FILE_MASKS[file]) == Bitboard::zero()
}