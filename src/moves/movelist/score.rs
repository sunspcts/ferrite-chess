use crate::{board::Board, heuristics::calc_mvv_lva_heuristic, moves::move_flags};

use super::*;

impl MoveList {
    pub fn score_moves(&mut self, board: &Board, tt_move: Option<Move>, killers: &[u16]) {
        let len = self.len as usize;
        for i in 0..len {
            self.scores[i] = score_move(self.moves[i], tt_move, killers, board);
        }
    }

    pub fn score_qsearch_moves(&mut self, board: &Board) {
        let len = self.len as usize;
        for i in 0..len {
            let mv = self.moves[i];
            self.scores[i] = calc_mvv_lva_heuristic(board[mv.from_sq()], mv.captured_piece(board));
        }
    }

    pub fn sort_moves(&mut self) {
        let len = self.len as usize;
        if len <= 1 {
            return;
        }

        let mut scored: [(i16, Move); 256] = [(0, Move::new_from_raw(0)); 256];
        for i in 0..len {
            scored[i] = (self.scores[i], self.moves[i]);
        }

        scored[..len].sort_unstable_by(|a, b| b.0.cmp(&a.0));

        for i in 0..len {
            self.scores[i] = scored[i].0;
            self.moves[i] = scored[i].1;
        }
    }
}

#[inline]
fn score_move(mv: Move, tt_move: Option<Move>, killers: &[u16], board: &Board) -> i16 {
    if Some(mv) == tt_move {
        return i16::MAX;
    }
    if mv.flags() & move_flags::QUEEN_PROMO == move_flags::QUEEN_PROMO {
        return 20000;
    }
    if mv.is_capture() {
        let piece = board[mv.from_sq()];
        let captured = mv.captured_piece(board);

        let mvv_lva = calc_mvv_lva_heuristic(piece, captured);
        return 10000 + mvv_lva;
    }
    if mv.data() == killers[0] {
        return 9000;
    }
    if mv.data() == killers[1] {
        return 8999;
    }
    0
}