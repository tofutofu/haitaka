// Find black magic numbers to set up slider move hash tables.
//
// Run with: cargo run --release --example find_black_magics
use std::env::args;
use std::ops::Not;
use std::time::Instant;

use haitaka_types::*;

pub struct Rng(u128);

impl Default for Rng {
    fn default() -> Self {
        Self(0xcafe_f00d_d15e_a5e5 | 1)
    }
}

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(0x2360_ED05_1FC6_5DA4_4385_DF64_9FCC_F645);
        let rot = (self.0 >> 122) as u32;
        let xsl = ((self.0 >> 64) as u64 ^ self.0 as u64).rotate_right(rot);

        xsl
    }

    // random a pseudo-random u64 with approx. 8 bits set
    pub fn thin_random(&mut self) -> u64 {
        self.next_u64() & self.next_u64() & self.next_u64()
    }
}

fn get_relevant_blockers(piece: Piece, square: Square) -> BitBoard {
    match piece {
        Piece::Rook => get_rook_relevant_blockers(square),
        Piece::Bishop => get_bishop_relevant_blockers(square),
        _ => panic!("Expected a Rook or Bishop"),
    }
}

fn get_moves(piece: Piece, square: Square, blockers: BitBoard) -> BitBoard {
    match piece {
        Piece::Rook => get_rook_moves_slow(square, blockers),
        Piece::Bishop => get_bishop_moves_slow(square, blockers),
        _ => panic!("Expected a Rook or Bishop"),
    }
}

// MagicEntry needs to match what is in haitaka_types.
// We duplicate the code since it's not exported by haitaka_types.
#[allow(dead_code)]
struct MagicEntry {
    neg_mask: u128,
    mask: u128,  // relevant blockers mask for slider on square
    magic: u64,  // multiplier for hash:  hash = (mask & occ) * multiplier
    shift: u8,   // bit shift for index: index = hash >> shift
    offset: u32, // offset in final table: table[offset + index] = moves
}

// The merge function needs to be identical to the one in haitaka_types (magics.rs).
#[inline(always)]
const fn merge(x: u128) -> u64 {
    ((x >> 63) | x) as u64
}

// The get_magic_index function needs to closely match the one in haitaka_types.
const fn get_magic_index(magic: &MagicEntry, blockers: &BitBoard) -> usize {
    //let blockers = merge(blockers.0 & magic.mask);
    // let hash = blockers.wrapping_mul(magic.magic);

    let blockers: u128 = blockers.0 | magic.neg_mask;
    let hash: u128 = blockers.wrapping_mul(magic.magic as u128);
    // magic.offset as usize +
    (hash >> (64 + magic.shift)) as usize
}

fn find_all_magics(slider_name: &str, piece: Piece, rng: &mut Rng, verbose: bool) -> u32 {
    println!("const {slider_name}_MAGICS: &[MagicEntry; Square::NUM] = &[");
    let mut table_size = 0;
    for &square in &Square::ALL {
        let start = Instant::now();
        let (entry, num_slots, shared_slots, max_index) = find_magic(piece, square, table_size, rng, verbose);
        if verbose {
            println!(
                "    // {:?} slots={} max_index={} shared={} time={:.2}s",
                square,
                num_slots,
                max_index,
                shared_slots,
                start.elapsed().as_secs_f64()
            );
        }
        println!(
            "    MagicEntry {{ mask: 0x{:016X}, magic: 0x{:016X}, shift: {}, offset: {} }},",
            entry.mask, entry.magic, entry.shift, entry.offset
        );
        table_size += num_slots;
    }
    println!("];");
    println!(
        "pub const {}_TABLE_SIZE: usize = {};",
        slider_name, table_size
    );

    table_size
}

const MAX_TRIALS: u32 = 20_000_000;
const RESET_SHIFT: u32 = MAX_TRIALS >> 1;

fn find_magic(
    piece: Piece,
    square: Square,
    offset: u32,
    rng: &mut Rng,
    verbose: bool,
) -> (MagicEntry, u32, u32, usize) {
    // mask is set to the relevant blockers of the piece
    let mask: BitBoard = get_relevant_blockers(piece, square);
    assert!(!mask.has(square));

    // _max_ required number of bits to encode the table index
    let index_bits = mask.count_ones() as u8;
    assert!(index_bits == mask.len() as u8);
    assert!(0 < index_bits && index_bits <= 14);

    let mask: u128 = mask.0;
    let neg_mask: u128 = mask.not() & BitBoard::FULL.0;
    let mut shift = 64 - index_bits;

    // keeping some stats
    let mut num_trials: u32 = 0;
    let mut bad_magics: u32 = 0;

    loop {
        // Magics require a low number of active bits (approx. 6 to 9).
        let magic = rng.thin_random();
        
        // assert!(merge(mask) != 0);
        let x = merge(mask).wrapping_mul(magic) & 0xFFFC_0000_0000_0000;
        if x.count_ones() < 6 as u32 {
            // bad magic
            bad_magics += 1;
            //if verbose && bad_magics % 10_000 == 0 {
            //    println!("    // num_trials={} bad_magics={}", num_trials, bad_magics);    
            //}
            continue;
        }

        let magic_entry = MagicEntry {
            neg_mask,
            mask,
            magic,
            shift,
            offset,
        };
        if let Ok((num_slots, shared_slots, max_index)) = try_make_table(piece, square, &magic_entry) {
            return (magic_entry, num_slots, shared_slots, max_index);
        }

        if num_trials >= MAX_TRIALS {
            // Return a dummy result just to keep going with the rest.
            println!("    // ERROR: No solution found for square {}", square);
            println!("    // Returning dummy magic");
            println!("    // num_trials={} bad_magics={}", num_trials, bad_magics);
            return (magic_entry, 0, 0, 0);
        }

        if num_trials == RESET_SHIFT {
            shift -= 1;
            if verbose {
                println!(
                    "    // Square {} is problematic. Relaxing shift to {}",
                    square, shift
                );
                println!("    // num_trials={} bad_magics={}", num_trials, bad_magics);
            }
        } 
        num_trials += 1;
    }
}

struct TableFillError;

// Try to store all blocker configs in a hash table, given a single magic multiplier.
// Keys to the table are the blocker configs, values are the associated movesets.

fn try_make_table(
    piece: Piece,
    square: Square,
    magic_entry: &MagicEntry,
) -> Result<(u32, u32, usize), TableFillError> {
    // create a default table to hold up to 2 ** index_bits slots
    // in the degenerate case this will just store _all_ blocker configs directly
    let index_bits = 64 - magic_entry.shift;
    let mut table: Vec<BitBoard> = vec![BitBoard::EMPTY; 1 << index_bits];

    let mut num_slots: u32 = 0;
    let mut blockers = BitBoard::EMPTY;

    // stats
    let mut shared_slots: u32 = 0;
    let mut max_index: usize = 0;

    loop {
        let moves = get_moves(piece, square, blockers);

        let index: usize = get_magic_index(magic_entry, &blockers);
        let table_entry: &mut BitBoard = &mut table[index];
        if table_entry.is_empty() {
            // Write to empty slot
            *table_entry = moves;
            num_slots += 1;
            max_index = max_index.max(index);
        } else if *table_entry != moves {
            // Hash collisions are unacceptable, we want a perfect hash table.
            return Err(TableFillError);
        } else {
            // So far, I didn't see any :/
            shared_slots += 1;
        }

        // Carry-Rippler trick to enumerate all subsets of the mask, getting us all blockers.
        // This enumerates them in ordinal order (from small numbers until equal to mask).
        // In other words, the blockers with the smallest footprint (smallest MSB bit) are
        // generated first.
        // https://www.chessprogramming.org/Traversing_Subsets_of_a_Set#All_Subsets_of_any_Set
        blockers.0 = blockers.0.wrapping_sub(magic_entry.mask) & magic_entry.mask;
        if blockers.is_empty() {
            // We came full circle
            break;
        }
    }
    assert!(num_slots > 0);
    Ok((table.len() as u32, shared_slots, max_index))
}

fn main() {
    let mut verbose = false;
    for arg in args().skip(1) {
        if arg == "--verbose" || arg == "-v" {
            verbose = true;
            continue;
        }
        if arg == "--help" || arg == "-h" {
            println!("Run: cargo run --release --example find_magics [-- --verbose]");
            return;
        }
        eprintln!("ERROR: Unexpected argument '{}'.", arg);
        eprintln!("Run: cargo run --release --example find_magics [-- --verbose]");
        return;
    }

    let mut rng = Rng::default();

    let start = Instant::now();
    let rook_table_size = find_all_magics("ROOK", Piece::Rook, &mut rng, verbose);
    let rook_time = start.elapsed();

    let start = Instant::now();
    let bishop_table_size = find_all_magics("BISHOP", Piece::Bishop, &mut rng, verbose);
    let bishop_time = start.elapsed();

    println!(
        "pub const SLIDING_MOVES_TABLE_SIZE: usize = {};",
        rook_table_size + bishop_table_size
    );

    println!(
        "// Total time to generate Rook magics:   {}s",
        rook_time.as_secs()
    );
    println!(
        "// Total time to generate Bishop magics: {}s",
        bishop_time.as_secs()
    );
    println!(
        "// Total time to generate all magics:    {}s",
        (rook_time + bishop_time).as_secs()
    );
}
