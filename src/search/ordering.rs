use crate::moves::{Move, MoveList};

#[inline]
pub fn sort_moves(moves: &mut MoveList, tt_move: Option<Move>, killers: &[u16]) {
    moves.sort_by(|a, b| {
        let score_a = score_move(*a, tt_move, killers);
        let score_b = score_move(*b, tt_move, killers);
        score_b.cmp(&score_a)
    });
}

#[inline]
fn score_move(mv: Move, tt_move: Option<Move>, killers: &[u16]) -> i16 {
    if Some(mv) == tt_move {
        return i16::MAX;
    }
    if mv.is_capture() {
        return 10000 + mv.score()
    }
    if mv.data() == killers[0] {
        return 9000
    }
    if mv.data() == killers[1] {
        return 8999
    }
    mv.score()
}

#[inline]
pub fn sort_qsearch_moves(moves: &mut MoveList) {
    moves.sort_by(|a, b| b.score().cmp(&a.score()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tt_move_sorted_first() {
        let mut list = MoveList::default();
        let m1 = Move::new_without_score(10);
        let m2 = Move::new_without_score(20);
        list.push(m1);
        list.push(m2);

        sort_moves(&mut list, Some(m2), &[0,0]);
        assert_eq!(list[0], m2);
    }
}
