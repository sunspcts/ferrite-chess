use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use crate::{board::Board, eval::eval, moves::{Move, MoveList}};
const MATE_EVAL: i64 = 30000;
const NODE_CHECK_INTERVAL_MASK: u64 = 2047; // Check search control every 2048 nodes
pub const MAX_PLY: usize = 256;

// Holds global search variables shared across the recursion
pub struct SearchEnv<'a> {
    pub nodes_visited: u64,
    pub node_limit: u64,
    pub hash_history: Vec<u64>,
    pub search_control: SearchControl,
    pub stopped: bool,
    pub age: u8,
    pub tt: &'a mut TT,
    pub move_lists: [MoveList; MAX_PLY],
}

impl<'a> SearchEnv<'a> {
    #[inline(always)]
    pub fn is_repetition(&self, key: u64, half_moves: usize) -> bool {
        self.hash_history.iter().rev().take(half_moves).any(|&k| k == key)
    }

    #[inline(always)]
    pub fn step_node_and_check(&mut self) -> bool {
        self.nodes_visited += 1;
        if self.stopped || self.nodes_visited >= self.node_limit {
            self.stopped = true;
            return true;
        }
        if (self.nodes_visited & NODE_CHECK_INTERVAL_MASK == 0) && self.search_control.is_stopped() {
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
    pub allow_nmp: bool,
} 

impl SearchContext {
    pub fn next_context(&self, depth: i64) -> Self {
        SearchContext {
            alpha: -self.beta,
            beta: -self.alpha,
            ply: self.ply + 1,
            depth,
            allow_nmp: true,
        }
    }

    #[inline]
    pub fn update_alpha(&mut self, score: i64) -> bool {
        if score > self.alpha {
            self.alpha = score;
        }
        self.alpha >= self.beta
    }

    #[inline]
    pub fn node_type(&self, max_score: i64, old_alpha: i64) -> NodeType {
        if max_score >= self.beta {
            NodeType::LowerBound
        } else if max_score > old_alpha {
            NodeType::Exact
        } else {
            NodeType::UpperBound
        }
    }
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


#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum NodeType {
    #[default]
    None = 0,
    Exact = 1,
    LowerBound = 2,
    UpperBound = 3,
}

#[derive(Clone, Copy, Default)]
pub struct TTEntry {
    pub zobrist_key: u64,
    pub score: i16,
    pub move_data: u16,
    pub depth: i8,
    pub node_type: NodeType,
    pub age: u8,
}

#[inline(always)]
fn score_to_tt(score: i64, ply: i64) -> i16 {
    if score > MATE_EVAL - 1000 {
        (score + ply) as i16
    } else if score < -MATE_EVAL + 1000 {
        (score - ply) as i16
    } else {
        score as i16
    }
}

#[inline(always)]
fn score_from_tt(score: i16, ply: i64) -> i64 {
    let s = score as i64;
    if s > MATE_EVAL - 1000 {
        s - ply
    } else if s < -MATE_EVAL + 1000 {
        s + ply
    } else {
        s
    }
}

impl TTEntry {
    pub fn best_move(&self) -> Option<Move> {
        if self.move_data == 0 {
            None
        } else {
            Some(Move::new_without_score(self.move_data))
        }
    }

    pub fn cutoff(&self, alpha: i64, beta: i64, depth: i64, ply: i64) -> Option<i64> {
        if (self.depth as i64) >= depth {
            let score = score_from_tt(self.score, ply);
            match self.node_type {
                NodeType::Exact => Some(score),
                NodeType::LowerBound if score >= beta => Some(score),
                NodeType::UpperBound if score <= alpha => Some(score),
                _ => None,
            }
        } else {
            None
        }
    }
}

pub struct TT {
    entries: Vec<TTEntry>,
}

impl TT {                                                                                                                                                                                                           
    pub fn new(size_mb: usize) -> Self {                                                                                                                                                                        
        let num_entries = (size_mb * 2_usize.pow(20)) / std::mem::size_of::<TTEntry>();
        TT {                                                                                                                                                                                                        
            entries: vec![TTEntry::default(); num_entries],                                                                                                                                                                       
        }                                                                                                                                                                                                           
    }

    pub fn clear(&mut self) {
        self.entries.fill(TTEntry::default());
    }

    pub fn get(&self, zobrist_key: u64) -> Option<TTEntry> {
        let index = (zobrist_key as usize) % self.entries.len();
        let entry = self.entries[index];
        if entry.node_type != NodeType::None && entry.zobrist_key == zobrist_key {
            Some(entry)
        } else {
            None
        }
    }

    pub fn store(&mut self, entry: TTEntry) {
        let index = (entry.zobrist_key as usize) % self.entries.len();
        let existing = self.entries[index];
        if existing.node_type != NodeType::None {
            if existing.zobrist_key == entry.zobrist_key {
                if existing.depth > entry.depth {
                    return;
                }
            } else {
                let is_stale = existing.age != entry.age;
                if !is_stale && existing.depth > entry.depth {
                    return;
                }
            }
        }
        self.entries[index] = entry;
    }
}

fn nmp(board: &Board, context: &SearchContext, env: &mut SearchEnv) -> Option<i64> {
    let reduction = 2 + context.depth / 6;

    if context.allow_nmp
        && context.depth >= 3
        && context.ply > 0
        && context.beta < MATE_EVAL - 100
        && !board.king_pawn_only()
        && !board.is_in_check()
        && eval(board) >= context.beta
    {
        let null_board = board.make_null_move();
        let null_context = SearchContext {
            alpha: -context.beta,
            beta: -context.beta + 1,
            depth: context.depth - 1 - reduction,
            ply: context.ply + 1,
            allow_nmp: false,
        };

        env.hash_history.push(board.game_state.curr_zobrist_key);
        let null_score = -negamax(&null_board, null_context, env);
        env.hash_history.pop();

        if env.stopped {
            return Some(0);
        }

        if null_score >= context.beta {
            return Some(context.beta);
        }
    }
    None
}

fn negamax(board: &Board, mut context: SearchContext, env: &mut SearchEnv) -> i64 {
    if env.step_node_and_check() { return 0; }

    if context.ply > 0 && env.is_repetition(board.game_state.curr_zobrist_key, board.game_state.half_moves as usize) {
        return 0;
    }

    let in_check = board.is_in_check();
    let depth = if in_check { context.depth + 1 } else { context.depth };

    if depth <= 0 {
        return quiescense(board, context, env);
    }

    let tt_entry = env.tt.get(board.game_state.curr_zobrist_key);
    let tt_move = tt_entry.and_then(|e| e.best_move());

    if context.ply > 0 {
        if let Some(score) = tt_entry.and_then(|e| e.cutoff(context.alpha, context.beta, depth, context.ply)) {
            return score;
        }
    }

    if let Some(nmp_score) = nmp(board, &context, env) {
        if env.stopped {
            return 0;
        }
        env.tt.store(TTEntry {
            zobrist_key: board.game_state.curr_zobrist_key,
            score: score_to_tt(nmp_score, context.ply),
            move_data: 0,
            depth: depth as i8,
            node_type: NodeType::LowerBound,
            age: env.age,
        });
        return nmp_score;
    }

    let ply = (context.ply as usize).min(MAX_PLY - 1);
    board.generate_pseudolegal_moves(&mut env.move_lists[ply]);
    let mut moves = env.move_lists[ply];

    moves.sort_by(|a, b| {
        let score_a = if Some(*a) == tt_move { i16::MAX } else { a.score() };
        let score_b = if Some(*b) == tt_move { i16::MAX } else { b.score() };
        score_b.cmp(&score_a)
    });

    let mut legal_moves_count = 0;
    let mut max_score = i64::MIN;
    let mut best_move = None;
    let old_alpha = context.alpha;

    for &candidate_move in &moves {
        if let Some(next_board) = board.make(candidate_move) {
            legal_moves_count += 1;

            env.hash_history.push(board.game_state.curr_zobrist_key);
            let score = -negamax(&next_board, context.next_context(depth - 1), env);
            env.hash_history.pop();

            if env.stopped {
                return 0;
            }

            if score > max_score {
                max_score = score;
                best_move = Some(candidate_move);
            }

            if context.update_alpha(score) {
                break;
            }
        }
    }

    if legal_moves_count == 0 {
        if in_check {
            return -MATE_EVAL + context.ply;
        } else {
            return 0; // Stalemate
        }
    }

    let node_type = context.node_type(max_score, old_alpha);

    if !env.stopped {
        env.tt.store(TTEntry {
            zobrist_key: board.game_state.curr_zobrist_key,
            score: score_to_tt(max_score, context.ply),
            move_data: best_move.map(|m| m.data()).unwrap_or(0),
            depth: depth as i8,
            node_type,
            age: env.age,
        });
    }

    max_score
}

fn quiescense(board: &Board, mut context: SearchContext, env: &mut SearchEnv) -> i64 {
    if env.step_node_and_check() { return 0; }
    
    let static_eval = eval(board);
    let mut best_value = static_eval;

    if best_value >= context.beta {
        return best_value;
    }
    if best_value > context.alpha {
        context.alpha = best_value;
    }

    let ply = (context.ply as usize).min(MAX_PLY - 1);
    board.generate_pseudolegal_moves(&mut env.move_lists[ply]);
    let mut moves = env.move_lists[ply];
    moves.retain(|mv| mv.is_capture() || mv.is_promo());
    moves.sort_by(|a, b| b.score().cmp(&a.score()));

    for &candidate_move in &moves {
        if let Some(next_board) = board.make(candidate_move) {
            let score = -quiescense(&next_board, context.next_context(0), env);

            if env.stopped {
                return 0;
            }

            if score > best_value {
                best_value = score;
            }

            if context.update_alpha(score) {
                return score;
            }
        }
    }

    best_value
}

fn search_fixed_depth(board: &Board, depth: i64, env: &mut SearchEnv) -> (i64, Option<Move>) {
    let tt_move = env.tt.get(board.game_state.curr_zobrist_key).and_then(|e| e.best_move());
    let ply = 0;
    board.generate_pseudolegal_moves(&mut env.move_lists[ply]);
    let mut moves = env.move_lists[ply];
    moves.sort_by(|a, b| {
        let score_a = if Some(*a) == tt_move { i16::MAX } else { a.score() };
        let score_b = if Some(*b) == tt_move { i16::MAX } else { b.score() };
        score_b.cmp(&score_a)
    });

    let mut best_move = None;
    let mut max_score = i64::MIN;
    let mut alpha = -1_000_000;
    let beta = 1_000_000;

    for &candidate_move in &moves {
        if env.stopped {
            break;
        }

        if let Some(next_board) = board.make(candidate_move) {
            let context = SearchContext {
                alpha: -beta,
                beta: -alpha,
                depth: depth - 1,
                ply: 1,
                allow_nmp: true,
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
            score: score_to_tt(max_score, 0),
            move_data: best_move.map(|m| m.data()).unwrap_or(0),
            depth: depth as i8,
            node_type: NodeType::Exact,
            age: env.age,
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
