use crate::board::Board;

use super::MoveList;

impl Board {
    #[cfg(test)]
    pub fn perft(&self, depth: u8) -> u64 {
        let mut move_lists = [MoveList::default(); 256];
        self.perft_helper(depth, 0, &mut move_lists)
    }

    #[cfg(test)]
    fn perft_helper(&self, depth: u8, ply: usize, move_lists: &mut [MoveList; 256]) -> u64 {
        if depth == 0 {
            return 1;
        }

        let ply_idx = ply.min(255);
        self.generate_pseudolegal_moves(&mut move_lists[ply_idx]);
        let moves = move_lists[ply_idx];

        if depth == 1 {
            let mut count = 0;
            for &m in &moves {
                if self.make(m).is_some() {
                    count += 1;
                }
            }
            return count;
        }

        let mut nodes = 0;
        for &m in &moves {
            if let Some(next_board) = self.make(m) {
                nodes += next_board.perft_helper(depth - 1, ply + 1, move_lists);
            }
        }

        nodes
    }
}

#[test]
fn perft_startpos() {
    let board = Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let start = std::time::Instant::now();
    let nodes = board.perft(5);
    let elapsed = start.elapsed();
    println!("perft(5): {} nodes in {:.3?}, {:.2} MNPS", nodes, elapsed, (nodes as f64 / elapsed.as_secs_f64()) / 1_000_000.0);
    assert_eq!(nodes, 4865609);
}