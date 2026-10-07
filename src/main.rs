/*
   Purpose
     Entry point for the game. Wires the other modules together and runs the turn loop for a local two-player match. 
     Also handles the "demo" argument that runs the requirement showcase instead of a game.

   Key Terms and Definitions
     mod - the line at the top of this file that tells Rust a .rs file exists. Without it the compiler never looks at board.rs, parse.rs, etc.

     hot-seat - both players take turns on the same machine, passing the keyboard back and forth. No networking involved.

     turn loop - the while loop that keeps running until one player's fleet is sunk. Each pass: show the board, read a move, fire, swap players.

     stdin - standard input, meaning whatever the player types into the terminal. We read a line at a time and hand it to parse_move().

     Shot - the enum board::fire() gives back. Tells us whether the shot was a Hit, Miss, Sunk, or a square that was AlreadyFired at.

   Inputs
     - command line args - "demo" runs the showcase, anything else starts a game
     - keyboard input - player names and moves like "B7", read from stdin
     - parse::parse_move() - turns typed text into a Coord, or an error
     - board::Board - holds the grid and ships, handles placing and firing
     - stats::save_result() - writes the finished match to history.json

   Output
     - Prints both boards and the result of each shot to the terminal
     - Prints the winner when all of one player's ships are sunk
     - Saves the match result so stats survive between runs

 */

mod board;
mod demo;
mod parse;
mod stats;

use board::Board;
use board::Ship;
use board::Shot;
use parse::Coord;
use std::io;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // "cargo run demo runs the requirement showcase instead of a game
    if args.len() > 1 {
        if args[1] == "demo" {
            demo::run();
            return;
        }
    }

    println!("Battleship");
    println!("");

    let player1_name = read_line("Player 1 name: ");
    let player2_name = read_line("Player 2 name: ");

    let mut board1 = Board::new();
    let mut board2 = Board::new();

    println!("");
    println!("{} place your ships.", player1_name);
    place_fleet(&mut board1);

    println!("");
    println!("{} place your ships.", player2_name);
    place_fleet(&mut board2);

    let mut player1_shots: u32 = 0;
    let mut player1_hits: u32 = 0;
    let mut player2_shots: u32 = 0;
    let mut player2_hits: u32 = 0;

    let mut player1_turn = true;
    let mut game_over = false;

    // filled in when somebody wins, used for saving the match at the end
    let mut winner_name = String::new();
    let mut loser_name = String::new();
    let mut winner_shots: u32 = 0;
    let mut winner_hits: u32 = 0;

    while game_over == false {
        // figure out whose turn it is and which board they are shooting at
        let current_name;
        let enemy_board;
        if player1_turn {
            current_name = player1_name.clone();
            enemy_board = &mut board2;
        } else {
            current_name = player2_name.clone();
            enemy_board = &mut board1;
        }

        println!("");
        println!("{}'s turn", current_name);
        println!("{}", enemy_board.render(true)); // true means hide their ships

        let coord = read_coord("Fire at (e.g. B7): ");
        let result = enemy_board.fire(coord);

        let mut was_a_shot = true;
        let mut was_a_hit = false;

        match result {
            Shot::Hit => {
                was_a_hit = true;
                println!("Hit!");
            }
            Shot::Sunk(ship_name) => {
                was_a_hit = true;
                println!("Hit! You sank their {}.", ship_name);
            }
            Shot::Miss => {
                println!("Miss.");
            }
            Shot::AlreadyFired => {
                // this one does not count, they just picked a square they already tried
                was_a_shot = false;
                println!("You already fired there. Try again.");
            }
        }

        let everything_sunk = enemy_board.all_sunk();

        // add to whichever player just went
        if player1_turn {
            if was_a_shot {
                player1_shots = player1_shots + 1;
            }
            if was_a_hit {
                player1_hits = player1_hits + 1;
            }
        } else {
            if was_a_shot {
                player2_shots = player2_shots + 1;
            }
            if was_a_hit {
                player2_hits = player2_hits + 1;
            }
        }

        if everything_sunk {
            println!("");
            println!("{} wins!", current_name);
            game_over = true;

            if player1_turn {
                winner_name = player1_name.clone();
                loser_name = player2_name.clone();
                winner_shots = player1_shots;
                winner_hits = player1_hits;
            } else {
                winner_name = player2_name.clone();
                loser_name = player1_name.clone();
                winner_shots = player2_shots;
                winner_hits = player2_hits;
            }
        } else {
            // only swap turns if they actually took a shot
            if was_a_shot {
                if player1_turn {
                    player1_turn = false;
                } else {
                    player1_turn = true;
                }
            }
        }
    }

    save_match(winner_name, loser_name, winner_shots, winner_hits);
}

// walks a player through placing every ship in the fleet
fn place_fleet(board: &mut Board) {
    let ship_names = ["Carrier", "Battleship", "Cruiser", "Submarine", "Destroyer"];
    let ship_sizes = [5, 4, 3, 3, 2];

    let mut i = 0;
    while i < ship_names.len() {
        let name = ship_names[i];
        let size = ship_sizes[i];

        let prompt = format!("{} ({} squares) - start square: ", name, size);
        let coord = read_coord(&prompt);

        let direction = read_line("Horizontal or vertical? (h/v): ");
        let mut horizontal = true;
        if direction == "v" || direction == "V" {
            horizontal = false;
        }

        let ship = Ship {
            name: String::from(name),
            size: size,
            hits: 0,
        };

        match board.place_ship(ship, coord, horizontal) {
            Ok(()) => {
                println!("{}", board.render(false)); // false means show your own ships
                i = i + 1; // only move on to the next ship if this one actually fit
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

fn save_match(winner: String, loser: String, shots: u32, hits: u32) {
    let result = stats::MatchResult {
        winner: winner,
        loser: loser,
        shots: shots,
        hits: hits,
    };

    match stats::save_result(&result) {
        Ok(()) => {
            let percent = stats::accuracy(hits, shots);
            println!("Saved. Accuracy: {:.1}%", percent);
        }
        Err(e) => {
            // a broken history file should not lose somebody their game, just say so
            println!("Couldn't save the match: {}", e);
        }
    }
}

// keeps asking until they type something the parser understands
fn read_coord(prompt: &str) -> Coord {
    loop {
        let input = read_line(prompt);
        match parse::parse_move(&input) {
            Ok(coord) => {
                return coord;
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

// prompt, then read one line of input back
fn read_line(prompt: &str) -> String {
    print!("{}", prompt);

    // print! doesn't flush on its own, so without this the prompt shows up after the
    // cursor is already waiting and it looks like the program froze
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    return String::from(input.trim());
}