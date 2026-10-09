/*
PURPOSE
    This is the translator between the player and the game. This file
    has two jobs:

    (1) Convert good input, by trimming spaces and uppercasing,
        splitting the letter from the number, turning both into
        numbers, and returning a Coord.
    (2) Reject bad input clearly, by returning an error with a message
        telling the player what went wrong, if the input is empty, has
        a bad letter/number, points off board, etc.

KEY TERMS & DEFINITIONS

Coord

INPUTS

OUTPUTS

Work Notes
Field names: row and col DONE
Indexing: 0 based, board is a 10x10 array
Type: usize DONE
Mapping: Letter -> Column, Number -> Row
Derives: Debug, Clone. Copy, PartialEq DONE
*/

// Formatting tool
use std::fmt;

// Debug and PartialEq for printing and tests; Clone and Copy so board.rs
// can reuse a Coord without it being moved
#[derive(Debug, Clone, Copy, PartialEq)]

// A board square, 0-indexed for array use: "A1" is (0, 0), "J10" is (9, 9)
pub struct Coord {
    pub row: usize,
    pub col: usize,
}

//Enum for error choices, empty input, bad letter, bad number, and number being off the board
#[derive(Debug, PartialEq)]
pub enum ParseError {
    EmptyInput,
    InvalidLetter,
    InvalidNumber,
    NumberOutOfRange,
}

// Display for ParseError
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "You fired at nothing. Please enter a square, such as B6"),
            ParseError::InvalidLetter => write!(f, "That column is in another ocean. Use a letter from A to J"),
            ParseError::InvalidNumber => write!(f, "Rows are numbers, captain. Try something like B6"),
            ParseError::NumberOutOfRange => write!(f, "That is off the map. Rows span from 1 to 10"),
        }
    }
}

pub fn parse_move(input: &str) -> Result<Coord, ParseError> {
    todo!()
}