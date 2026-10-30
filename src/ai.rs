/*

PURPOSE
    The purpose of ai.rs is to let the user play Battleship against the computer.
    The AI "brain" has three jobs:

    (1) Set up five ships on the board, without showing where
    (2) Each turn, decide which square to fire at. Never repeat the square and never go off board
    (3) Learn by remembering what happened. If hit, aim near that square instead of guessing randomly


*/

// The computer opponent which remembers every square it has fired so it doesnt repeat any fires in the same place
pub struct Ai {
    fired: [[bool; 10]; 10],
}

impl Ai {
    // Makes a fresh AI that hasn't fired anywhere yet
    pub fn new() -> Ai {
        Ai { fired: [[false; 10]; 10] }
    }
}