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

Coord - a square on the board as row and col numbers. Both start at 0, so "A1" is
     row 0, col 0 and "J10" is row 9, col 9. The letter picks the column, the number picks the row.

     ParseError - the enum for the ways input can go wrong: EmptyInput, InvalidLetter,
     InvalidNumber, and NumberOutOfRange. Display gives each one a message for the player.

     Result - Rust's replacement for exceptions. parse_move gives back Ok(Coord) when the
     input is good, or Err(ParseError) when it isn't, and the caller has to handle both.

     0-indexed - counting from 0 instead of 1. Players type rows 1-10 but the board array
     uses 0-9, so we subtract 1.

INPUTS

- text the player typed, as a &str. Spaces and lowercase are fine, " b7 " works.

OUTPUTS

 - Ok(Coord) for a valid square, A-J and 1-10
     - Err(ParseError) for anything else. Bad input never crashes the program.

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

// Turns what the player typed (like " b7 ") into a Coord, or a ParseError if it's bad
pub fn parse_move(input: &str) -> Result<Coord, ParseError> {
    // Step 1: clean up the input, " b7 " becomes "B7"
    let cleaned = input.trim().to_uppercase();

    // Step 2: nothing typed at all
    if cleaned.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    // Step 3: split the first character (column letter) from the rest (row number)
    let mut chars = cleaned.chars();
    let letter = match chars.next() {
        Some(c) => c,
        None => return Err(ParseError::EmptyInput),
    };
    let number_text = chars.as_str();

    // Step 4a: the column has to be a letter from A to J
    if !letter.is_ascii_uppercase() || letter > 'J' {
        return Err(ParseError::InvalidLetter);
    }
    let col = (letter as u8 - b'A') as usize; // A -> 0, B -> 1, ... J -> 9

    // Step 4b: the rest has to be a number
    let number = match number_text.parse::<usize>() {
        Ok(n) => n,
        Err(_) => return Err(ParseError::InvalidNumber),
    };

    // Step 4c: the number has to be on the board, 1 to 10
    if number < 1 || number > 10 {
        return Err(ParseError::NumberOutOfRange);
    }
    let row = number - 1; // players count from 1, the array counts from 0

    // Step 5: everything checked out
    Ok(Coord { row, col })
}