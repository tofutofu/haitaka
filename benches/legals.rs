use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::time::Duration;

use sparrow::{
    //get_lance_moves, get_rook_moves, get_bishop_moves,
    //BitBoard,
    Board,
    Move,
    Piece,
    bishop_pseudo_attacks,
    //silver_attacks,
    gold_attacks,
    king_attacks,
    knight_attacks,
    lance_pseudo_attacks,
    pawn_attacks,
    rook_pseudo_attacks,
};

// Input positions for testing
const POSITIONS: &[&str] = &[
    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1",
    "ln1g5/1r2S1k2/p2pppn2/2ps2p2/1p7/2P6/PPSPPPPLP/2G2K1pr/LN4G1b w BGSLPnp 62",
    "ln1gk1snl/1r5b1/p1ppppgpp/1s4p2/1p7/P1P3R2/1P1PPPP1P/1BG3S2/LNS1KG1NL b P",
];

// Testing the legality of BoardMoves. This is a speed and stress test for `Board::is_legal`
macro_rules! bench_piece {
    ($criterion:expr, $positions:expr, $piece:expr, $move_gen:expr, $name:expr) => {
        let to_check: Vec<_> = $positions
            .iter()
            .flat_map(move |board| {
                (board.pieces($piece) & board.colors(board.side_to_move()))
                    .iter()
                    .flat_map(move |from| {
                        $move_gen(board.side_to_move(), from).iter().map(move |to| {
                            (
                                board,
                                Move::BoardMove {
                                    from,
                                    to,
                                    promotion: false,
                                },
                            )
                        })
                    })
            })
            .collect();

        $criterion
            .benchmark_group("legality")
            .throughput(Throughput::Elements(to_check.len() as u64))
            .bench_function($name, |b| {
                b.iter(|| {
                    for &(ref board, mv) in &to_check {
                        black_box(board.is_legal(mv));
                    }
                })
            });
    };
}

pub fn criterion_benchmark(criterion: &mut Criterion) {
    let positions: Vec<Board> = POSITIONS.iter().map(|pos| pos.parse().unwrap()).collect();

    bench_piece!(criterion, positions, Piece::Pawn, pawn_attacks, "pawns");
    bench_piece!(
        criterion,
        positions,
        Piece::Knight,
        knight_attacks,
        "knights"
    );
    bench_piece!(criterion, positions, Piece::Knight, gold_attacks, "golds");
    bench_piece!(
        criterion,
        positions,
        Piece::Lance,
        lance_pseudo_attacks,
        "lances"
    );
    bench_piece!(
        criterion,
        positions,
        Piece::Rook,
        |_, from| { rook_pseudo_attacks(from) },
        "rooks"
    );
    bench_piece!(
        criterion,
        positions,
        Piece::Bishop,
        |_, from| { bishop_pseudo_attacks(from) },
        "bishops"
    );
    bench_piece!(criterion, positions, Piece::King, king_attacks, "kings");
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(300).measurement_time(Duration::from_secs(30));
    targets = criterion_benchmark
}
criterion_main!(benches);
