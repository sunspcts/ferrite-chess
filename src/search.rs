use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use crate::{board::Board, eval::eval, moves::Move};
const MATE_EVAL: i64 = 30000;
const NODE_CHECK_INTERVAL_MASK: u64 = 2047; // Check search control every 2048 nodes

// Holds global search variables shared across the recursion
pub struct SearchEnv {
    pub nodes_visited: u64,
    pub node_limit: u64,
    pub hash_history: Vec<u64>,
    pub search_control: SearchControl,
    pub stopped: bool,
    pub tt: TT,
}

impl SearchEnv {
    pub fn is_repetition(&self, key: u64) -> bool {
        self.hash_history.contains(&key)
    }

    pub fn check_stop(&mut self) -> bool {
        if self.stopped || self.search_control.is_stopped() {
            self.stopped = true;
            return true;
        }
        false
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

    if env.stopped || env.nodes_visited >= env.node_limit {
        env.stopped = true;
        return 0;
    }

    if (env.nodes_visited & NODE_CHECK_INTERVAL_MASK == 0) && env.check_stop() {
        return 0;
    }

    if context.ply > 0 && env.is_repetition(board.game_state.curr_zobrist_key) {
        return 0;
    }

    if context.depth <= 0 {
        return quiescense(board, context, env);
    }


    let tt_entry = env.tt.get(board.game_state.curr_zobrist_key);
    let tt_move = tt_entry.and_then(|e| e.best_move);

    if context.ply > 0 {
        if let Some(entry) = tt_entry {
            if entry.depth >= context.depth {
                match entry.node_type {
                    NodeType::Exact => return entry.score,
                    NodeType::LowerBound if entry.score >= context.beta => return entry.score,
                    NodeType::UpperBound if entry.score <= context.alpha => return entry.score,
                    _ => {}
                }
            }
        }
    }

    let mut moves = board.generate_pseudolegal_moves();
    moves.sort_by(|a, b| {
        let score_a = if Some(*a) == tt_move { i32::MAX } else { a.score() };
        let score_b = if Some(*b) == tt_move { i32::MAX } else { b.score() };
        score_b.cmp(&score_a)
    });

    let mut legal_moves_count = 0;
    let mut max_score = i64::MIN;
    let mut best_move = None;
    let old_alpha = context.alpha;

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

            if env.stopped {
                return 0;
            }

            if score > max_score {
                max_score = score;
                best_move = Some(candidate_move);
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

    let node_type = if max_score >= context.beta {
        NodeType::LowerBound
    } else if max_score > old_alpha {
        NodeType::Exact
    } else {
        NodeType::UpperBound
    };

    if !env.stopped {
        env.tt.store(TTEntry {
            zobrist_key: board.game_state.curr_zobrist_key,
            best_move,
            score: max_score,
            depth: context.depth,
            node_type,
        });
    }

    max_score
}

fn quiescense(board: &Board, mut context: SearchContext, env: &mut SearchEnv) -> i64 {
    env.nodes_visited += 1;

    if env.stopped || env.nodes_visited >= env.node_limit {
        env.stopped = true;
        return 0;
    }

    if (env.nodes_visited & NODE_CHECK_INTERVAL_MASK == 0) && env.check_stop() {
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

            if env.stopped {
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

fn search_fixed_depth(board: &Board, depth: i64, env: &mut SearchEnv) -> (i64, Option<Move>) {
    let tt_move = env.tt.get(board.game_state.curr_zobrist_key).and_then(|e| e.best_move);
    let mut moves = board.generate_pseudolegal_moves();
    moves.sort_by(|a, b| {
        let score_a = if Some(*a) == tt_move { i32::MAX } else { a.score() };
        let score_b = if Some(*b) == tt_move { i32::MAX } else { b.score() };
        score_b.cmp(&score_a)
    });

    let mut best_move = None;
    let mut max_score = i64::MIN;
    let mut alpha = -1_000_000;
    let beta = 1_000_000;

    for candidate_move in moves {
        if env.stopped {
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

            if env.stopped {
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

    if !env.stopped && best_move.is_some() {
        env.tt.store(TTEntry {
            zobrist_key: board.game_state.curr_zobrist_key,
            best_move,
            score: max_score,
            depth,
            node_type: NodeType::Exact,
        });
    }

    (max_score, best_move)
}

pub fn search(board: &Board, max_depth: i64, env: &mut SearchEnv) -> (i64, Option<Move>) {
    let mut global_best_move = None;
    let mut global_best_score = 0;

    for d in 1..=max_depth {
        let (score, best_move) = search_fixed_depth(board, d, env);

        if env.stopped {
            if global_best_move.is_none() && best_move.is_some() {
                global_best_move = best_move;
                global_best_score = score;
            }
            break;
        }

        if let Some(mv) = best_move {
            global_best_move = Some(mv);
            global_best_score = score;

            println!(
                "info depth {} score cp {} nodes {} pv {}",
                d, score, env.nodes_visited, mv
            );
        }
    }

    (global_best_score, global_best_move)
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub zobrist_key: u64,
    pub best_move: Option<Move>,
    pub score: i64,
    pub depth: i64,
    pub node_type: NodeType,
}

pub struct TT {
    entries: Vec<Option<TTEntry>>,
}

impl TT {                                                                                                                                                                                                           
    pub fn new(size_mb: usize) -> Self {                                                                                                                                                                        
        let num_entries = (size_mb * 2_usize.pow(20)) / std::mem::size_of::<Option<TTEntry>>();
        TT {                                                                                                                                                                                                        
            entries: vec![None; num_entries],                                                                                                                                                                       
        }                                                                                                                                                                                                           
    }

    pub fn get(&self, zobrist_key: u64) -> Option<TTEntry> {
        let index = (zobrist_key as usize) % self.entries.len();
        if let Some(entry) = self.entries[index] {
            if entry.zobrist_key == zobrist_key {
                return Some(entry);
            }
        }
        None
    }

    pub fn store(&mut self, entry: TTEntry) {
        let index = (entry.zobrist_key as usize) % self.entries.len();
        self.entries[index] = Some(entry);
    }
}

#[derive(Clone, Copy)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}