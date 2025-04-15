// Find magic numbers to set up slider move hash tables.
//
// Run with: cargo run --release --example find_magics
// Expected total run time is about 150s.
use std::env::args;
use std::time::Instant;

use haitaka_types::*;

// This file generates "magic numbers" for Rook and Bishop moves on a Shogi board.
// These numbers are used to create perfect hash tables for slider move generation. The code is
// adapted from analog-hors's magic bitboard demo (https://github.com/analog-hors/magic-bitboards-demo),
// with modifications for Shogi's larger board size (using u128 instead of u64 as backing for bitsets)
// and with some extra optimizations (that turned out to be really required).
//
// Three good overview of how "Magic Bitboards" work and how to find the "magic numbers" are:
// - [Chess programming: Magic Bitboards](https://www.chessprogramming.org/Magic_Bitboards)
// - [Magical Bitboards and How to Find Them](https://analog-hors.github.io/site/magic-bitboards/) by Analog-hors
// - [Magic Move-Bitboard Generation in Computer Chess](http://pradu.us/old/Nov27_2008/Buzz/research/magic/Bitboards.pdf)
// by Pradyumna Kannan

// Simple Pcg64Mcg impl (Mcg128Xsl64) - essentially the same as in the `rand` crate.
// See the `rand` documentation for background info.
pub struct Rng(u128);

impl Default for Rng {
    fn default() -> Self {
        // 0xcafe_f00d_d15e_a5e5 is the Rust `rand` default (and easter egg).
        //
        // It's important that the random seed is an odd number. It doesn't seem to
        // be very relevant otherwise what the value is (which makes sense).
        // Self(0x7369_7874_6565_6E20_6279_7465_206E_756Du128 | 1)
        // Self(0xc021_c6e55_f00d_1ab5 | 1)
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
struct MagicEntry {
    mask: u128,  // relevant blockers mask for slider on square
    magic: u64,  // multiplier for hash:  hash = (mask & occ) * multiplier
    shift: u8,   // bit shift for index: index = hash >> shift
    offset: u32, // offset in final table: table[offset + index] = moves
}

// The merge function needs to be identical to the one in haitaka_types (magics.rs).
// For an explanation of this function, see haitaka_types. Don't mess with this
// function, otherwise the search may be guaranteed to fail for all squares except
// the corners.
#[inline(always)]
const fn merge(x: u128) -> u64 {
    ((x >> 63) | x) as u64
}

// The get_magic_index function needs to closely match the one in haitaka_types.
//
// Note: On an empty board (blockers is empty), the hash will be 0 and the
// index will therefore also be 0, whatever the mask is. When the index is 0
// we always require the max. number of table slots to store all configs.
// So, the hash table segment for that square then degenerates into a normal
// lookup table in which each of the `2 ** magic.shift` blocker configs gets
// its own slot.

const fn get_magic_index(magic: &MagicEntry, blockers: &BitBoard) -> usize {
    let blockers = merge(blockers.0 & magic.mask);
    let hash = blockers.wrapping_mul(magic.magic);
    // magic.offset as usize +
    (hash >> magic.shift) as usize
}

fn find_all_magics(slider_name: &str, piece: Piece, rng: &mut Rng, verbose: bool) -> u32 {
    println!("const {slider_name}_MAGICS: &[MagicEntry; Square::NUM] = &[");
    let mut table_size = 0;
    for &square in &Square::ALL {
        let start = Instant::now();
        let (entry, num_slots, shared_slots) = find_magic(piece, square, table_size, rng, verbose);
        if verbose {
            println!(
                "    // {:?} slots={} shared={} time={:.2}s",
                square,
                num_slots,
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

// If all goes well, then  `find_magic` returns a suitable MagicEntry and the
// required number of slots to store all blocker configs. If we don't find a magic
// number in MAX_TRIALS trials, we return an invalid dummy entry, but also report
// and error.

const MAX_TRIALS: u32 = 20_000_000;
const RESET_SHIFT: u32 = MAX_TRIALS >> 1;

fn find_magic(
    piece: Piece,
    square: Square,
    offset: u32,
    rng: &mut Rng,
    verbose: bool,
) -> (MagicEntry, u32, u32) {
    // mask is set to the relevant blockers of the piece
    let mask: BitBoard = get_relevant_blockers(piece, square);
    assert!(!mask.has(square));

    // _max_ required number of bits to encode the table index
    let index_bits = mask.count_ones() as u8;
    assert!(index_bits == mask.len() as u8);
    assert!(0 < index_bits && index_bits <= 14);

    let mask: u128 = mask.0;

    // It's guaranteed that we can comfortably hash all blocker configurations
    // with `mask.count_ones()` bits, since that is the max. number of different
    // configurations for the given mask (the default blocker config).
    //
    // We can try to find a smaller table by setting `index_bits` lower.
    // This will "stress" the search, but a magic number might still be found.
    // I ran some quick-and-dirty experiments with this, but didn't find any.
    //
    // The _min_ number of required index bits equals the number of bits needed
    // to encode the number of different movesets associated with the mask.
    // The max number of possible moves (up to and including the first blocker
    // in any blocker configurations) is:
    // Rook: corner: 7 * 7 = 49
    //       edge:   7 * 6 = 42
    //       board:  3 * 3 * 3 * 3 = 81
    // Bishop: corner: 7
    //         middle: <= 81
    // So, to store all movesets the absolute minimum is 7 bits. But it's easy to
    // see that it's not possible to find a simple perfect hashing function (based
    // on a single magic multiplier) that achieves that minimum.

    // Trying to make the table more compact - didn't work - perhaps to be expected :/
    // let mut shift = if piece == Piece::Rook && !BitBoard::EDGES.has(square) { 64 - index_bits + 1 } else { 64 - index_bits};

    let mut shift = 64 - index_bits;

    // keeping some stats
    let mut num_trials: u32 = 0;
    let mut bad_magics: u32 = 0;

    loop {
        // Magics require a low number of active bits (approx. 6 to 9).
        let magic = rng.thin_random();

        // Brute-force trial and error has millions of misses!
        // The search time is significantly cut down by in advance weeding out
        // unpromising magics.
        //
        // The function `get_magic_index` will only look at the top `index_bits` number
        // of bits of the hash value. That is, only the top 12 to 14 bits (for the Rook)
        // or the top 7 to 12 bits (for the Bishop), since that is the minimum number of
        // bits required to encode all Rook or Bishop blocker configurations (the hash inputs).
        // This implies that generally we need about half of that number of bits to be set
        // in order to find a multiplier that works for all configs.

        let x = merge(mask).wrapping_mul(magic) & 0xFFFC_0000_0000_0000;
        if x.count_ones() < 6 as u32 {
            // bad magic
            bad_magics += 1;
            continue;
        }

        let magic_entry = MagicEntry {
            mask,
            magic,
            shift,
            offset,
        };
        if let Ok((num_slots, shared_slots)) = try_make_table(piece, square, &magic_entry) {
            return (magic_entry, num_slots, shared_slots);
        }

        if num_trials >= MAX_TRIALS {
            // Return a dummy result just to keep going with the rest.
            // This does mean the table needs to be redone! :/
            // I've not run into this situation when using the relax-shift trick (see below).
            println!("    // ERROR: No solution found for square {}", square);
            println!("    // Returning dummy magic");
            println!("    // num_trials={} bad_magics={}", num_trials, bad_magics);
            return (magic_entry, 0, 0);
        }

        if num_trials == RESET_SHIFT {
            // We're running into a wall with the current trial-and-error.
            // We could keep on trying, and eventually something might still be found,
            // but in early experiments (with unlimited number of trials) I could easily
            // run for an hour for particular squares without finding a solution. So, as
            // a fallback, I relax the shift _once_. This increases the table size but
            // since it only happens for 3 squares, it seems justified.
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
//
// We're actually trying to construct a locality sensitive perfect hash (LSH), where two
///configs are equivalent if they have the same associated moveset.
// A collision is only a relevant hash collision if the movesets are different.
// We want to find a perfect table, without any relevant collisions.
// The function fails if we cannot do so given the magic number.
//
// Note also that we always need to return the table.len(). We cannot take advantage of
// the fact that not all slots may be filled (which happens when we need to relax the
// shift amount and which could have if we'd have some shared slots), since the filled
// slots may not be a contiguous segment!
fn try_make_table(
    piece: Piece,
    square: Square,
    magic_entry: &MagicEntry,
) -> Result<(u32, u32), TableFillError> {
    // create a default table to hold up to 2 ** index_bits slots
    // in the degenerate case this will just store _all_ blocker configs directly
    let index_bits = 64 - magic_entry.shift;
    let mut table: Vec<BitBoard> = vec![BitBoard::EMPTY; 1 << index_bits];

    let mut num_slots: u32 = 0;
    let mut blockers = BitBoard::EMPTY;

    // stats

    // If we don't find shared_slots (and that seems to be the usual case!) then our
    // hash function could also very easily be constructed deterministically as
    // file_blockers << 7 | rank_blockers
    // (making sure file blockers and rank blockers are first limited to a bit length of 7).
    // Problem is that this construction seems more compute-intensive than the one in
    // `get_magic_index`, since we'd want to also only use max 12 bits for board squares,
    // rather than 14 bits for all squares...

    let mut shared_slots: u32 = 0;

    loop {
        let moves = get_moves(piece, square, blockers);

        let table_entry: &mut BitBoard = &mut table[get_magic_index(magic_entry, &blockers)];
        if table_entry.is_empty() {
            // Write to empty slot
            *table_entry = moves;
            num_slots += 1;
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
    Ok((table.len() as u32, shared_slots))
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
