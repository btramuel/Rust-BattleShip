
use crate::board::Ship;
use crate::board::Shot;
use crate::board::FLEET;
use crate::game::Match;
use crate::parse;
use crate::stats;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use tungstenite::Message;
use tungstenite::WebSocket;

struct Player {
    socket: WebSocket<TcpStream>
}

impl Player {
    // does the websocket handshake. None means handshake failed
    fn new(stream: TcpStream) -> Option<Player> {
        match tungstenite::accept(stream) {
            Ok(socket) => Some(Player { socket: socket }),
            Err(_) => None,
        }
    }
}