use crate::*;

///
/// Basic attack vectors on an empty board
///

// TODO: Move most into a macro?

pub const fn king(square: Square) -> BitBoard {
    const TABLE: [BitBoard; Square::NUM] = {
        let src = Square::B2;
        let pattern = bitboard! {
            . . . . . . X X X
            . . . . . . X * X
            . . . . . . X X X
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
        };        
        let mut table = [BitBoard::EMPTY; Square::NUM];
        let mut sq: usize = 0;
        while sq < Square::NUM {
            table[sq] = pattern.shift(src, Square::index_const(sq));
            sq += 1;
        }
        table
    };

    TABLE[square as usize]
}

pub const fn gold(color: Color, square: Square) -> BitBoard {
    const TABLE: [[BitBoard; Square::NUM]; Color::NUM] = {
        let src = Square::B2;
        let bpattern = bitboard! {
            . . . . . . X X X
            . . . . . . X * X
            . . . . . . . X .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
        };    
        let wpattern = bitboard! {
            . . . . . . . X .
            . . . . . . X * X
            . . . . . . X X X
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
        };    
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Color::NUM];
        let mut sq: usize = 0;
        while sq < Square::NUM {
            table[Color::White as usize][sq] = wpattern.shift(src, Square::index_const(sq));
            table[Color::Black as usize][sq] = bpattern.shift(src, Square::index_const(sq));
            sq += 1;
        }
        table
    };

    TABLE[color as usize][square as usize]
}

pub const fn silver(color: Color, square: Square) -> BitBoard {
    const TABLE: [[BitBoard; Square::NUM]; Color::NUM] = {
        let src = Square::B2;
        let bpattern = bitboard! {
            . . . . . . X X X
            . . . . . . . * .
            . . . . . . X . X
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
        };    
        let wpattern = bitboard! {
            . . . . . . X . X
            . . . . . . . * .
            . . . . . . X X X
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
        };    
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Color::NUM];
        let mut sq: usize = 0;
        while sq < Square::NUM {
            table[Color::White as usize][sq] = wpattern.shift(src, Square::index_const(sq));
            table[Color::Black as usize][sq] = bpattern.shift(src, Square::index_const(sq));
            sq += 1;
        }
        table
    };

    TABLE[color as usize][square as usize]
}

pub const fn knight(color: Color, square: Square) -> BitBoard {
    const TABLE: [[BitBoard; Square::NUM]; Color::NUM] = {
        let src = Square::B3;
        let bpattern = bitboard! {
            . . . . . . X . X
            . . . . . . . . .
            . . . . . . . * .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
        };    
        let wpattern = bitboard! {
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . * .
            . . . . . . . . .
            . . . . . . X . X
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
        };    
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Color::NUM];
        let mut sq: usize = 0;
        while sq < Square::NUM {
            table[Color::White as usize][sq] = wpattern.shift(src, Square::index_const(sq));
            table[Color::Black as usize][sq] = bpattern.shift(src, Square::index_const(sq));
            sq += 1;
        }
        table
    };

    TABLE[color as usize][square as usize]
}

pub const fn pawn(color: Color, square: Square) -> BitBoard {
    const TABLE: [[BitBoard; Square::NUM]; Color::NUM] = {
        let src = Square::B2;
        let bpattern = bitboard! {
            . . . . . . . . X
            . . . . . . . . *
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
        };    
        let wpattern = bitboard! {
            . . . . . . . . .
            . . . . . . . . *
            . . . . . . . . X
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
        };    
        let mut table = [[BitBoard::EMPTY; Square::NUM]; Color::NUM];
        let mut sq: usize = 0;
        while sq < Square::NUM {
            table[Color::White as usize][sq] = wpattern.shift(src, Square::index_const(sq));
            table[Color::Black as usize][sq] = bpattern.shift(src, Square::index_const(sq));
            sq += 1;
        }
        table
    };

    TABLE[color as usize][square as usize]
}

/* 
pub const fn lance(color: Color, square: Square) -> BitBoard {

}

pub const fn rook(square: Square) -> BitBoard {
    todo!()
}

pub const fn bishop(square: Square) -> BitBoard {
    todo!()
}
*/

