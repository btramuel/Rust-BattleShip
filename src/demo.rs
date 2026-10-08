/*
   Purpose
     A scripted run that shows every Oct 9 requirement in one place. this will run "cargo run -- demo" and sees each requirement
      printed with a label instead of having to play a full game to find them.

   Key Terms and Definitions
     scripted - the moves are hardcoded, not typed by a user. Same every time it runs, so the output is predictable and easy to grade.

     requirement - one of the three program requirements from the assignment. Each gets its own labeled section in the output.

     real functions - this file calls parse.rs, board.rs, and stats.rs directly. It does NOT copy their logic.
     If the game works, the demo works, and there is only one version of everything to maintain.

   Inputs
     - nothing from the user, everything is hardcoded in this file
     - parse::parse_move() - for the String and char work, plus the error handling
     - board::Board - for the structs, array, Vec, loops, and if/else
     - stats::accuracy() -for the integer and float work

   Output
     - Labeled lines printed to the terminal, grouped by requirement, so each one
       can be checked off against the assignment
*/

use crate::board::Board;
use crate::board::Ship;
use crate::board::Shot;
use crate::parse;
use crate::stats;

pub fn run() {
    println!("=== Rust Battleship - requirement demo ===");
    println!("Watch Dogs - Brian Tramuel, Rebecca, Lee, Mason");
    println!("");

    program_one();
    program_two();
    program_three();

    println!("");
    println!("Full requirement mapping is in README.md");
}

// 4 data types, 2 built in methods each
fn program_one() {
    println!("--- Program 1: data types and built-in methods ---");
    println!("");

    let raw = String::from("  b7  ");
    println!("String  starting value      [{}]", raw);
    println!("String  .trim()             [{}]", raw.trim());
    println!("String  .to_uppercase()     [{}]", raw.trim().to_uppercase());

    let letter = 'b';
    println!("char    .to_ascii_uppercase()   {}", letter.to_ascii_uppercase());
    println!("char    .is_ascii_alphabetic()  {}", letter.is_ascii_alphabetic());

    let shots: u32 = 24;
    let hits: u32 = 9;
    println!("u32     .max()              {}", shots.max(hits));

    // checked_sub hands back None instead of crashing when the answer would go below zero
    match hits.checked_sub(shots) {
        Some(answer) => println!("u32     .checked_sub()      {}", answer),
        None => println!("u32     .checked_sub()      None (would go below zero)"),
    }

    let percent = stats::accuracy(hits, shots);
    println!("f64     accuracy            {}", percent);
    println!("f64     .round()            {}", percent.round());
    println!("f64     .clamp()            {}", percent.clamp(0.0, 100.0));
}

// 2 data structures, struct, array, Vec and 2 control structures, while, for, if/else, match
fn program_two() {
    println!("");
    println!("--- Program 2: data structures and control structures ---");
    println!("");

    // Board is a struct holding a 10x10 array and a Vec of Ship structs
    let mut board = Board::new();

    let destroyer = Ship {
        name: String::from("Destroyer"),
        size: 2,
        hits: 0,
    };

    match parse::parse_move("A1") {
        Ok(start) => match board.place_ship(destroyer, start, true) {
            Ok(()) => println!("placed Destroyer at A1, horizontal"),
            Err(e) => println!("placement failed: {}", e),
        },
        Err(e) => println!("could not read A1: {}", e),
    }

    // for loop over an array of squares, firing at each one
    let squares = ["A1", "A2", "E5"];
    for square in squares {
        match parse::parse_move(square) {
            Ok(coord) => {
                let result = board.fire(coord);
                println!("fire at {} -> {}", square, shot_text(result));
            }
            Err(e) => {
                println!("fire at {} -> {}", square, e);
            }
        }
    }

    println!("");
    println!("{}", board.render(false));
}

// error handling, which is Rust's answer to exceptions
fn program_three() {
    println!("");
    println!("--- Program 3: error handling ---");
    println!("");
    println!("Rust has no exceptions or try/catch. Anything that can fail gives back a");
    println!("Result, and the caller has to deal with both cases before it will compile.");
    println!("");

    // every one of these is bad input, and none of them crash the program
    let bad_inputs = ["", "Z9", "B99", "77", "hello"];
    for bad in bad_inputs {
        match parse::parse_move(bad) {
            Ok(coord) => println!("[{}] parsed as row {} col {}", bad, coord.row, coord.col),
            Err(e) => println!("[{}] rejected: {}", bad, e),
        }
    }

    println!("");

    // same idea with files, which fail for reasons outside our control
    match stats::load_history("file_that_does_not_exist.json") {
        Ok(history) => println!("loaded {} past matches", history.len()),
        Err(e) => println!("missing history file handled cleanly: {}", e),
    }
}

// turns a Shot into words so we can print it
fn shot_text(result: Shot) -> String {
    match result {
        Shot::Hit => String::from("hit"),
        Shot::Miss => String::from("miss"),
        Shot::Sunk(ship_name) => format!("sunk the {}", ship_name),
        Shot::AlreadyFired => String::from("already fired there"),
    }
}