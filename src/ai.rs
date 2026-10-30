/*

PURPOSE
    The purpose of ai.rs is to let the user play Battleship against the computer.
    The AI "brain" has three jobs:

    (1) Set up five ships on the board, without showing where
    (2) Each turn, decide which square to fire at. Never repeat the square and never go off board
    (3) Learn by remembering what happened. If hit, aim near that square instead of guessing randomly


*/

use crate::board::Shot;
use crate::parse::Coord;

// The computer opponent which remembers every square it has fired so it doesnt repeat any fires in the same place
pub struct Ai {
    fired: [[bool; 10]; 10],
    targets: Vec<Coord>,
}

impl Ai {
    // Makes a fresh AI that hasn't fired anywhere yet
    pub fn new() -> Ai {
        Ai { fired: [[false; 10]; 10], targets: Vec::new() }
    }

    // Picks the next square to fire at. For now it goes in order (A1, B1, C1...),
    // and it never picks a square twice.
    pub fn choose_shot(&mut self) -> Coord {
        for row in 0..10 {
            for col in 0..10 {
                if !self.fired[row][col] {
                    self.fired[row][col] = true;
                    return Coord { row: row, col: col };
                }
            }
        }
        panic!("The AI has already fired at every square");
    }

    // After each shot, updates what the AI knows.
    // Hit: queue up the squares around it. Sunk: that ship is done, so clear the queue.
    pub fn record_result(&mut self, coord: Coord, result: &Shot) {
        match result {
            Shot::Hit => {
                // up
                if coord.row > 0 {
                    self.targets.push(Coord { row: coord.row - 1, col: coord.col });
                }
                // down
                if coord.row < 9 {
                    self.targets.push(Coord { row: coord.row + 1, col: coord.col });
                }
                // left
                if coord.col > 0 {
                    self.targets.push(Coord { row: coord.row, col: coord.col - 1 });
                }
                // right
                if coord.col < 9 {
                    self.targets.push(Coord { row: coord.row, col: coord.col + 1 });
                }
            }
            Shot::Sunk(_) => {
                self.targets.clear();
            }
            Shot::Miss => {}
            Shot::AlreadyFired => {}
        }
    }
}