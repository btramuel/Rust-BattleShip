/*

PURPOSE
    The purpose of ai.rs is to let the user play Battleship against the computer.
    The computer's fleet is placed by place_fleet_randomly in board.rs, so the AI "brain" has two jobs:

    (1) Each turn, decide which square to fire at. Never repeat the square and never go off board
    (2) Learn by remembering what happened. If hit, aim near that square instead of guessing randomly


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

    // Picks the next square to fire at. If a recent hit queued up nearby squares,
    // it tries those first. Otherwise it picks a random square it hasn't fired at yet.
    // It never picks a square twice.
    pub fn choose_shot(&mut self) -> Coord {
        // Target mode: chase squares next to a hit
        while let Some(target) = self.targets.pop() {
            if !self.fired[target.row][target.col] {
                self.fired[target.row][target.col] = true;
                return target;
            }
        }

         // Hunt mode: nothing queued, so collect every unfired square...
        let mut options = Vec::new();
        for row in 0..10 {
            for col in 0..10 {
                if !self.fired[row][col] {
                    options.push(Coord { row: row, col: col });
                }
            }
        }

        if options.is_empty() {
            panic!("The AI has already fired at every square");
        }

        // ...and pick one of them at random
        let pick = options[rand::random_range(0..options.len())];
        self.fired[pick.row][pick.col] = true;
        pick
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