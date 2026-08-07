use super::*;
use crate::{board::Board, eval::eval};

pub(super) fn nmp(board: &Board, context: &SearchContext, env: &mut SearchEnv) -> Option<(i64, i64)> {
    if !context.nmp_allowed {
        return None;
    }

    let static_eval = eval(board);
    if static_eval < context.beta {
        return None;
    }

    let reduction = 3 + context.depth / 6;
    let null_depth = context.depth - reduction;

    if null_depth <= 0 {
        return None;
    }

    if context.depth >= 3 
        && context.ply > 0 
        && context.beta < MATE_EVAL - 100 
        && !board.king_pawn_only() 
        && !board.is_in_check() 
    {
        let null_board = board.make_null_move();

        let null_context = SearchContext {
            alpha: -context.beta,
            beta: -(context.beta - 1),
            depth: null_depth,
            ply: context.ply + 1,
            nmp_allowed: false,
        };

        env.hash_history.push(board.game_state.curr_zobrist_key);
        let score = -negamax(&null_board, null_context, env);
        env.hash_history.pop();

        if score >= context.beta {
            return Some((context.beta, null_depth));
        }
    }
    None
}
