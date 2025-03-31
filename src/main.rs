pub use sparrow::*;

// Using main - for now - as scratch pad.

fn main() {
    println!("Hello, Shogi World!");

    test4();

    println!("Done!");
}

fn test4() {
    let mv = Move::parse("P*7b").unwrap();

    if let Move::Drop { piece, to } = mv {
        println!("This is a drop move.");
        println!("Piece: {:?}, To square: {:?}", piece, to);
    } else {
        println!("This is not a drop move.");
    }

    let mv = Move::parse("B8hx3c+").unwrap();
    assert!(mv.is_board_move());
    assert!(mv.piece().is_none());
    assert_eq!(mv.from(), Some(Square::H8));
    assert_eq!(mv.to(), Square::C3);
    assert!(mv.is_promotion());
}

#[allow(dead_code)]
pub fn test3() {
    println!("{:#?}", get_between_rays(Square::E5, Square::B2));
}

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
