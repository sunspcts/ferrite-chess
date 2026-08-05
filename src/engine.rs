use std::{io::{self, BufRead}, thread};

use crate::{board::Board, moves::Move, search::SearchControl};

const ENGINE_NAME: &str = "Ferrite";
const ENGINE_AUTHOR: &str = "sunspcts";

const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn engine() {
    let stdin = io::stdin();
    let mut board = Board::new_from_fen(STARTPOS_FEN);
    let mut hash_history = vec![board.game_state.curr_zobrist_key];
    let mut search_control = SearchControl::new();
    let mut search_thread: Option<thread::JoinHandle<()>> = None;

    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("uci") => {
                println!("id name {}", ENGINE_NAME);
                println!("id author {}", ENGINE_AUTHOR);
                println!("uciok");
            }
            Some("isready") => println!("readyok"),
            Some("position") => {
                stop_search(&mut search_thread, &mut search_control);
                (board, hash_history) = parse_uci_position(board, line);
            }
            Some("ucinewgame") => {
                stop_search(&mut search_thread, &mut search_control);
                board = Board::new_from_fen(STARTPOS_FEN);
                hash_history = vec![board.game_state.curr_zobrist_key];
            }
            Some("stop") => {
                stop_search(&mut search_thread, &mut search_control);
                if let Some(handle) = search_thread.take() {
                    let _ = handle.join();
                }
            }
            Some("quit") => {
                stop_search(&mut search_thread, &mut search_control);
                if let Some(handle) = search_thread.take() {
                    let _ = handle.join();
                }
                break;
            }
            _ => {}
        }
    }

    let _ = (board, hash_history);
}

pub fn stop_search(search_thread: &mut Option<thread::JoinHandle<()>>, search_control: &mut SearchControl) {
    search_control.stop();
    if let Some(handle) = search_thread.take() {
        let _ = handle.join();
    }
}

#[derive(Default)]
struct GoParameters {
    depth: Option<i64>,
    movetime: Option<u64>,
    nodes: Option<u64>,
    wtime: Option<u64>,
    btime: Option<u64>,
    winc: Option<u64>,
    binc: Option<u64>,
    infinite: bool
}

// Handles all possible go parameters
fn parse_go(line: &str) -> GoParameters {
    let mut parts = line.split_whitespace();
    
    let mut params = GoParameters::default();

    while let Some(part) = parts.next() {
        match part {
            "depth" => {
                if let Some(val) = parts.next() {
                    if let Ok(parsed) = val.parse::<i64>() {
                        params.depth = Some(parsed);
                    }
                }
            }
            "movetime" => {
                if let Some(val) = parts.next() {
                    if let Ok(parsed) = val.parse::<u64>() {
                        params.movetime = Some(parsed);
                    }
                }
            }
            "nodes" => {
                if let Some(val) = parts.next() {
                    if let Ok(parsed) = val.parse::<u64>() {
                        params.nodes = Some(parsed);
                    }
                }
            }
            "wtime" => {
                if let Some(val) = parts.next() {
                    if let Ok(parsed) = val.parse::<u64>() {
                        params.wtime = Some(parsed);
                    }
                }
            }
            "btime" => {
                if let Some(val) = parts.next() {
                    if let Ok(parsed) = val.parse::<u64>() {
                        params.btime = Some(parsed);
                    }
                }
            }
            "winc" => {
                if let Some(val) = parts.next() {
                    if let Ok(parsed) = val.parse::<u64>() {
                        params.winc = Some(parsed);
                    }
                }
            }
            "binc" => {
                if let Some(val) = parts.next() {
                    if let Ok(parsed) = val.parse::<u64>() {
                        params.binc = Some(parsed);
                    }
                }
            }
            "infinite" => params.infinite = true,
            _ => {}
            
        }
    }

    params
}

fn parse_uci_position(curr_board: Board, line: &str) -> (Board, Vec<u64>) {
    let mut parts = line.split_whitespace();
    let mut board = curr_board;

    let _ = parts.next();
    let mode = parts.next();

    if mode == Some("fen") {
        let fen_parts: Vec<&str> = parts.by_ref().take_while(|part| *part != "moves").collect();
        let fen = fen_parts.join(" ");
        board = Board::new_from_fen(&fen);
    } else if mode == Some("startpos") {
        board = Board::new_from_fen(STARTPOS_FEN);
    }

    let mut hash_history = vec![board.game_state.curr_zobrist_key];

    for m in parts {
        if m == "moves" {
            continue
        } 

        if let Some(mv) = Move::from_uci(&board, m) {
            if let Some(next_board) = board.make(mv) {
                board = next_board;
                hash_history.push(board.game_state.curr_zobrist_key);
            }
        }
    }
    (board, hash_history)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uci_position_from_startpos() {
        let startpos_board = Board::new_from_fen(STARTPOS_FEN);
        //Scotch my beloved <3
        let (board, _) = parse_uci_position(startpos_board, "position startpos moves e2e4 e7e5 g1f3 b8c6 d2d4");
        let fen_board = Board::new_from_fen("r1bqkbnr/pppp1ppp/2n5/4p3/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq d3 0 3"); 

        assert_eq!(board, fen_board)
    }

    
}