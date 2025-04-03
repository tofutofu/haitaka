pub use sparrow::*;
// Using main - for now - as scratch pad.

fn main() {
    println!("Hello, Shogi World!");

    test11();

    println!("Done!");
}

pub fn test12() {
    let mut board = Board::default(); // empty board
    board.unchecked_set_hand(Color::Black, Piece::Pawn, 5);
    board.unchecked_put(Color::Black, Piece::Pawn, Square::G1);
    board.generate_drops(|moves| {
        for mv in moves {
            println!("Move: {:?}", mv);
        }
        false
    });
    println!("Done");
}

pub fn test11() {
    let board = Board::startpos();
    let mut history = Vec::new();
    let depth = 7;
    let nodes = perft(&board, depth, &mut history);
    println!("depth={} nodes ={}", depth, nodes);
}

fn perft(board: &Board, depth: u8, history: &mut Vec<Move>) -> u64 {
    let mut nodes: u64 = 0;
    if depth == 0 {
        1
    } else if depth == 1 {
        board.generate_board_moves(|moves| {
            nodes += moves.into_iter().len() as u64;
            false
        });
        nodes
    } else {
        let mut nodes = 0;
        let mut err = 0;
        board.generate_board_moves(|moves| {
            for mv in moves {
                let mut board = board.clone();
                if board.is_legal(mv) {
                    board.play_unchecked(mv);
                    history.push(mv);
                    nodes += perft(&board, depth - 1, history);
                    history.pop();
                } else {
                    println!("Err History:");
                    for (i, &m) in history.iter().enumerate() {
                        println!("{}. {}", i + 1, m);
                    }
                    println!("{}. {} <<< non-legal?", history.len() + 1, mv);
                    err += 1;
                    if err >= 2 {
                        panic!("Err depth={} move={:?} history={:?}", depth, mv, history);
                    }
                }
            }
            false
        });
        nodes
    }
}

pub fn test10() {
    /*
    let board = Board::startpos();
    if board.generate_moves(|_| { println!("listener called"); true }) {
        println!("got some moves");
    } else {
        println!("no moves!");
    } */

    let sfen = "ln2k2nl/2g1G2+R1/p1pppp2p/6p2/9/2P6/P1+bPPPP1P/5K3/L1+rS1GSNL w S2Pbgsn2p 34";
    let board = Board::from_sfen(sfen).unwrap();
    if board.generate_moves(|_| {
        println!("listener called");
        true
    }) {
        println!("got some moves");
    } else {
        println!("no moves!");
    }

    let res = board.generate_moves(|mv| {
        println!("{:?}", mv);
        true
    });
    println!("generate_moves returned: {}", res);
}

pub fn test9() {
    let sfen: &str = "lnsgk2nl/1r4gs1/p1pppp1pp/1p4p2/7P1/2P6/PP1PPPP1P/1SG4R1/LN2KGSNL b Bb 11";
    let board = Board::from_sfen(sfen).unwrap();
    assert_eq!(board.side_to_move(), Color::Black);
    let hand = board.hand(Color::Black);
    assert_eq!(hand[Piece::Bishop as usize], 1);
    let occ = board.occupied();
    let mut num_drops = 0;
    board.generate_drops(|moves| {
        println!("{}", moves.len());

        match moves {
            PieceMoves::Drops { color, piece, to } => {
                println!("{} {:?} To: {:#?}", color, piece, to);
                assert_eq!(to, !occ);
            }
            _ => {
                println!("Huh?");
            }
        }

        for mv in moves {
            println!("{}", mv);
            assert!(mv.is_drop());
            num_drops += 1;
            //if num_drops > 3 {
            //    break;
            //}
        }

        false
    });
    assert_eq!(num_drops, 81 - occ.len());
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
            if let Move::BoardMove { .. } = mv {
                println!("ok");
            }
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
