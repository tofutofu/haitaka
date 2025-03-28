pub use sparrow::*;

fn main() {
    println!("Hello, Shogi World!");

    let gs = board::GameStatus::Drawn;

    println!("{:#?}", gs);
    /*
    let a1 = Square::A1.bitboard();
    let b1 = Square::B1.bitboard();
    print!("{:#?}", a1);
    print!("{:#?}", b1);
    print!("{:#?}", a1 | b1);
    */

    /*
    println!("\nAbove rank G for Black");
    let mut bb = Rank::G.above(Color::Black);
    print!("{:#?}", bb);

    println!("\nBelow rank C for White");
    bb = Rank::C.below(Color::White);
    print!("{:#?}", bb);
    */

    //bb = BitBoard::FULL;
    //print!("{:#?}", bb >> 27);
    /*
    let bb = bitboard! {
            . . . . . . X X X
             . . . . . . X . X
             . . . . . . X X X
             . . . . . . . . .
             . . . . . . . . .
             . . . . . . . . .
             . . . . . . . . .
             . . . . . . . . .
             . . . . . . . . .
    };
    println!("{:#?}", bb);
    println!("{:#?}", bb.shl(2));
    */

    /*
    let bb1 = bitboard! {
         . . . . . . X X X
         . . . . . . X . X
         . . . . . . X X X
         . . . . . . . . .
         . . . . . . . . .
         . . . . . . . . .
         X X X . . . . . .
         X . X . . . . . .
         X X X . . . . . .
    };
    println!("{:#?}", bb1);
    println!("{:#?}", bb1 << 1);
    */

    /*
    let bb = attacks::knight(Color::Black, Square::C3);
    println!("{:#?}", bb);
    */

    // println!("{:#?}\n", bishop_pseudo_attacks(Square::C3));

    /*
    println!("{:?}\n", bitboard! {
        X . . . . . . . .
        . X . . . . . . .
        . . X . . . . . .
        . . . X . . . . .
        . . . . X . . . .
        . . . . . X . . .
        . . . . . . X . .
        . . . . . . . X .
        . . . . . . . . X
    }.0);
    */

    // println!("{:#?}", lance_pseudo_attacks(Color::White, Square::G8));

    let occ = bitboard! {
        . . . . . . . . .
        . . . . X . . X .
        . . X . . . . X .
        . . . . . . . . .
        X X . . X . . X .
        . . . . . . . . .
        . . . . . . X . .
        . X . . X . . . X
        . . . . . . . . X
    };
    // println!("{:#?}", occ);
    // println!("{:#?}", get_rook_moves(Square::H1, occ));

    /*
    let (nw, ne_rev, sw, se_rev) = BISHOP_RAY_MASKS[Square::C3 as usize];

    println!("NW:{:#?}", BitBoard(nw));
    println!("NE:{:#?}", BitBoard(ne_rev.reverse_bits()));
    println!("SW:{:#?}", BitBoard(sw));
    println!("SE:{:#?}", BitBoard(se_rev.reverse_bits()));
    */

    println!("{:#?}", occ);
    println!("{:#?}", get_bishop_moves(Square::E5, occ));

    //println!("{}", Square::G1 as usize);

    //let fwd_⁄_slash = "forward slash";
    //let bwd_∖_slash = "backward slash";
    //println!("{} and {}", fwd_⁄_slash, bwd_∖_slash);
}
