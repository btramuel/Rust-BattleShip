
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

//send a message to the player. Returns true if the message was sent successfully, false otherwise
fn say(player: &mut Player, text: &str) -> bool {
    return player.socket.send(Message::Text(text.into())).is_ok();
}

// wait for one text message. None means the player disconnected
fn hear(player: &mut Player) -> Option<String> {
    loop {
        match player.socket.read() {
            Ok(Message::Text(text)) => return Some(text.as_str().trim().to_string()),
            Ok(Message::Close(_)) => return None,
            Ok(_) => continue,
            Err(_) => return None,
        }
    }
}

// start listening. call this from main.rs, e.g. net::server::run("0.0.0.0:8080")
pub fn run(address: &str) {
    let listener = TcpListener::bind(address).expect("could not bind the address");
    println!("listening on {}", address);
 
    // the player waiting for an opponent, if there is one
    let mut lobby: Option<(String, Player)> = None;
 
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
 
        // turn the raw connection into a websocket
        let mut player = match Player::new(stream) {
            Some(p) => p,
            None => continue,
        };
 
        // the first thing they have to say is JOIN name
        let name = match hear(&mut player) {
            Some(line) if line.starts_with("JOIN ") && line[5..].trim().len() > 0 => {
                line[5..].trim().to_string()
            }
            _ => {
                say(&mut player, "ERROR first message must be: JOIN yourname");
                continue;
            }
        };
 
        say(&mut player, &format!("WELCOME {}", name));
 
        // if somebody is already waiting, pair them up. otherwise this player waits
        match lobby.take() {
            Some((name1, player1)) => {
                thread::spawn(move || {
                    play_match(name1, player1, name, player);
                });
            }
            None => {
                say(&mut player, "WAITING for an opponent");
                lobby = Some((name, player));
            }
        }
    }
}

// stub: just proves that two players got paired
fn play_match(name1: String, mut p1: Player, name2: String, mut p2: Player) {
    say(&mut p1, &format!("MATCH FOUND - you are playing {}", name2));
    say(&mut p2, &format!("MATCH FOUND - you are playing {}", name1));
}

// lets you start the server without touching main.rs:  cargo test serve -- --nocapture
// it blocks forever, stop it with Ctrl+C
#[cfg(test)]
mod tests {
    #[test]
    fn serve() {
        super::run("127.0.0.1:8080");
    }
}