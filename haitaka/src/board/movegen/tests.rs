use rand::rng;
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;

// Movegenerator tests
use super::*;
use std::collections::HashSet;

// Tests the generation of board moves based on giving a subset of squares
#[test]
fn subset_movegen_habu_position() {
    fn visit(board: &Board, depth: u8) {
        let random = board.hash();
        let subset_a = BitBoard::new(random.into());
        let subset_b = !subset_a;
        let mut subset_moves = 0;

        board.generate_board_moves_for(subset_a, |moves| {
            subset_moves += moves.len();
            false
        });
        board.generate_board_moves_for(subset_b, |moves| {
            subset_moves += moves.len();
            false
        });

        let mut total_moves = 0;
        board.generate_board_moves(|moves| {
            total_moves += moves.len();
            false
        });
        assert_eq!(subset_moves, total_moves);

        if depth > 0 {
            board.generate_moves(|moves| {
                for mv in moves {
                    let mut board = board.clone();
                    board.play_unchecked(mv);
                    visit(&board, depth - 1);
                }
                false
            });
        }
    }
    // Famous Habu-Kato game (https://en.wikipedia.org/wiki/Shogi_notation)
    // with sublime Silver drop sacrifice on square 5b
    let board = "ln1g5/1r2S1k2/p2pppn2/2ps2p2/1p7/2P6/PPSPPPPLP/2G2K1pr/LN4G1b w BGSLPnp 62"
        .parse()
        .unwrap();
    visit(&board, 2);
}

fn test_is_legal(board: Board) {
    // both board_moves and drops are included
    let mut legals = HashSet::new();
    board.generate_moves(|mvs| {
        legals.extend(mvs);
        false
    });

    for from in Square::ALL {
        for to in Square::ALL {
            for promotion in [true, false] {
                let mv = Move::BoardMove {
                    from,
                    to,
                    promotion,
                };
                assert_eq!(legals.contains(&mv), board.is_legal(mv), "{}", mv);
            }
        }
    }
}

fn test_forbidden_drops(board: &Board) {
    let mut legals = HashSet::new();
    board.generate_drops(|mvs| {
        legals.extend(mvs);
        false
    });

    let forbidden = match board.side_to_move() {
        Color::White => Rank::I.bitboard(),
        Color::Black => Rank::A.bitboard(),
    };

    let forbidden_for_knight = match board.side_to_move() {
        Color::White => Rank::H.bitboard(),
        Color::Black => Rank::B.bitboard(),
    };

    for to in forbidden {
        for piece in [Piece::Pawn, Piece::Lance, Piece::Knight] {
            let mv = Move::Drop { piece, to };
            assert!(!legals.contains(&mv));
        }
    }

    for to in forbidden_for_knight {
        let mv = Move::Drop {
            piece: Piece::Knight,
            to,
        };
        assert!(!legals.contains(&mv));
    }
}

fn test_nifu(board: &Board) {
    let color = board.side_to_move();

    board.generate_drops_for(Piece::Pawn, |mvs| {
        for mv in mvs {
            assert_eq!(mv.piece().unwrap(), Piece::Pawn);
            assert!(board.pawn_drop_ok(color, mv.to()));
        }
        false
    });

    let pawns = board.colored_pieces(color, Piece::Pawn);
    for square in pawns {
        let forbidden = square.file().bitboard() & !board.occupied();
        for to in forbidden {
            let mv = Move::Drop {
                piece: Piece::Pawn,
                to,
            };
            assert!(!board.is_legal_drop(mv));
        }
    }
}

#[test]
fn legality_simple() {
    test_is_legal(Board::startpos());
    test_is_legal(
        "ln1g5/1r2S1k2/p2pppn2/2ps2p2/1p7/2P6/PPSPPPPLP/2G2K1pr/LN4G1b w BGSLPnp 62"
            .parse()
            .unwrap(),
    );
}

#[test]
fn legality_drops() {
    let board: Board = "ln1g5/1r2S1k2/p2pppn2/2ps2p2/1p7/2P6/PPSPPPPLP/2G2K1pr/LN4G1b w BGSLPnp 62"
        .parse()
        .unwrap();
    test_forbidden_drops(&board);
    test_nifu(&board);
}

#[test]
fn non_check() {
    let sfen: &str = "lnsgk1snl/1r4gb1/p1pppp2p/6pR1/1p7/2P6/PP1PPPP1P/1BG6/LNS1KGSNL w Pp 12";
    let board: Board = sfen.parse().unwrap();
    let checkers = board.checkers();
    assert!(checkers.is_empty());
}

#[test]
fn pawn_push_mate_is_valid() {
    // White King on 1e is almost mate
    let sfen = "lns4+Rl/1r1g5/p1p1pSp1p/1p1p1p3/8k/7N1/PPPPPPP1P/1B7/LNSGKGSNL b BG2p 25";
    let board: Board = sfen.parse().unwrap();
    assert!(board.checkers().is_empty());

    assert_eq!(board.side_to_move(), Color::Black);
    let mv = Move::Drop {
        piece: Piece::Gold,
        to: Square::F1,
    };
    assert!(board.is_legal_drop(mv));
    assert!(board.is_legal(mv));

    let mv = Move::Drop {
        piece: Piece::Gold,
        to: Square::E2,
    };
    assert!(board.is_legal_drop(mv));
    assert!(board.is_legal(mv));

    let mv = Move::BoardMove {
        from: Square::G1,
        to: Square::F1,
        promotion: false,
    };
    assert!(board.is_legal_board_move(mv));
    assert!(board.is_legal(mv));
}

#[test]
fn discount_pawn_drop_mate_in_perft() {
    // See old discussion at: https://www.talkchess.com/forum3/viewtopic.php?f=7&t=71550
    //
    // Testing this SFEN did expose a bug in the haitaka 0.2.1 code:
    // When generating Pawn drops, all drops would be skipped if the first drop we looked
    // at happened to be an illegal checkmate.
    let sfen: &str = "7lk/9/8S/9/9/9/9/7L1/8K b P 1";
    let board: Board = Board::tsume(sfen).unwrap();
    assert_eq!(board.side_to_move(), Color::Black);
    assert!(board.has_in_hand(Color::Black, Piece::Pawn));

    let mut num_moves = 0;
    board.generate_moves(|mvs| {
        // remember that the listener may be called back multiple times
        num_moves += mvs.into_iter().len();
        false
    });
    assert_eq!(num_moves, 85);
}

#[test]
fn donot_move_into_check() {
    let sfen: &str = "7lk/9/8S/9/9/9/9/7L1/8K b P 1";
    let mut board: Board = Board::tsume(sfen).unwrap();
    assert_eq!(board.side_to_move(), Color::Black);

    // Ki1-h1
    let mv = Move::BoardMove {
        from: Square::I1,
        to: Square::H1,
        promotion: false,
    };
    assert!(board.is_legal(mv));

    board.play_unchecked(mv);
    assert_eq!(board.side_to_move(), Color::White);
    assert_eq!(board.checkers, BitBoard::EMPTY);

    // L2ax2g+
    let mv = Move::BoardMove {
        from: Square::A2,
        to: Square::H2,
        promotion: true,
    };
    assert!(board.is_legal(mv));
    board.play_unchecked(mv);

    assert_eq!(board.side_to_move(), Color::Black);
    assert_eq!(board.checkers.len(), 1);
    assert!(board.checkers.has(Square::H2));
    assert_eq!(board.piece_on(Square::H2).unwrap(), Piece::PLance);
    assert_eq!(board.color_on(Square::H2).unwrap(), Color::White);

    let mv = Move::Drop {
        piece: Piece::Pawn,
        to: Square::E5,
    };
    assert!(!board.is_legal(mv));

    board.generate_moves(|mvs| {
        for mv in mvs {
            assert!(mv.is_board_move());
            let from: Square = mv.from().unwrap();
            let piece = board.piece_on(from).unwrap();
            assert_eq!(piece, Piece::King);
        }
        false
    });
}

#[test]
fn no_drop_on_top() {
    let board: Board = "ln1g5/1r4k2/p2pppn2/2ps2p2/1p7/2P6/PPSPPPPLP/2G2K1pr/LN4G1b b BG2SLPnp 61"
        .parse()
        .unwrap();
    assert_eq!(board.side_to_move(), Color::Black);
    let open_squares = !board.occupied();
    board.generate_drops(|mvs| {
        for mv in mvs {
            assert!(open_squares.has(mv.to()));
        }
        false
    });
}

#[test]
fn checkers_are_updated() {
    let sfen: &str = "7lk/9/8S/9/9/9/9/7L1/8K b P 1";
    let mut board: Board = Board::tsume(sfen).unwrap();

    // After K1i2i L2ax2h the Black King should be in check
    // and only King moves should be legal

    let mv1 = Move::BoardMove {
        from: Square::I1,
        to: Square::I2,
        promotion: false,
    };
    let mv2 = Move::BoardMove {
        from: Square::A2,
        to: Square::H2,
        promotion: false,
    };
    let mv3 = Move::BoardMove {
        from: Square::C1,
        to: Square::D2,
        promotion: true,
    };

    board.play_unchecked(mv1);
    assert_eq!(board.side_to_move(), Color::White);
    assert_eq!(board.checkers().len(), 0);

    board.play_unchecked(mv2);
    assert_eq!(board.side_to_move(), Color::Black);
    assert_eq!(board.checkers().len(), 1);
    assert!(board.checkers.has(Square::H2));
    assert!(!board.is_legal(mv3));
}

#[test]
fn tsume() {
    let sfen = "lpg6/3s2R2/1kpppp3/p8/9/P8/2N6/9/9 b BGN 1";
    // from_sfen will fail - since there is only one King on board
    assert!(matches!(
        Board::from_sfen(sfen),
        Err(SFENParseError::InvalidBoard)
    ));
    // tsume will succeed
    let board = Board::tsume(sfen).unwrap();
    assert!(board.has(Color::White, Piece::King));
    assert!(!board.has(Color::Black, Piece::King));
    assert_eq!(board.num_in_hand(Color::White, Piece::Gold), 2);
    assert_eq!(board.num_in_hand(Color::White, Piece::Silver), 3);
}

#[test]
fn generate_checks() {
    let sfen = "lpg6/3s2R2/1kpppp3/p8/9/P8/2N6/9/9 b BGN 1";
    let board = Board::tsume(sfen).unwrap();

    let mut nmoves: usize = 0;
    let mut nmoves_iter: usize = 0;
    let mut nmoves_into_iter: usize = 0;

    board.generate_board_moves(|mvs| {
        for mv in mvs {
            assert!(mv.is_board_move());
            nmoves += 1;
        }
        nmoves_iter += mvs.len();
        nmoves_into_iter += mvs.into_iter().len();
        false
    });
    assert_eq!(nmoves, 29);
    assert_eq!(nmoves_iter, 16); // this doesn't count promotions
    assert_eq!(nmoves_into_iter, 29); // should match nmoves

    let mut nchecks: usize = 0;
    let mut nchecks_iter: usize = 0;
    let mut nchecks_into_iter: usize = 0;

    board.generate_checks(|mvs| {
        for mv in mvs {
            assert!(mv.is_drop());
            nchecks += 1;
        }
        nchecks_iter += mvs.len();
        nchecks_into_iter += mvs.len();
        false
    });
    assert_eq!(nchecks, 15);
    assert_eq!(nchecks_iter, 15);
    assert_eq!(nchecks_into_iter, 15);
}

#[test]
fn play_tsume() {
    // first tsume in Zoku Tsumu-ya-Tsumuzaru-ya
    // by the First Meijin, Ohashi Sokei
    let sfen = "lpg6/3s2R2/1kpppp3/p8/9/P8/2N6/9/9 b BGN 1";
    let mut board = Board::tsume(sfen).unwrap();

    assert_eq!(board.side_to_move(), Color::Black);
    assert_eq!(board.status(), GameStatus::Ongoing);

    let moves = "\
        N*7e K8c-7b 
        B*8c K7b-8b 
        B8c-9b+ L9ax9b 
        R3bx6b= G7ax6b 
        S*8c K8b-9c 
        S8c-9b+ K9cx9b 
        G*8c K9b-9a L*9b";
    let moves: Vec<Move> = moves
        .split_ascii_whitespace()
        .map(|s| Move::parse(s).unwrap())
        .collect();

    for mv in moves {
        let mut v: Vec<Move> = Vec::new();

        if board.side_to_move() == Color::Black {
            board.generate_checks(|mvs| {
                v.extend(mvs);
                false
            });
        } else {
            assert_eq!(board.checkers.len(), 1);
            board.generate_moves(|mvs| {
                v.extend(mvs);
                false
            });
        }

        assert!(v.contains(&mv), "Move {mv} not found");
        board.play(mv);
    }

    assert_eq!(board.side_to_move(), Color::White);
    assert_eq!(board.status(), GameStatus::Won); // meaning: won for Black
}

#[test]
fn invalid_tsume() {
    // invalid position: White King is in check
    let sfen = "8l/5gB2/7Gk/7p1/7sp/9/9/9/9 b R";
    assert!(Board::tsume(sfen).is_err());
}

#[test]
fn discovered_checks1() {
    let sfen = "8l/5gB2/7G1/7pk/7sp/9/9/9/9 b R";
    let board = Board::tsume(sfen).unwrap();

    let checkers = board.checkers();
    let pinned = board.pinned();
    assert!(checkers.is_empty());
    assert!(pinned.is_empty());

    let mut moves: Vec<Move> = Vec::new();
    let mut checks: Vec<Move> = Vec::new();

    let gold = board.pieces(Piece::Gold);
    board.generate_board_moves_for(gold, |mvs| {
        moves.extend(mvs);
        false
    });
    assert_eq!(moves.len(), 5);

    board.generate_checks(|mvs| {
        for mv in mvs {
            if mv.is_board_move() {
                checks.push(mv);
            }
        }
        false
    });
    assert_eq!(checks.len(), 5);
    let mv: Move = "2c1b".parse::<Move>().unwrap();
    assert!(checks.contains(&mv));

    checks.clear();
    board.generate_checks(|mvs| {
        checks.extend(mvs);
        false
    });
    assert_eq!(checks.len(), 7);
}

#[test]
fn pinners() {
    let sfen = "8l/5gB2/8k/7p1/7sp/9/9/9/8K b RG";
    let mut board = Board::tsume(sfen).unwrap();

    assert!(board.checkers.is_empty());
    assert!(board.pinned.is_empty());

    let mv: Move = "G*2c".parse::<Move>().unwrap();
    assert!(board.is_legal(mv));
    board.play_unchecked(mv);

    assert!(board.checkers.len() == 1);
    assert!(board.pinned.is_empty());

    let mv: Move = "1c1d".parse::<Move>().unwrap();
    assert!(board.is_legal(mv));
    board.play_unchecked(mv);

    assert!(board.checkers.len() == 0);
    assert!(board.pinned.is_empty());
}

#[test]
fn undiscovered_checks() {
    /*
        R . . . . . G . k
        . . . . . . P . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
        . . . . . . . . .
    */

    let sfen = "R5G1k/6P2/9/9/9/9/9/9/9 b - 1";
    let board = Board::tsume(sfen).unwrap();

    let mv: Move = "3a2a".parse::<Move>().unwrap();
    assert_eq!(
        mv,
        Move::BoardMove {
            from: Square::A3,
            to: Square::A2,
            promotion: false
        }
    );
    assert!(board.is_legal(mv));

    let mut moves: Vec<Move> = Vec::new();
    let mut checks: Vec<Move> = Vec::new();

    board.generate_moves(|mvs| {
        moves.extend(mvs);
        false
    });

    board.generate_checks(|mvs| {
        checks.extend(mvs);
        false
    });

    assert!(moves.contains(&mv));
    assert!(checks.contains(&mv));
    assert_eq!(checks.len(), 1);
}

#[test]
fn discovered_checks2() {
    /*
    . . . . . . . . .
    . . . . . . . k .
    . . . . . . . . .
    . . . . . S . . .
    . . . . . . . . .
    . . . B . . . . .
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
    */

    let sfen = "9/7k1/9/5S3/9/3B5/9/9/9 b - 1";
    let board = Board::tsume(sfen).unwrap();

    let mv = "4d3c".parse::<Move>().unwrap();
    assert!(board.is_legal(mv));

    let mv: Move = "4d3c+".parse::<Move>().unwrap();
    assert!(board.is_legal(mv));

    let silver = Square::D4.bitboard();

    let mut moves: Vec<Move> = Vec::new();
    let mut checks: Vec<Move> = Vec::new();

    board.generate_board_moves_for(silver, |mvs| {
        moves.extend(mvs);
        false
    });

    board.generate_checks(|mvs| {
        checks.extend(mvs);
        false
    });

    assert_eq!(moves.len(), 8);
    assert_eq!(checks.len(), 7);

    let mv = "4d3c".parse::<Move>().unwrap();
    assert!(moves.contains(&mv));
    assert!(checks.contains(&mv));

    let mv = "4d3c+".parse::<Move>().unwrap();
    assert!(moves.contains(&mv));
    assert!(checks.contains(&mv));

    let mv = "4d5e".parse::<Move>().unwrap();
    assert!(moves.contains(&mv));
    assert!(!checks.contains(&mv));
}

#[test]
fn discovered_checks3() {
    /*
    . . . . . . . . .
    . . . . . . . k .
    . . . . . . . . .
    . . . . . . . S .
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . L .
    . . . . . . . . .
    */

    let sfen = "9/7k1/9/7S1/9/9/9/7L1/9 b -";
    let board = Board::tsume(sfen).unwrap();

    let mut moves: Vec<Move> = Vec::new();
    let mut checks: Vec<Move> = Vec::new();

    board.generate_moves(|mvs| {
        moves.extend(mvs);
        false
    });

    board.generate_checks(|mvs| {
        checks.extend(mvs);
        false
    });

    assert_eq!(moves.len(), 11);
    assert_eq!(checks.len(), 8);
}

#[test]
fn fuzzing_generate_moves() {
    let mut rng = rng();

    fn rollout(board: &mut Board, depth: usize, rng: &mut ThreadRng) -> bool {
        if depth == 0 {
            return true;
        }
        let mut v: Vec<Move> = Vec::new();
        board.generate_moves(|mvs| {
            v.extend(mvs);
            false
        });

        if v.is_empty() {
            return true;
        }
        let mv = v.choose(rng).unwrap();
        board.play_unchecked(*mv);
        rollout(board, depth - 1, rng)
    }
    for _ in 0..100 {
        let mut board = Board::startpos();
        assert!(rollout(&mut board, 100, &mut rng));
    }
}

#[test]
fn fuzzing_checks() {
    let mut rng = rng();

    // Zoku Tsumuya Tsumazaruya #198
    let sfen = "+P+n1g1+Pp+P1/2gg+p+s+pLn/1gppP1S+Pp/1+s+PPSPPPk/N1L2N+PL1/6L1+P/9/9/9 b - 1";

    fn rollout(board: &mut Board, depth: usize, rng: &mut ThreadRng) -> bool {
        if depth == 0 {
            return true;
        }
        let color = board.side_to_move();
        let mut v: Vec<Move> = Vec::new();
        if color == Color::Black {
            board.generate_checks(|mvs| {
                v.extend(mvs);
                false
            });
        } else {
            board.generate_moves(|mvs| {
                v.extend(mvs);
                false
            });
        }

        if v.is_empty() {
            return true;
        }
        let mv = v.choose(rng).unwrap();
        board.play(*mv);
        rollout(board, depth - 1, rng)
    }
    for _ in 0..200 {
        let mut board = Board::tsume(sfen).unwrap();
        assert!(rollout(&mut board, 100, &mut rng));
    }
}
