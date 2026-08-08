use crate::moves::Move;

pub const MAX_HISTORY: i32 = 8000;

#[inline]
pub fn update_history_cutoff(
    history: &mut [[[i32; 64]; 64]; 2],
    side: usize,
    cutoff_move: Move,
    quiet_moves_tried: &[Move],
    depth: i64,
) {
    let delta = (depth * depth) as i32;

    // Bonus for cutoff move
    let from = cutoff_move.from_sq() as usize;
    let to = cutoff_move.to_sq() as usize;
    let current_val = history[side][from][to];
    let bonus = delta - (current_val * delta) / MAX_HISTORY;
    history[side][from][to] += bonus;

    // Malus for quiet moves that failed to cause a cutoff
    for &q_move in quiet_moves_tried {
        let q_from = q_move.from_sq() as usize;
        let q_to = q_move.to_sq() as usize;
        let q_val = history[side][q_from][q_to];
        let malus = -delta - (q_val * delta) / MAX_HISTORY;
        history[side][q_from][q_to] += malus;
    }
}
