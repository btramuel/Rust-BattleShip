/*
   Purpose
     Entry point for the game. Wires the other modules together and runs the turn loop for a local two player match. 
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

mod ai;
mod board;
mod demo;
mod game;
mod net;
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

    // "cargo run demo" runs the requirement showcase instead of a game
    if args.len() > 1 {
        if args[1] == "demo" {
            demo::run();
            return;
        }
        if args[1] == "ai" {
            play_vs_computer();
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
/*
   Use case four, play against the computer. The AI lives in ai.rs and only does two things, pick a square
   to fire at, and remember what happened so it can hunt around a hit. Everything else is the same board
   and the same rules the two player game uses, so nothing in here knows how the AI makes up its mind.

   The computer's fleet goes down with place_fleet_randomly, which is also use case one's random placement.
*/
fn play_vs_computer() {
    println!("Battleship - you against the computer");
    println!("");

    let name = read_line("Your name: ");

    let mut my_board = Board::new();
    let mut ai_board = Board::new();
    let mut brain = ai::Ai::new();

    // the computer always places at random, you get the choice
    ai_board.place_fleet_randomly();

    let answer = read_line("Place your ships yourself, or random? (self/random): ");
    if answer == "random" || answer == "r" {
        my_board.place_fleet_randomly();
        println!("");
        println!("{}", my_board.render(false));
    } else {
        println!("");
        println!("{} place your ships.", name);
        place_fleet(&mut my_board);
    }

    let mut my_shots: u32 = 0;
    let mut my_hits: u32 = 0;
    let mut game_over = false;
    let mut i_won = false;

    while game_over == false {
        // your go
        println!("");
        println!("{}", ai_board.render(true)); // true means hide the computer's ships

        let coord = read_coord("Fire at (e.g. B7): ");
        let result = ai_board.fire(coord);

        let mut took_a_shot = true;
        match result {
            Shot::Hit => {
                my_shots = my_shots + 1;
                my_hits = my_hits + 1;
                println!("Hit!");
            }
            Shot::Sunk(ship_name) => {
                my_shots = my_shots + 1;
                my_hits = my_hits + 1;
                println!("Hit! You sank their {}.", ship_name);
            }
            Shot::Miss => {
                my_shots = my_shots + 1;
                println!("Miss.");
            }
            Shot::AlreadyFired => {
                took_a_shot = false;
                println!("You already fired there. Try again.");
            }
        }

        if ai_board.all_sunk() {
            game_over = true;
            i_won = true;
        }

        // the computer's go, but only if you actually used your turn
        if game_over == false && took_a_shot {
            let ai_coord = brain.choose_shot();
            let ai_result = my_board.fire(ai_coord);

            // tell the AI how that went so it can hunt around a hit next time
            brain.record_result(ai_coord, &ai_result);

            let square = square_name(ai_coord);
            match ai_result {
                Shot::Hit => println!("Computer fires at {} - hit.", square),
                Shot::Sunk(ship_name) => {
                    println!("Computer fires at {} - hit, and sank your {}.", square, ship_name)
                }
                Shot::Miss => println!("Computer fires at {} - miss.", square),
                Shot::AlreadyFired => println!("Computer fired at {} again.", square),
            }

            if my_board.all_sunk() {
                game_over = true;
                i_won = false;
            }
        }
    }

    println!("");
    println!("{}", my_board.render(false));

    if i_won {
        println!("{} wins!", name);
        save_match(name.clone(), String::from("Computer"), my_shots, my_hits);
    } else {
        println!("The computer wins.");
        save_match(String::from("Computer"), name.clone(), my_shots, my_hits);
    }
}

// turn a Coord back into something a person can read, so "row 1 col 6" prints as "G2"
fn square_name(coord: Coord) -> String {
    let letters = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];
    return format!("{}{}", letters[coord.col], coord.row + 1);
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