// Benchmark for Play Moves and Generate Moves
// Essentially copied from `cozy-chess` (apart from the positions of course)

use std::time::Duration;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use sparrow::Board;

const POSITIONS: &[&str] = &[
    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1",
    "ln1g5/1r2S1k2/p2pppn2/2ps2p2/1p7/2P6/PPSPPPPLP/2G2K1pr/LN4G1b w BGSLPnp 62",
    "ln1gk1snl/1r5b1/p1ppppgpp/1s4p2/1p7/P1P3R2/1P1PPPP1P/1BG3S2/LNS1KG1NL b P",
];

fn perft(board: &Board, depth: u8) -> u32 {
    if depth == 0 {
        1
    } else {
        let mut nodes = 0;
        board.generate_moves(|moves| {
            for mv in moves {
                let mut board = board.clone();
                board.play_unchecked(mv);
                nodes += perft(&board, depth - 1);
            }
            false
        });
        nodes
    }
}

pub fn criterion_benchmark(criterion: &mut Criterion) {
    let startpos = Board::startpos();

    let positions = POSITIONS
        .iter()
        .map(|pos| {
            let board: Board = pos.parse().unwrap();
            let mut all_moves = Vec::new();
            board.generate_moves(|moves| {
                all_moves.extend(moves);
                false
            });
            (board, all_moves)
        })
        .collect::<Vec<_>>();

    criterion
        .bench_function("Play moves", |b| {
            b.iter(|| {
                for (board, moves) in &positions {
                    for &mv in moves {
                        let mut board = board.clone();
                        board.play_unchecked(mv);
                        black_box(board);
                    }
                }
            });
        })
        .bench_function("Generate moves", |b| {
            b.iter(|| {
                for (board, _) in &positions {
                    board.generate_moves(|moves| {
                        for mv in moves {
                            black_box(mv);
                        }
                        false
                    });
                }
            });
        })
        .bench_function("Startpos perft 3", |b| {
            b.iter(|| {
                let pos = black_box(&startpos);
                let depth = black_box(3);
                black_box(perft(pos, depth));
            });
        });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(300).measurement_time(Duration::from_secs(30));
    targets = criterion_benchmark
}
criterion_main!(benches);
