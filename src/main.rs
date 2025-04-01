pub use sparrow::*;
// Using main - for now - as scratch pad.

fn main() {
    println!("Hello, Shogi World!");

    test8();

    println!("Done!");
}

pub fn test8() {
    let board = Board::startpos();
    //let piece = Piece::Pawn;
    //let from = board.pieces(piece);
    let mut num_moves = 0;
    board.generate_moves(|moves| {
        // Done this way for demonstration.
        // Actual counting is best done in bulk with moves.len().
        for mv in moves {
            println!("{}", mv);
            num_moves += 1;
        }
        false
    });
    //println!("{:?} moves: {}", piece, num_moves);
    println!("All moves: {}", num_moves);
}

pub fn test7() {
    let mut board = Board::startpos();
    board.play("2g2f".parse().unwrap());
    board.play("8c8d".parse().unwrap());
    board.play("2f2e".parse().unwrap());
    board.play("8d8e".parse().unwrap());
    println!("'{}'", board);
}

pub fn test6() {
    let sfen: &str = "ln3gsn1/7kl/3+B1p1p1/p4s2p/2P6/P2B3PP/1PNP+rPP2/2G3SK1/L4G1NL b G3Prs3p 65";
    let mut board = Board::from_sfen(sfen).unwrap();
    let mut pinned = board.pinned();
    println!("occ {:#?}", board.occupied());
    println!("white {:#?}", board.colors(Color::White));
    println!("black {:#?}", board.colors(Color::Black));
    println!("pinned {:#?}", pinned); // nothing yet - since for Black D4 is not a pinned piece!
    let mv = Move::BoardMove {
        from: Square::C6,
        to: Square::A4,
        promotion: false,
    };
    assert!(board.is_legal(mv));

    let mv1: Result<Move, _> = "6c4a".try_into();
    match mv1 {
        Ok(mv1) => println!("Parsed move: {:?}", mv1),
        Err(e) => println!("Failed to parse move: {}", e),
    }

    let mv2: Move = "6c4a".try_into().expect("huh");

    board.play(mv2);

    println!("occ {:#?}", board.occupied());
    pinned = board.pinned();
    println!("pinned {:#?}", pinned);
}

#[allow(dead_code)]
fn test5() {
    let bb = Square::A1.bitboard();
    println!("A1: {:#?}", bb);
    let shifted = bb.shift(Square::B1, Square::G2);
    println!("B1->G2: {:#?}", shifted);

    let mut board = Board::default();
    let color = board.side_to_move();
    println!("B {:#?}", board.colors(color));
    // println!("W {:#?}", !board.colors(color));
    println!("{:#?}", board.occupied());
    println!("0x{:x}", board.hash());
    println!("{} to move", board.side_to_move());

    assert_eq!(color, Color::Black);
    let mut mv = "2g2f".parse().unwrap();
    /*
    if let Move::BoardMove { from, to, .. } = mv {
        println!("Move: {}", mv);
        println!("{}: {:#?}", from, from.bitboard());
        println!("{}: {:#?}", to, to.bitboard());
        println!("{:?}", board.piece_on(Square::G2).unwrap());
        println!("Pawn attacks {:#?}", pawn_attacks(color, from));
        println!("Has {}: {}", to, pawn_attacks(color, from).has(to));
    }
    */
    board.play(mv);
    println!("{:#?}", board.occupied());
    println!("0x{:x}", board.hash());
    println!("{} to move", board.side_to_move());

    mv = "8c8d".parse().unwrap();
    board.play(mv);
    println!("{:#?}", board.occupied());
    println!("0x{:x}", board.hash());
    println!("{} to move", board.side_to_move());

    mv = "2f2e".parse().unwrap();
    board.play(mv);
    println!("{:#?}", board.occupied());
    println!("0x{:x}", board.hash());
    println!("{} to move", board.side_to_move());
}

#[allow(dead_code)]
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
