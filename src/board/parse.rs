use core::convert::TryInto;
use core::str::FromStr;
use core::fmt::{Display, Formatter};

use crate::*;
use super::ZobristBoard;

helpers::simple_error! {
    /// An error while parsing the SFEN string.
    pub enum SFENParseError {
        InvalidBoard = "The board is invalid.",
        InvalidHands = "The hands representation is invalid",
        InvalidSideToMove = "The side to move is invalid.",
        InvalidMoveNumber = "The move number is invalid.",
        MissingField = "The SFEN string is missing a field.",
        TooManyFields = "The SFEN string has too many fields."
    }
}

impl Board {
    /// Parse a SFEN string. You can also parse the board with [`FromStr`].
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// const STARTPOS: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPP/1B5R1/LNSGKGSNL w - 1";
    /// let board = Board::from_sfen(STARTPOS, false).unwrap();
    /// assert_eq!(format!("{}", board), STARTPOS);
    /// ```
    pub fn from_sfen(sfen: &str) -> Result<Self, SFENParseError> {
        use SFENParseError::*;

        let mut board = Self {
            inner: ZobristBoard::empty(),
            pinned: BitBoard::EMPTY,
            checkers: BitBoard::EMPTY,
            move_number: 0
        };

        let mut parts = sfen.split(' ');
        let mut next = || parts.next().ok_or(MissingField);
        
        Self::parse_board(&mut board, next()?)
            .map_err(|_| InvalidBoard)?;

        Self::parse_hands(&mut board, next()?)
            .map_err(|_| InvalidHands)?;

        Self::parse_side_to_move(&mut board, next()?)
            .map_err(|_| InvalidSideToMove)?;

        Self::parse_move_number(&mut board, next()?)
            .map_err(|_| InvalidMoveNumber)?;
        if !board.move_number_is_valid() {
            return Err(InvalidMoveNumber);
        }

        if parts.next().is_some() {
            return Err(TooManyFields);
        }

        if !board.is_valid() {
            return Err(InvalidBoard);
        }

        let (checkers, pinned) = board.calculate_checkers_and_pins(board.side_to_move());
        board.checkers = checkers;
        board.pinned = pinned;
        if !board.checkers_and_pins_are_valid() {
            return Err(InvalidBoard);
        }

        Ok(board)
    }

    /// Parse the board representation of a SFEN string.
    fn parse_board(board: &mut Board, s: &str) -> Result<(), ()> {
        for (rank, row) in s.rsplit('/').enumerate() {
            let rank = Rank::try_index(rank).ok_or(())?;
            let mut file = File::NUM;
            let mut prom: bool = false;

            for c in row.chars() {
                if let Some(offset) = c.to_digit(10) {
                    if prom {
                        return Err(());
                    };
                    file -= offset as usize;
                } else if c == '+' {
                    if prom { 
                        return Err(());
                    };
                    prom = true;                    
                } else if let Some((piece, color)) = Piece::try_from_char(c) {
                    let piece = piece.do_promote(prom);
                    let square = Square::new(
                        File::try_index(file).ok_or(())?,
                        rank
                    );
                    board.inner.xor_square(piece, color, square);

                    file -= 1;
                    prom = false;                   
                } else {
                    return Err(());
                }
            }
            if file != 0 {
                return Err(());
            }
        }
        Ok(())
    }

    fn parse_hands(board: &mut Board, s: &str) -> Result<(), ()> {
        let mut empty = false;
        let mut count: u16 = 0;

        for c in s.chars() {
            if !empty {
                if c == '-' {
                    empty = true;
                } else if let Some(num) = c.to_digit(10) {
                    count = 10 * count + num;
                } else if let Some((piece, color)) = Piece::try_from_char(c) {

                    // TODO: This needs to be done by function call
                    // since the hash also needs to be updated!

                    board.inner.hands[color as usize][piece as usize] = max(count, 1);
                    count = 0
                } else {
                    return Err(());
                }
            } else {
                return Err(());
            }
        }
        if count > 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    fn parse_side_to_move(board: &mut Board, s: &str) -> Result<(), ()> {
        if s.parse::<Color>().map_err(|_| ())? != board.side_to_move() {
            board.inner.toggle_side_to_move();
        }
        Ok(())
    }

    fn parse_move_number(board: &mut Board, s: &str) -> Result<(), ()> {
        board.move_number = s.parse().map_err(|_| ())?;
        if board.move_number == 0 {
            return Err(());
        }
        Ok(())
    }
}

impl FromStr for Board {
    type Err = SFENParseError;

    /// Parse a SFEN string.
    /// 
    /// See also: [`Board::from_sfen`].
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// const STARTPOS: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPP/1B5R1/LNSGKGSNL w - 1";
    /// let board: Board = STARTPOS.parse().unwrap();
    /// assert_eq!(format!("{}", board), STARTPOS);
    /// ```
    fn from_str(sfen: &str) -> Result<Self, Self::Err> {
        match Self::from_sfen(sfen) {
            Ok(board) => Ok(board),
            Err(error) => Err(error)
        }
    }
}

impl Display for Board {
    /// Display the board. 
    /// 
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// const STARTPOS: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPP/1B5R1/LNSGKGSNL w - 1";
    /// let board = Board::default();
    /// assert_eq!(format!("{}", board), STARTPOS);
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // BOARD
        for &file in File::ALL.iter().rev() { // 9 .. 1
            let mut empty = 0;
            for &rank in Rank::ALL.iter() { // a .. i
                let square = Square::new(file, rank);
                if let Some(piece) = self.colored_piece_on(square) {
                    if empty > 0 {
                        write!(f, "{}", empty)?;
                        empty = 0;
                    }
                    write!(f, "{}", piece)?;
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                write!(f, "{}", empty)?;
            }
            if file < File::One {
                write!(f, "/")?;
            }
        }
        
        // HANDS
        if self.is_hand_empty(Color::White) && self.is_hand_empty(Color::Black) {
            write!(f, " -")?;
        } else {
            write!(f, " ")?;
            for &color in Color::ALL.iter().rev() {
                // TODO: Probably need to change the order of pieces in hand
                for (piece, &count) in self.hand(color).iter().enumerate() {
                    if count > 0 {
                        if count > 1 {
                            write!(f, "{}{}", count, piece)?;
                        } else {
                            write!(f, "{}", piece)?;
                        }
                    }
                }
            }
        }

        // STM
        write!(f, " {}", self.side_to_move())?;

        // MOVE_NUMBER
        write!(f, " {}", self.move_number)?;
        
        Ok(())
    }
}

/* 
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_valid_fens() {
        for fen in include_str!("test_data/valid.sfens").lines() {
            let board = Board::from_fen(fen, true).unwrap();
            assert!(board.validity_check());
        }
    }

    #[test]
    fn handles_invalid_fens() {
        for fen in include_str!("test_data/invalid.sfens").lines() {
            assert!(Board::from_fen(fen, true).is_err(), "FEN \"{}\" should not parse", fen);
        }
    }

    #[test]
    fn invalid_ep_fen() {
        let fen = "4k3/8/5N2/8/2pP4/8/8/4K3 b - d3 0 1";
        assert!(matches!(fen.parse::<Board>(), Err(FenParseError::InvalidEnPassant)));
    }
}
*/