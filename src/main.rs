pub use sparrow::*;

fn main() {
    println!("Hello, Shogi World!");

    // const STARTPOS: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1";
    println!("'{}'", board::SFEN_STARTPOS);

    let mut board = Board::default();
    println!("'{}'", board);

    println!("'{}'", board::SFEN_6PIECE_HANDICAP);
    board = Board::from_sfen(board::SFEN_6PIECE_HANDICAP).unwrap();
    println!("'{}'", board);

    println!("'{}'", board::SFEN_4PIECE_HANDICAP);
    board = board::SFEN_4PIECE_HANDICAP.parse().unwrap();
    println!("'{}'", board);

    println!("'{}'", board::SFEN_2PIECE_HANDICAP);
    board = board::SFEN_2PIECE_HANDICAP.parse().unwrap();
    println!("'{}'", board);

}
