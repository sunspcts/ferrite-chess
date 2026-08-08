use crate::{board::Board, heuristics::calc_mvv_lva_heuristic, moves::{Move, MoveList}};

#[inline]
pub fn sort_moves(moves: &mut MoveList, tt_move: Option<Move>, killers: &[u16], board: &Board) {
    moves.sort_by(|a, b| {
        let score_a = score_move(*a, tt_move, killers, board);
        let score_b = score_move(*b, tt_move, killers, board);
        score_b.cmp(&score_a)
    });
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
        return 10000 + mvv_lva
    }
    if mv.data() == killers[0] {
        return 9000
    }
    if mv.data() == killers[1] {
        return 8999
    }
    0
}

#[inline]
pub fn sort_qsearch_moves(moves: &mut MoveList, board: &Board) {
    moves.sort_by(|a, b| {
        let score_a = calc_mvv_lva_heuristic(board[a.from_sq()], a.captured_piece(board));
        let score_b = calc_mvv_lva_heuristic(board[b.from_sq()], b.captured_piece(board));
        score_b.cmp(&score_a)
    });
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
