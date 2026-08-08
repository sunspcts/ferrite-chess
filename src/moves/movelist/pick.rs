use super::*;

impl MoveList {
    pub fn pick_best(&mut self, idx: usize) -> Move {
        if idx >= self.len() {
            return Move::new_from_raw(0);
        }

        let mut best_idx = idx;
        let mut best_score = self.scores[idx];
        
        let mut curr_idx = idx;
        
        while curr_idx < self.len() {
            if self.scores[curr_idx] > best_score {
                best_score = self.scores[curr_idx];
                best_idx = curr_idx;
            }

            curr_idx += 1
        }

        self.swap(idx, best_idx);
        return self[idx];
    }
}