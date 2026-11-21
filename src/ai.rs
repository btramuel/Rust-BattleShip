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

/*
   EXPERIMENT 1 - does hunt/target make the AI better?

   Plays 1000 games with each strategy against a fleet from place_fleet_randomly and
   compares how many shots it takes to sink everything.

     Pure random  - the AI is never told how its shots went, so targets stays empty and
                    every shot is a random unfired square
     Hunt/target  - the AI is told every result, so after a hit it chases the squares around it

   Run it with:  cargo test experiment_1 -- --nocapture
*/
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    const GAMES: u32 = 1000;

    // Plays one full game and returns how many shots it took to sink every ship.
    // use_targeting decides whether the AI hears about its hits.
    fn play_one_game(use_targeting: bool) -> u32 {
        let mut board = Board::new();
        board.place_fleet_randomly();

        let mut ai = Ai::new();
        let mut shots = 0;

        while !board.all_sunk() {
            let coord = ai.choose_shot();
            let result = board.fire(coord);
            if use_targeting {
                ai.record_result(coord, &result);
            }
            shots += 1;
        }
        shots
    }

    // Plays GAMES games and returns (average, fewest, most) shots.
    fn run_strategy(use_targeting: bool) -> (f64, u32, u32) {
        let mut total = 0;
        let mut fewest = u32::MAX;
        let mut most = 0;

        for _ in 0..GAMES {
            let shots = play_one_game(use_targeting);
            total += shots;
            if shots < fewest {
                fewest = shots;
            }
            if shots > most {
                most = shots;
            }
        }
        (total as f64 / GAMES as f64, fewest, most)
    }

    #[test]
    fn experiment_1() {
        let (random_avg, random_min, random_max) = run_strategy(false);
        let (hunt_avg, hunt_min, hunt_max) = run_strategy(true);

        println!();
        println!("Experiment 1 - shots to sink the whole fleet, {} games each", GAMES);
        println!("  Pure random:  average {:.1}, fewest {}, most {}", random_avg, random_min, random_max);
        println!("  Hunt/target:  average {:.1}, fewest {}, most {}", hunt_avg, hunt_min, hunt_max);
        println!("  Hunt/target saves {:.1} shots per game on average", random_avg - hunt_avg);

        // the whole point of hunt/target is that it should win faster
        assert!(hunt_avg < random_avg);
    }
}