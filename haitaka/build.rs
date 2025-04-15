// Build script for the large SLIDING_MOVES table
// Code adapted from the cozy-chess code.
//
// There is still a chicken-and-egg problem in building with special features,
// since build.rs doesn't know about those features at compile time. So
// to build with the `qugiy` feature, the feature also needs to be made known
// to the compiler, for instance by setting RUSTCFLAGS. So run with:
// ```
// RUSTFLAGS="--cfg feature=\"qugiy\"" cargo build -v -p haitaka --features qugiy
// ```

#![allow(unused_imports)]
use std::fs::File;
use std::io::BufWriter;
use std::io::prelude::*;
use std::path::PathBuf;

use haitaka_types::*;

// Stats for the currently generated SLIDING_MOVES table:
// - table len: 540416
// - total zeros: 64752
// - num of zero stretches: 9105
// - average length of zero stretches: 12

#[cfg(not(feature = "qugiy"))]
const GENERATED_FILE_NAME: &str = "sliding_moves_table.rs";

#[cfg(not(feature = "qugiy"))]
fn write_moves(
    table: &mut [u128],
    relevant_blockers: impl Fn(Square) -> BitBoard,
    table_index: impl Fn(Square, BitBoard) -> usize,
    slider_moves: impl Fn(Square, BitBoard) -> BitBoard,
) {
    let mut zeros: usize = 0;
    for &square in &Square::ALL {
        let mask = relevant_blockers(square);
        for blockers in mask.iter_subsets() {
            let moves: u128 = slider_moves(square, blockers).0;
            table[table_index(square, blockers)] = moves;
            if moves == 0 {
                zeros += 1;
            }
        }
    }
    assert!(table.len() != zeros, "write_moves only generated zeros!");
}

#[cfg(feature = "qugiy")]
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:warning=INFO: The 'qugiy' feature is active in build.rs.");
    return; // Exit early, do nothing
}

#[cfg(not(feature = "qugiy"))]
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut rook_table: Vec<u128> = vec![0u128; ROOK_TABLE_SIZE];
    let mut bishop_table: Vec<u128> = vec![0u128; BISHOP_TABLE_SIZE];

    write_moves(
        &mut rook_table,
        get_rook_relevant_blockers,
        get_rook_moves_index,
        get_rook_moves_slow,
    );

    write_moves(
        &mut bishop_table,
        get_bishop_relevant_blockers,
        get_bishop_moves_index,
        get_bishop_moves_slow,
    );

    let mut out_file: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    out_file.push(GENERATED_FILE_NAME);

    let mut out_file = BufWriter::new(File::create(out_file).unwrap());
    let mut num = 0;
    write!(
        &mut out_file,
        "const ROOK_MOVES: &[u128; {}] = &[",
        rook_table.len()
    )
    .unwrap();
    for &magic in &rook_table {
        write!(&mut out_file, "0x{:x},", magic).unwrap();
        num += 1;
        if num == 4 {
            writeln!(&mut out_file).unwrap();
            num = 0;
        }
    }
    writeln!(&mut out_file, "];").unwrap();

    num = 0;
    write!(
        &mut out_file,
        "const BISHOP_MOVES: &[u128; {}] = &[",
        bishop_table.len()
    )
    .unwrap();
    for magic in &bishop_table {
        write!(&mut out_file, "0x{:x},", magic).unwrap();
        num += 1;
        if num == 4 {
            writeln!(&mut out_file).unwrap();
            num = 0;
        }
    }
    writeln!(&mut out_file, "];").unwrap();
}
