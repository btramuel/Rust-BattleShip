/*
   Purpose


   Key Terms and Definitions

   Inputs


   Output

 */

use crate::parse::Coord;

const BOARD_SIZE: usize = 10;

//Represents a ship placed on the board
#[derive(Debug, Clone)]
pub struct Ship {
    pub name: String,
    pub size: usize,
    pub hits: usize,
}

// Check if the ship is sunk
impl Ship {
    pub fn is_sunk(&self) -> bool {
        self.hits >= self.size
    }
}

// Possible results of a shot fired at the board
#[derive(Debug, PartialEq, Eq)]
pub enum Shot {
    Hit, 
    Sunk(String),
    Miss, 
    AlreadyFired,
}

pub struct Board{
    // 10x10 array
    grid: [[Option<usize>; BOARD_SIZE]; BOARD_SIZE],

    // A growable collecton (Vec) of ships on the board
    ships: Vec<Ship>,

    // Tracks whether each square has been fired upon 
    // False means the square has not been fired upon
    shots: [[bool; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    // New function that creates an empty board and returns it
    pub fn new() -> Self {
        Self {
            // 10x10 array of none values, meaning no ships are placed yet
            grid: [[None; BOARD_SIZE]; BOARD_SIZE],
            // Creates an empty vector of ships
            ships: Vec::new(),
            // 10x10 array of false values, meaning no shots have been fired yet
            shots: [[false; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    // Places a ship on the board at the given coordinate and orientation
    // Returns an error if the ship cannot be placed due to overlap or out of bounds
    pub fn place_ship(&mut self, ship: Ship, coord: Coord, horizontal: bool) -> Result<(), String> {
        let row = coord.row;
        let col = coord.col;

        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            return Err("The starting coordinate is out of bounds for the board.".to_string());
        }

        for offset in 0..ship.size {

            let (r, c) = if horizontal { (row, col + offset) } else { (row + offset, col) };

            // Check if the ship fits within the board boundaries
            if r >= BOARD_SIZE || c >= BOARD_SIZE {
                return Err(format!("Ship {} does not fit on the board at the given position.", ship.name));
            }

            // Check if the ship overlaps with another ship
            if self.grid[r][c].is_some() {
                return Err(format!("Ship {} overlaps with another ship at {:?}", ship.name, (r, c)));
            }
        }

        // Place the ship on the grid
        for i in 0..ship.size {
            let (r, c) = if horizontal { (row, col + i) } else { (row + i, col) };
            // Store the index of the ship in the grid
            self.grid[r][c] = Some(self.ships.len());
        }

        // Add the ship to the list of ships
        self.ships.push(ship);
        Ok(())
    }

    // Fires at the given coordinate on the board and returns the result of the shot
    pub fn fire(&mut self, coord: Coord) -> Shot {
        let row = coord.row;
        let col = coord.col;

        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            return Shot::Miss; // Out of bounds shots are treated as misses
        }

        if self.shots[row][col] {
            return Shot::AlreadyFired; // Already fired upon this square
        }

        self.shots[row][col] = true; // Mark the square as fired upon

        // If a ship is hit, increment its hit count and check if it's sunk
        match self.grid[row][col] {
            Some(ship_index) => {
                let ship = &mut self.ships[ship_index];
                ship.hits += 1;

                if ship.is_sunk() {
                    Shot::Sunk(ship.name.clone())
                } else {
                    Shot::Hit
                }
            }
            None => Shot::Miss,
        }
    }

    // Checks if all ships on the board have been sunk
    pub fn all_sunk(&self) -> bool {
        !self.ships.is_empty()
            && self.ships.iter().all(|ship| ship.is_sunk())
    }

    // Renders the board as a string for display, optionally hiding ships
    pub fn render(&self, hide_ships: bool) -> String {
        let mut output = String::new();

        output.push_str("    A B C D E F G H I J\n");
        output.push_str("   -------------------\n");

        for row in 0..BOARD_SIZE {
            output.push_str(&format!("{:2} |", row + 1));

            for col in 0..BOARD_SIZE {
                let symbol = if self.shots[row][col] {
                    match self.grid[row][col] {
                        Some(_) => 'X',
                        None => 'O',
                    }
                } else if self.grid[row][col].is_some() && !hide_ships {
                    'S'
                } else {
                    '.'
                };

                output.push(' ');
                output.push(symbol);
            }

            output.push('\n');
        }

        output
    }
}