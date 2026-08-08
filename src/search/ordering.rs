use crate::{board::Board, heuristics::calc_mvv_lva_heuristic, moves::{Move, MoveList}};

#[inline]
pub fn sort_moves(moves: &mut MoveList, tt_move: Option<Move>, killers: &[u16], board: &Board) {
    let len = moves.len();
    if len <= 1 {
        return;
    }

    let mut scored: [(i16, Move); 256] = [(0, Move::new_from_raw(0)); 256];
    for i in 0..len {
        let mv = moves[i];
        scored[i] = (score_move(mv, tt_move, killers, board), mv);
    }

    scored[..len].sort_unstable_by(|a, b| b.0.cmp(&a.0));

    for i in 0..len {
        moves[i] = scored[i].1;
    }
}

#[inline]
fn score_move(mv: Move, tt_move: Option<Move>, killers: &[u16], board: &Board) -> i16 {
    if Some(mv) == tt_move {
        return i16::MAX;
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

#[inline]
pub fn sort_qsearch_moves(moves: &mut MoveList, board: &Board) {
    let len = moves.len();
    if len <= 1 {
        return;
    }

    let mut scored: [(i16, Move); 256] = [(0, Move::new_from_raw(0)); 256];
    for i in 0..len {
        let mv = moves[i];
        let score = calc_mvv_lva_heuristic(board[mv.from_sq()], mv.captured_piece(board));
        scored[i] = (score, mv);
    }

    scored[..len].sort_unstable_by(|a, b| b.0.cmp(&a.0));

    for i in 0..len {
        moves[i] = scored[i].1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tt_move_sorted_first() {
        let mut list = MoveList::default();
        let m1 = Move::new_from_raw(10);
        let m2 = Move::new_from_raw(20);
        list.push(m1);
        list.push(m2);

        let board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        sort_moves(&mut list, Some(m2), &[0,0], &board);
        assert_eq!(list[0], m2);
    }
}
