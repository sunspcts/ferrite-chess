use crate::{board::Board, eval::eval, moves::Move};
const MATE_EVAL: i64 = 30000;

// Holds global search variables shared across the recursion
pub struct SearchEnv {
    pub nodes_visited: u64,
    pub hash_history: Vec<u64>
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

fn negamax(board: &Board, mut context: SearchContext, env: &mut SearchEnv) -> i64 {
    env.nodes_visited += 1;

    if context.ply > 0 && env.is_repetition(board.game_state.curr_zobrist_key) {
        return 0;
    }

    if context.depth == 0 {
        return eval(board);
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

pub fn search(board: &Board, depth: i64, env: &mut SearchEnv) -> (i64, Option<Move>) {
    let mut moves = board.generate_pseudolegal_moves();
    moves.sort_by(|a, b| b.score().cmp(&a.score()));

    let mut best_move = None;
    let mut max_score = i64::MIN;
    let mut alpha = -1_000_000;
    let beta = 1_000_000;

    for candidate_move in moves {
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