use std::time::Duration;

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use sparrow::*;

fn bench(criterion: &mut Criterion, id: &str, elem: usize, mut routine: impl FnMut()) {
    criterion
        .benchmark_group("slider_moves")
        .throughput(Throughput::Elements(elem as u64))
        .bench_function(id, |b| b.iter(&mut routine));
}

pub fn criterion_benchmark(criterion: &mut Criterion) {
    // Simple Pcg64Mcg random number generator - Copied from cozy-chess.
    // We don't need any strong randomness. We do always want to use the
    // same random seed.

    let mut state = 0x6D696E75736B656C76696E2062616974u128 | 1;
    let mut rand = || {
        state = state.wrapping_mul(0x2360ED051FC65DA44385DF649FCCF645);
        let rot = (state >> 122) as u32;
        let xsl = (state >> 64) as u64 ^ state as u64;
        xsl.rotate_right(rot) as u128
    };

    // x will have a bitcount of approx. 64 bits, randomly distributed.
    // BitBoard::new will effectively ignore the top 47 bits, and only keep
    // the lower 81 bits needed for the 9x9 Shogi board.
    // So the board will have approx. 40 bits set, close to the total number
    // of Shogi pieces, so no need to thin out the bitcount further by xor-ing
    // the random numbers.
    // let _rand128: impl Fn() -> u128 = || {
    //    (rand() << 64) | rand()
    // };

    let blockers = (0..1000)
        .map(|_| BitBoard::new(rand() ^ rand() ^ rand()))
        .collect::<Vec<_>>();

    bench(
        criterion,
        "get_rook_moves",
        Square::NUM * blockers.len(),
        || {
            for &square in black_box(&Square::ALL) {
                for &blockers in black_box(&blockers) {
                    black_box(get_rook_moves(Color::Black, square, blockers));
                }
            }
        },
    );

    bench(
        criterion,
        "get_bishop_moves",
        Square::NUM * blockers.len(),
        || {
            for &square in black_box(&Square::ALL) {
                for &blockers in black_box(&blockers) {
                    black_box(get_bishop_moves(Color::Black, square, blockers));
                }
            }
        },
    );

    bench(
        criterion,
        "get_lance_moves",
        Color::NUM * Square::NUM * blockers.len(),
        || {
            for &color in black_box(&Color::ALL) {
                for &square in black_box(&Square::ALL) {
                    for &blockers in black_box(&blockers) {
                        black_box(get_lance_moves(color, square, blockers));
                    }
                }
            }
        },
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(300).measurement_time(Duration::from_secs(30));
    targets = criterion_benchmark
}
criterion_main!(benches);
