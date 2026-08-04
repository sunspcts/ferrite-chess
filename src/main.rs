mod bitboard;
mod board;
mod heuristics;
mod moves;
mod piece;


fn main() {
    let board = board::Board::new_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    println!("woop woop")
}
