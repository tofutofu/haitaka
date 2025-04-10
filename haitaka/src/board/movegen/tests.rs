// Movegenerator tests
use super::*;
use std::collections::HashSet;

// Tests the generation of board moves based on giving a subset of squares
// TODO: There is still a bug lurking at deeper depths.
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
    // test_is_legal(Board::default()); // This is an empty board - will assert in debug, otherwise panice
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
