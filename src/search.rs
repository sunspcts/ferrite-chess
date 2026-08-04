use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use crate::{board::Board, eval::eval, moves::Move};
const MATE_EVAL: i64 = 30000;
const NODE_CHECK_INTERVAL_MASK: u64 = 2047; // Check search control every 2048 nodes

// Holds global search variables shared across the recursion
pub struct SearchEnv {
    pub nodes_visited: u64,
    pub hash_history: Vec<u64>,
    pub search_control: SearchControl
}

impl SearchEnv {
    pub fn is_repetition(&self, key: u64) -> bool {
        self.hash_history.contains(&key)
    }
}

struct SearchContext {
    pub alpha: i64,
    pub beta: i64,
    pub ply: i64,
    pub depth: i64,
} 

#[derive(Clone)]
pub struct SearchControl {
    pub stop: Arc<AtomicBool>,
}

impl SearchControl {
    pub fn new() -> Self {
        SearchControl { stop: Arc::new(AtomicBool::new(false)) }
    }

    pub fn is_stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}


fn negamax(board: &Board, mut context: SearchContext, env: &mut SearchEnv) -> i64 {
    env.nodes_visited += 1;

    if (env.nodes_visited & NODE_CHECK_INTERVAL_MASK == 0) && env.search_control.is_stopped() {
        return 0;
    }

    if context.ply > 0 && env.is_repetition(board.game_state.curr_zobrist_key) {
        return 0;
    }

    if context.depth <= 0 {
        return quiescense(board, context, env);
    }

    let mut moves = board.generate_pseudolegal_moves();
    moves.sort_by(|a, b| b.score().cmp(&a.score()));

    let mut legal_moves_count = 0;
    let mut max_score = i64::MIN;

    for candidate_move in moves {
        if let Some(next_board) = board.make(candidate_move) {
            legal_moves_count += 1;

            let next_context = SearchContext {
                alpha: -context.beta,
                beta: -context.alpha,
                depth: context.depth - 1,
                ply: context.ply + 1,
            };

            env.hash_history.push(board.game_state.curr_zobrist_key);
            let score = -negamax(&next_board, next_context, env);
            env.hash_history.pop();

            if env.search_control.is_stopped() {
                return 0;
            }

            if score > max_score {
                max_score = score;
            }

            if score > context.alpha {
                context.alpha = score;
            }

            if context.alpha >= context.beta {
                break; // Beta cutoff! Remaining moves are pruned without executing board.make()
            }
        }
    }

    if legal_moves_count == 0 {
        if board.is_in_check() {
            return -MATE_EVAL + context.ply;
        } else {
            return 0; // Stalemate
        }
    }

    max_score
}

fn quiescense(board: &Board, mut context: SearchContext, env: &mut SearchEnv) -> i64 {
    env.nodes_visited += 1;

    if (env.nodes_visited & NODE_CHECK_INTERVAL_MASK == 0) && env.search_control.is_stopped() {
        return 0;
    }
    
    let static_eval = eval(board);

    let mut best_value = static_eval;

    if best_value >= context.beta {
        return best_value;
    }
    if best_value > context.alpha {
        context.alpha = best_value;
    }

    let mut moves = board.generate_pseudolegal_moves();
    moves.retain(|mv| mv.is_capture() || mv.is_promo());
    moves.sort_by(|a, b| b.score().cmp(&a.score()));

    for candidate_move in moves {
        if let Some(next_board) = board.make(candidate_move) {
            let next_context = SearchContext {
                alpha: -context.beta,
                beta: -context.alpha,
                depth: 0,
                ply: context.ply + 1,
            };

            let score = -quiescense(&next_board, next_context, env);

            if env.search_control.is_stopped() {
                return 0;
            }

            if score >= context.beta {
                return score;
            }
            if score >= best_value {
                best_value = score;
            }
            if score > context.alpha {
                context.alpha = score;
            }
        }
    }
    best_value
}

pub fn search(board: &Board, depth: i64, env: &mut SearchEnv) -> (i64, Option<Move>) {
    let mut moves = board.generate_pseudolegal_moves();
    moves.sort_by(|a, b| b.score().cmp(&a.score()));

    let mut best_move = None;
    let mut max_score = i64::MIN;
    let mut alpha = -1_000_000;
    let beta = 1_000_000;

    for candidate_move in moves {
        if env.search_control.is_stopped() {
            break;
        }

        if let Some(next_board) = board.make(candidate_move) {
            let context = SearchContext {
                alpha: -beta,
                beta: -alpha,
                depth: depth - 1,
                ply: 1,
            };

            env.hash_history.push(board.game_state.curr_zobrist_key);
            let score = -negamax(&next_board, context, env);
            env.hash_history.pop();

            if env.search_control.is_stopped() {
                break;
            }

            if score > max_score {
                max_score = score;
                best_move = Some(candidate_move);
            }

            if score > alpha {
                alpha = score;
            }
        }
    }

    (max_score, best_move)
}