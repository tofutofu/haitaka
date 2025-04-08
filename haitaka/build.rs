// Build script for the large SLIDING_MOVES table
// Code adapted from the cozy-chess code.
use std::path::PathBuf;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use haitaka_types::*;

// Stats for the currently generated SLIDING_MOVES table:
// - table len: 540416
// - total zeros: 64752
// - num of zero stretches: 9105  
// - average length of zero stretches: 12

const GENERATED_FILE_NAME: &str = "sliding_moves_table.rs";

fn write_moves(
    table: &mut [BitBoard],
    relevant_blockers: impl Fn(Square) -> BitBoard,
    table_index: impl Fn(Square, BitBoard) -> usize,
    slider_moves: impl Fn(Square, BitBoard) -> BitBoard
) {
    for &square in &Square::ALL {
        let mask = relevant_blockers(square);
        for blockers in mask.iter_subsets() {
            table[table_index(square, blockers)] = slider_moves(square, blockers);
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut table: Vec<BitBoard> = vec![BitBoard::EMPTY; SLIDING_MOVE_TABLE_SIZE];
    write_moves(
        &mut table,
        get_rook_relevant_blockers,
        get_rook_moves_index,
        get_rook_moves_slow
    );
    write_moves(
        &mut table,
        get_bishop_relevant_blockers,
        get_bishop_moves_index,
        get_bishop_moves_slow
    );

    let mut out_file: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    out_file.push(GENERATED_FILE_NAME);

    let mut out_file = BufWriter::new(File::create(out_file).unwrap());
    write!(&mut out_file, "const SLIDING_MOVES: &[u128; {}] = &[", table.len()).unwrap();
    for magic in &table {
        write!(&mut out_file, "0x{:x},", magic.0).unwrap();
    }
    write!(&mut out_file, "];").unwrap();
}

