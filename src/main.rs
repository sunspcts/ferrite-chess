use board::init_boardbuilder_from_fen;

mod board;
mod movegen;

fn main() {
    let boardbuilder = init_boardbuilder_from_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPPKPPP/RNBQ1BNR b kq - 1 2");

    let board = boardbuilder.init();

    println!("Woo!")
}
