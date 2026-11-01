/*
   Purpose
     Holds one whole match: both boards, both names, whose turn it is, and the shot counts. right now main.rs
     keeps all of that in loose variables inside the turn loop, so this pulls it into one struct that the game
     thread in net/server.rs can own instead of juggling eight variables. this file is also where the anti-cheat
     happens, because it's the only place that can see both boards at once and decide what each player gets told.

   Key Terms and Definitions
     Match - one game between two players. owns both boards, so nothing else needs to.

     player1_turn - a bool instead of a number. true means it's player one's go, and flipping it is the whole
     turn swap.

     view_for - builds the PlayerView for one player. this is the method that makes the anti-cheat real, because
     it draws the enemy board with hide_ships set to true, so what comes out is just dots and hit markers.
     the ship positions stay in here.

     is_over - the game ends when one fleet is gone, so this just asks both boards whether everything on them is sunk.

   Inputs
     - board.rs - Board and Shot
     - parse.rs - Coord
     - net/mod.rs - PlayerView, which is what view_for hands back

   Output
     - A Match the server can drive one move at a time
     - A PlayerView per player, safe to send over the network
*/
use crate::board::Board;
use crate::board::Shot;
use crate::net::PlayerView;
use crate::parse::Coord;

pub struct Match {
    pub player1_name: String,
    pub player2_name: String,

    pub board1: Board,
    pub board2: Board,

    // true means player one is up
    pub player1_turn: bool,

    pub player1_shots: u32,
    pub player1_hits: u32,
    pub player2_shots: u32,
    pub player2_hits: u32,
}

impl Match {
    // empty match, player one goes first
    pub fn new(player1_name: String, player2_name: String) -> Match {
        Match {
            player1_name: player1_name,
            player2_name: player2_name,
            board1: Board::new(),
            board2: Board::new(),
            player1_turn: true,
            player1_shots: 0,
            player1_hits: 0,
            player2_shots: 0,
            player2_hits: 0,
        }
    }

    // fire at whoever is NOT going right now
    pub fn fire(&mut self, at: Coord) -> Shot {
        let result;
        if self.player1_turn {
            result = self.board2.fire(at);
        } else {
            result = self.board1.fire(at);
        }

        // work out what that shot counted as before we touch anything
        let mut was_a_shot = true;
        let mut was_a_hit = false;

        match result {
            Shot::Hit => {
                was_a_hit = true;
            }
            Shot::Sunk(_) => {
                was_a_hit = true;
            }
            Shot::Miss => {
                // counts as a shot, just didn't land
            }
            Shot::AlreadyFired => {
                // doesn't count, they picked a square they already tried
                was_a_shot = false;
            }
        }

        // add it to whoever just went
        if self.player1_turn {
            if was_a_shot {
                self.player1_shots = self.player1_shots + 1;
            }
            if was_a_hit {
                self.player1_hits = self.player1_hits + 1;
            }
        } else {
            if was_a_shot {
                self.player2_shots = self.player2_shots + 1;
            }
            if was_a_hit {
                self.player2_hits = self.player2_hits + 1;
            }
        }

        // only swap turns if they actually took a shot, and not if they just won
        if was_a_shot {
            if self.is_over() == false {
                if self.player1_turn {
                    self.player1_turn = false;
                } else {
                    self.player1_turn = true;
                }
            }
        }

        return result;
    }

    // game's done when one side has nothing left
    pub fn is_over(&self) -> bool {
        if self.board1.all_sunk() {
            return true;
        }
        if self.board2.all_sunk() {
            return true;
        }
        return false;
    }

    // whoever still has ships. empty string if nobody has won yet
    pub fn winner(&self) -> String {
        if self.board2.all_sunk() {
            return self.player1_name.clone();
        }
        if self.board1.all_sunk() {
            return self.player2_name.clone();
        }
        return String::new();
    }

    // build what one player is allowed to see. the important bit is the `true` we pass to render on the enemy
    // board, which hides their ships, so what comes back is dots, X's and O's with no ship positions in it.
    // we send that string and the real Board never leaves this struct, which is the anti-cheat in one line
    pub fn view_for(&self, player_one: bool) -> PlayerView {
        let my_board;
        let enemy_board;

        if player_one {
            my_board = self.board1.render(false); // your own ships, you can see them
            enemy_board = self.board2.render(true); // theirs, hidden
        } else {
            my_board = self.board2.render(false);
            enemy_board = self.board1.render(true);
        }

        PlayerView {
            my_board: my_board,
            enemy_board: enemy_board,
            my_turn: self.player1_turn == player_one,
        }
    }
}