/*
   Purpose
     The message protocol. This file says what the client and the server are allowed to say to each other,
     and nothing else. No networking code lives here yet, just the shapes of the messages. Both sides have to
     agree on these before either one can be written, which is why this file goes first.

   Key Terms and Definitions
     protocol - the agreed list of messages. if the client sends something that isn't a ClientMessage,
     the server won't understand it.

     ClientMessage - anything a player's browser sends up to the server: joining, placing a ship, firing a shot.

     ServerMessage - anything the server sends back down: whose turn it is, what a shot did, the new state
     of the boards, who won.

     PlayerView - what ONE player is allowed to see, and the whole point of the anti cheat use case. the server
     owns both boards but never sends a whole board down. it sends this instead, with the enemy board already
     drawn with the ships hidden, so the positions never leave the server at all. there is nothing to dig out
     of the network traffic because it was never in there.

     serde - the crate that turns these into JSON text and back. the derive lines are what make that work.

   Inputs
     - board.rs - Shot, so we can tell the player what their shot did
     - parse.rs - Coord, so we can say which square they mean

   Output
     - The three types below, used by net/server.rs, game.rs, and the browser front end
*/

// these two are empty for now, but the folder isn't a module without them
pub mod client;
pub mod server;

use crate::board::Shot;
use crate::parse::Coord;
use serde::Deserialize;
use serde::Serialize;

// everything a player can send up to the server
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    // the name they typed on the way in
    Join(String),

    // where a ship goes, and which way it points
    PlaceShip { start: Coord, horizontal: bool },

    // the square they're firing at
    Fire(Coord),
}

// everything the server can send back down
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    // you're connected, here's the name we have for you
    Welcome(String),

    // go ahead, it's your turn
    YourTurn,

    // sit tight, the other player is going
    WaitingForOpponent,

    // what your last shot did
    ShotResult(Shot),

    // the boards as they look to YOU right now
    ViewUpdate(PlayerView),

    // something went wrong, here's a message to show the player
    Error(String),

    // game's done
    GameOver { winner: String },
}

// what one player is allowed to see. my_board shows your own ships, enemy_board is drawn with theirs
// hidden, so it's only dots, X's for your hits and O's for your misses. we send the drawn text and not
// the board itself, so an un-hit ship square never goes over the wire
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerView {
    pub my_board: String,
    pub enemy_board: String,
    pub my_turn: bool,
}