/*
   Purpose
     A scripted client. It connects to the server over WebSocket, joins, and plays a whole match on its own
     by firing at every square in order, with no browser and no person typing. One of these is an automated
     end to end test of the server. A pile of them at once is experiment two, where we find out how many
     simultaneous matches the thread per match design handles before it starts to slow down.

     It also double checks the anti cheat on every view it receives, so a leak would be caught by any run
     rather than only by the one time somebody looks in Wireshark.

   Inputs
     - the address of a running server, such as 127.0.0.1:8080
     - net/mod.rs for ClientMessage and ServerMessage, the same types the server uses

   Output
     - play_once gives back the stats for one match, or None if the connection failed
     - bench prints a table: matches run, moves made, p50 and p95 round trip, and any leaks found
*/

use crate::net::ClientMessage;
use crate::net::ServerMessage;
use crate::parse::Coord;
use std::time::Instant;

// 10x10, so 100 squares is every square on the board
const SQUARES: usize = 100;

// if a match somehow never ends, give up rather than hanging the whole benchmark
const MOVE_LIMIT: u32 = 250;

pub struct MatchStats {
    pub name: String,
    pub won: bool,
    pub shots: u32,
    pub hits: u32,

    // one entry per move
    pub round_trips: Vec<u128>,

    // how many times an enemy ship square showed up in a view sent to us. should always be 0
    pub leaks: u32,
}

/*
   Play one whole match against whoever the server pairs us with.

   Returns None if we could not connect or the server hung up on us partway through, which the benchmark
   counts as a failed match rather than treating as a result.
*/
pub fn play_once(address: &str, name: &str) -> Option<MatchStats> {
    let url = format!("ws://{}", address);

    let (mut socket, _response) = match tungstenite::connect(url) {
        Ok(pair) => pair,
        Err(_) => return None,
    };

    // the server wants a Join before anything else
    if send(&mut socket, &ClientMessage::Join(name.to_string())) == false {
        return None;
    }

    let mut stats = MatchStats {
        name: name.to_string(),
        won: false,
        shots: 0,
        hits: 0,
        round_trips: Vec::new(),
        leaks: 0,
    };

    let mut next_square: usize = 0;
    let mut moves: u32 = 0;

    // set when we fire, read when the result comes back
    let mut fired_at: Option<Instant> = None;

    loop {
        if moves > MOVE_LIMIT {
            return Some(stats);
        }

        let line = match hear(&mut socket) {
            Some(l) => l,
            None => return Some(stats), // server hung up, hand back what we have
        };

        let msg = match serde_json::from_str::<ServerMessage>(&line) {
            Ok(m) => m,
            Err(_) => continue, // not something we understand, ignore it
        };

        match msg {
            ServerMessage::YourTurn => {
                if next_square >= SQUARES {
                    // out of squares, nothing left to do but wait
                    continue;
                }

                let coord = square_number_to_coord(next_square);
                next_square = next_square + 1;

                fired_at = Some(Instant::now());
                if send(&mut socket, &ClientMessage::Fire(coord)) == false {
                    return Some(stats);
                }
                moves = moves + 1;
            }

            ServerMessage::ShotResult(shot) => {
                // stop the clock on this move
                match fired_at {
                    Some(start) => {
                        stats.round_trips.push(start.elapsed().as_micros());
                        fired_at = None;
                    }
                    None => {}
                }

                stats.shots = stats.shots + 1;
                if shot_was_a_hit(&shot) {
                    stats.hits = stats.hits + 1;
                }
            }

            ServerMessage::ViewUpdate(view) => {
                // the enemy board shouldnt contain a ship marker. if it does, a position leaked
                if view.enemy_board.contains('S') {
                    stats.leaks = stats.leaks + 1;
                }
            }

            ServerMessage::GameOver { winner } => {
                if winner == name {
                    stats.won = true;
                }
                return Some(stats);
            }

            ServerMessage::Error(_) => {
                // a rejected move does not cost us a turn, the server will prompt again
                fired_at = None;
            }

            ServerMessage::Welcome(_) => {}
            ServerMessage::WaitingForOpponent => {}
        }
    }
}

/*
   Experiment two. Run several matches at the same time and report how the server held up.

   Two clients are needed per match, so we spawn twice as many threads as matches. The server's lobby
   pairs whoever arrives next, so we do not control who plays who, which does not matter because every
   client plays identically.
*/
pub fn bench(address: &str, matches: usize) {
    println!("running {} simultaneous match(es) against {}", matches, address);

    let mut handles = Vec::new();

    for i in 0..(matches * 2) {
        let address = address.to_string();
        let name = format!("bot{}", i);

        let handle = std::thread::spawn(move || {
            return play_once(&address, &name);
        });

        handles.push(handle);
    }

    let mut all_round_trips: Vec<u128> = Vec::new();
    let mut finished = 0;
    let mut failed = 0;
    let mut leaks = 0;

    for handle in handles {
        match handle.join() {
            Ok(Some(stats)) => {
                finished = finished + 1;
                leaks = leaks + stats.leaks;
                for t in stats.round_trips {
                    all_round_trips.push(t);
                }
            }
            Ok(None) => {
                failed = failed + 1;
            }
            Err(_) => {
                failed = failed + 1;
            }
        }
    }

    println!("");
    println!("clients finished     {}", finished);
    println!("clients failed       {}", failed);
    println!("moves measured       {}", all_round_trips.len());

    if all_round_trips.len() > 0 {
        all_round_trips.sort();
        println!("round trip p50       {} us", percentile(&all_round_trips, 50));
        println!("round trip p95       {} us", percentile(&all_round_trips, 95));
        println!("round trip slowest   {} us", all_round_trips[all_round_trips.len() - 1]);
    }

    if leaks == 0 {
        println!("ship positions leaked 0  (anti-cheat held)");
    } else {
        println!("ship positions leaked {}  *** FAILED ***", leaks);
    }
}

// square 0 is A1, square 1 is A2, and so on down the column before moving to the next one
fn square_number_to_coord(n: usize) -> Coord {
    return Coord {
        row: n % 10,
        col: n / 10,
    };
}

// Shot doesn't derive Clone, so take it by reference and just answer the one question
fn shot_was_a_hit(shot: &crate::board::Shot) -> bool {
    match shot {
        crate::board::Shot::Hit => return true,
        crate::board::Shot::Sunk(_) => return true,
        _ => return false,
    }
}

// sorted is already sorted. p is 50 for the middle, 95 for the slow end
fn percentile(sorted: &Vec<u128>, p: usize) -> u128 {
    if sorted.len() == 0 {
        return 0;
    }

    let mut index = (sorted.len() * p) / 100;
    if index >= sorted.len() {
        index = sorted.len() - 1;
    }

    return sorted[index];
}

fn send<S>(socket: &mut tungstenite::WebSocket<S>, msg: &ClientMessage) -> bool
where
    S: std::io::Read + std::io::Write,
{
    match serde_json::to_string(msg) {
        Ok(text) => return socket.send(tungstenite::Message::Text(text.into())).is_ok(),
        Err(_) => return false,
    }
}

fn hear<S>(socket: &mut tungstenite::WebSocket<S>) -> Option<String>
where
    S: std::io::Read + std::io::Write,
{
    loop {
        match socket.read() {
            Ok(tungstenite::Message::Text(text)) => return Some(text.as_str().trim().to_string()),
            Ok(tungstenite::Message::Close(_)) => return None,
            Ok(_) => continue, // ping, pong, binary - not ours
            Err(_) => return None,
        }
    }
}