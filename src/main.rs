pub use sparrow::*;

fn main() {
    println!("Hello, Shogi World!");

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

    println!("{:#?}", lance_pseudo_attacks(Color::White, Square::G8));

}
