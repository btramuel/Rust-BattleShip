
use crate::board::Shot;
use crate::board::BOARD_SIZE;
use crate::game::Match;
use crate::net::ClientMessage;
use crate::net::PlayerView;
use crate::net::ServerMessage;
use crate::stats;
use crate::stats::MatchResult;
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
fn send(player: &mut Player, msg: &ServerMessage) -> bool {
    match serde_json::to_string(msg) {
        Ok(text) => player.socket.send(Message::Text(text.into())).is_ok(),
        Err(_) => false,
    }
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
 
        // the first thing they have to say is {"Join":"name"}
        let name = match hear(&mut player)
            .and_then(|line| serde_json::from_str::<ClientMessage>(&line).ok())
        {
            Some(ClientMessage::Join(n)) if !n.trim().is_empty() => n.trim().to_string(),
            _ => {
                send(&mut player, &ServerMessage::Error(
                    "first message must be a Join with your name".to_string()));
                continue;
            }
        };

        send(&mut player, &ServerMessage::Welcome(name.clone()));

        match lobby.take() {
            Some((name1, player1)) => {
                thread::spawn(move || {
                    play_match(name1, player1, name, player);
                });
            }
            None => {
                send(&mut player, &ServerMessage::WaitingForOpponent);
                lobby = Some((name, player));
            }
        }
    }
}

// stub: just proves that two players got paired
fn play_match(name1: String, mut p1: Player, name2: String, mut p2: Player) {
    let mut game = Match::new(name1.clone(), name2.clone());

    // no placement phase in the protocol yet, so both fleets go down randomly
    game.board1.place_fleet_randomly();
    game.board2.place_fleet_randomly();

    send_views(&game, &mut p1, &mut p2);

    // Some(true) means player one won by forfeit, Some(false) means player two did
    let mut forfeit: Option<bool> = None;

    while !game.is_over() {
        let p1_up = game.player1_turn;
        let (mover, waiter) = if p1_up { (&mut p1, &mut p2) } else { (&mut p2, &mut p1) };

        send(mover, &ServerMessage::YourTurn);
        send(waiter, &ServerMessage::WaitingForOpponent);

        // None from hear() means the mover disconnected, so the other player wins
        let line = match hear(mover) {
            Some(line) => line,
            None => {
                forfeit = Some(!p1_up);
                break;
            }
        };

        // anything that isn't valid JSON for a ClientMessage re-prompts, no turn lost
        let msg = match serde_json::from_str::<ClientMessage>(&line) {
            Ok(m) => m,
            Err(_) => {
                send(mover, &ServerMessage::Error("could not understand that message".to_string()));
                continue;
            }
        };

        let coord = match msg {
            ClientMessage::Fire(c) => c,
            _ => {
                send(mover, &ServerMessage::Error("only Fire is allowed during a match".to_string()));
                continue;
            }
        };

        // Board::fire counts out-of-bounds as a miss, so reject it here instead of burning their turn
        if coord.row >= BOARD_SIZE || coord.col >= BOARD_SIZE {
            send(mover, &ServerMessage::Error("that square is off the board".to_string()));
            continue;
        }

        let result = game.fire(coord);

        // Match::fire doesn't swap turns on this, so just ask again
        if result == Shot::AlreadyFired {
            send(mover, &ServerMessage::Error("you already fired there, pick another square".to_string()));
            continue;
        }

        send(mover, &ServerMessage::ShotResult(result));
        send_views(&game, &mut p1, &mut p2);
    }

    // a forfeit decides it directly, otherwise whoever sank the other fleet won
    let p1_wins = forfeit.unwrap_or(game.board2.all_sunk());
    let (winner, loser) = if p1_wins { (&name1, &name2) } else { (&name2, &name1) };

    if forfeit.is_some() {
        let remaining = if p1_wins { &mut p1 } else { &mut p2 };
        send(remaining, &ServerMessage::Error("opponent disconnected, you win by forfeit".to_string()));
    }

    // sending to the player who left just fails quietly
    let over = ServerMessage::GameOver { winner: winner.clone() };
    send(&mut p1, &over);
    send(&mut p2, &over);

    let (shots, hits) = if p1_wins {
        (game.player1_shots, game.player1_hits)
    } else {
        (game.player2_shots, game.player2_hits)
    };

    let result = MatchResult { winner: winner.clone(), loser: loser.clone(), shots, hits };
    if let Err(e) = stats::save_result(&result) {
        eprintln!("could not save match result: {}", e);
    }
}

// each player gets their own view, with the enemy ships hidden by view_for. this is the anti-cheat step
fn send_views(game: &Match, p1: &mut Player, p2: &mut Player) {
    let v1: PlayerView = game.view_for(true);
    let v2: PlayerView = game.view_for(false);
    send(p1, &ServerMessage::ViewUpdate(v1));
    send(p2, &ServerMessage::ViewUpdate(v2));
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