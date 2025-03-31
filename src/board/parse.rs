// use core::convert::TryInto;
use core::fmt::{Display, Formatter};
use core::str::FromStr;

use super::ZobristBoard;
use crate::*;

helpers::simple_error! {
    /// An error while parsing the SFEN string.
    pub enum SFENParseError {
        InvalidBoard = "The board representation is invalid.",
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
    /// const STARTPOS: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1";
    /// let board = Board::from_sfen(STARTPOS).unwrap();
    /// assert_eq!(format!("{}", board), STARTPOS);
    /// ```
    pub fn from_sfen(sfen: &str) -> Result<Self, SFENParseError> {
        use SFENParseError::*;

        let mut board = Self {
            inner: ZobristBoard::empty(),
            pinned: BitBoard::EMPTY,
            checkers: BitBoard::EMPTY,
            move_number: 0,
        };

        let mut parts = sfen.split(' ');
        let mut next = || parts.next().ok_or(MissingField);

        Self::parse_board(&mut board, next()?).map_err(|_| InvalidBoard)?;
        Self::parse_side_to_move(&mut board, next()?).map_err(|_| InvalidSideToMove)?;
        Self::parse_hands(&mut board, next()?).map_err(|_| InvalidHands)?;

        // Parse the move number if it exists, otherwise set a default value
        if let Some(move_number_str) = parts.next() {
            Self::parse_move_number(&mut board, move_number_str).map_err(|_| InvalidMoveNumber)?;
        } else {
            // Default move number: 1 if Black to move, 2 if White to move
            board.move_number = if board.side_to_move() == Color::Black {
                1
            } else {
                2
            };
        }

        if parts.next().is_some() {
            return Err(TooManyFields);
        }

        if !board.move_number_is_valid() {
            return Err(InvalidMoveNumber);
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
        for (rank, row) in s.split('/').enumerate() {
            let rank = Rank::try_index(rank).ok_or(())?;
            let mut file = File::NUM;
            let mut prom: bool = false;

            for c in row.chars() {
                if let Some(offset) = c.to_digit(10) {
                    if prom {
                        return Err(());
                    };
                    file -= offset as usize; // let it panic
                } else if c == '+' {
                    if prom {
                        return Err(());
                    };
                    prom = true;
                } else if let Some((piece, color)) = Piece::try_from_char(c) {
                    file -= 1; // let it panic
                    let piece = piece.do_promote(prom);
                    let square = Square::new(File::try_index(file).ok_or(())?, rank);
                    board.inner.xor_square(piece, color, square);
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

    /// Parse the SFEN hands.
    fn parse_hands(board: &mut Board, s: &str) -> Result<(), ()> {
        let mut empty = false;
        let mut found: bool = false;
        let mut count: u32 = 0;

        for c in s.chars() {
            if !empty {
                if c == '-' {
                    empty = true;
                } else if let Some(num) = c.to_digit(10) {
                    count = 10 * count + num;
                } else if let Some((piece, color)) = Piece::try_from_char(c) {
                    board.unchecked_set_hand(color, piece, if count > 0 { count } else { 1 });
                    count = 0;
                    found = true;
                } else {
                    return Err(());
                }
            } else {
                // we read another '-'
                return Err(());
            }
        }

        if empty == found {
            // both are false should not be possible, given non-empty input string;
            // both true, implies an ill-formatted input string (containing pieces and '-')
            return Err(());
        }
        if count > 0 {
            // we read a dangling number without associated piece
            return Err(());
        }

        Ok(())
    }

    fn parse_side_to_move(board: &mut Board, s: &str) -> Result<(), ()> {
        let stm: Color = s.parse().map_err(|_| ())?;
        if stm != board.side_to_move() {
            board.inner.toggle_side_to_move();
        }
        Ok(())
    }

    fn parse_move_number(board: &mut Board, s: &str) -> Result<(), ()> {
        // TODO: make optional
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
    /// const STARTPOS: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1";
    /// let board: Board = STARTPOS.parse().unwrap();
    /// assert_eq!(format!("{}", board), STARTPOS);
    /// ```
    fn from_str(sfen: &str) -> Result<Self, Self::Err> {
        match Self::from_sfen(sfen) {
            Ok(board) => Ok(board),
            Err(error) => Err(error),
        }
    }
}

impl Display for Board {
    /// Display the board.
    ///
    /// # Examples
    /// ```
    /// # use sparrow::*;
    /// let mut board: Board = SFEN_6PIECE_HANDICAP.parse().unwrap();
    /// assert_eq!(format!("{}", board), SFEN_6PIECE_HANDICAP);
    /// board = SFEN_4PIECE_HANDICAP.parse().unwrap();
    /// assert_eq!(format!("{}", board), SFEN_4PIECE_HANDICAP);
    /// board = SFEN_2PIECE_HANDICAP.parse().unwrap();
    /// assert_eq!(format!("{}", board), SFEN_2PIECE_HANDICAP);
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // BOARD
        for &rank in Rank::ALL.iter() {
            let mut empty = 0;
            for &file in File::ALL.iter().rev() {
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
            if (rank as usize) < 8 {
                write!(f, "/")?;
            }
        }

        // STM
        write!(f, " {}", self.side_to_move())?;

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

        // MOVE_NUMBER
        write!(f, " {}", self.move_number)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_valid_sfens() {
        for sfen in include_str!("test_data/valid.sfens").lines() {
            let board = Board::from_sfen(sfen).unwrap();
            assert!(board.validity_check());
        }
    }

    #[test]
    fn handles_invalid_sfens() {
        for sfen in include_str!("test_data/invalid.sfens").lines() {
            assert!(
                Board::from_sfen(sfen).is_err(),
                "FEN \"{}\" should not parse",
                sfen
            );
        }
    }
}
