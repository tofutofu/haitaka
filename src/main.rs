pub use sparrow::*;

/*
const fn get_between_rays(from: Square, to: Square) -> BitBoard {
    let dx = to.file() as i8 - from.file() as i8; // -8 .. 8
    let dy = to.rank() as i8 - from.rank() as i8; // -8 .. 8
    let orthogonal = dx == 0 || dy == 0;
    let diagonal = dx.abs() == dy.abs();
    if !(orthogonal || diagonal) {
        return BitBoard::EMPTY;
    }
    let dx = dx.signum(); // -1, 0, 1
    let dy = dy.signum();
    let mut square = from.offset(dx, dy);
    let mut between = BitBoard::EMPTY;
    while square as usize != to as usize {
        between.0 |= square.bitboard().0;
        square = square.offset(dx, dy);
    }
    between
}
*/

fn main() {
    println!("Hello, Shogi World!");

    println!("{:#?}", get_between_rays(Square::E5, Square::B2));

    println!("Done!");
}

#[allow(dead_code)]
pub fn test2() {
    let mut board = Board::default();

    assert_eq!(board.color_on(Square::I5).unwrap(), Color::Black);

    board.play("2g2f".parse().unwrap());
    board.play("8c8d".parse().unwrap());
    board.play("2f2e".parse().unwrap());
    board.play("8d8e".parse().unwrap());

    println!("'{}'", board);
}

#[allow(dead_code)]
fn test1() {
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
