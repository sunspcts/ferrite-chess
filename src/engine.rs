use std::{io::{self, BufRead}, thread};

use crate::{board::Board, search::SearchControl};

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
                println!("option name Hash type spin default 16 min 1 max 1024");
                println!("uciok");
            }
            Some("isready") => println!("readyok"),
            Some("quit") => break,
            _ => {}
        }
    }
}