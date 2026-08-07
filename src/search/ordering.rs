use crate::moves::{Move, MoveList};

#[inline]
pub fn sort_moves(moves: &mut MoveList, tt_move: Option<Move>) {
    moves.sort_by(|a, b| {
        let score_a = if Some(*a) == tt_move { i16::MAX } else { a.score() };
        let score_b = if Some(*b) == tt_move { i16::MAX } else { b.score() };
        score_b.cmp(&score_a)
    });
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

        sort_moves(&mut list, Some(m2));
        assert_eq!(list[0], m2);
    }
}
