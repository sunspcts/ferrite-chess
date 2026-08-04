mod attacks;
mod bitboard;
mod board;
mod heuristics;
mod moves;
mod piece;


fn main() {
    let square = 28;
    for dir in 0..8 {
        println!("{:?}", attacks::RAYS[dir][square])
    }
}
