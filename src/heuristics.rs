use crate::piece::Piece;

const MVV_LVA_TAB: [[i16; 6]; 6] = init_mvv_lva_table();

const fn init_mvv_lva_table() -> [[i16; 6]; 6] {
    let mut tab = [[0; 6]; 6];
    let mut a = 0; let mut v = 0;
    while a < 6 {
        while v < 6 {
            tab[v][a] = (((v + 1) * 10) - (6 - a)) as i16;
            v += 1;
        }
        a += 1;
    }
    tab
}
#[inline(always)]
pub fn calc_mvv_lva_heuristic(piece: Piece, enemy_piece: Piece) -> i16 {
    MVV_LVA_TAB[enemy_piece as usize][piece as usize]
}