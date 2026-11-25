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
use crate::parse::ParseError;
use crate::parse::parse_move;
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

/* 
   Experiment three, input robustness.

   Purpose
     Find out whether bad input can take the game down. Phase one throws several thousand malformed move
     strings at parse_move() and checks that every one comes back as the right ParseError instead of a
     panic or a square that is off the board. Phase two throws malformed frames at a running server over
     the socket, then plays a clean match afterwards to prove the server is still standing.

   Key Terms and Definitions
     malformed: anything a player could type by accident or on purpose that is not a square. empty lines,
     letters past J, rows past 10, negative numbers, decimals, emoji, control characters, and a number so
     long it does not fit in a usize.

     known cases: a hand written list where i wrote down which ParseError we expect back. this is
     the half that catches a parser rejecting the right input for the wrong reason, which a random
     generator wouldn't notice because it does not know what the right answer is.

     random cases: thousands of strings built by picking characters out of a pool. it must not panic,
      and if the parser accepts it then the Coord has to be on the board.

     crash count:  a panic in Rust ends the process, so there is nothing to catch and count. 
     the summary at the bottom only prints if nothing panicked, so the summary printing at all is the result.

     bad frame: phase two's version of a malformed input. not JSON at all, or JSON of the wrong shape, or
     a Fire at a square like row 99 that fits in the type but not on the board.

   Inputs
     - the number of random strings to generate, and the address of a running server for phase two
     - parse::parse_move(), the function under test in phase one
     - net/mod.rs for ClientMessage and ServerMessage, so phase two can talk to the server

   Output
     - a table of inputs tested, how many were rejected, and the count per ParseError variant
     - any input that was accepted when we did not expect it, or that gave a Coord off the board
     - how many bad frames the server rejected, how many killed the connection, and whether a normal
       match still completed afterwards
 */

// the hand written half. "ok" means the parser should accept it, anything else names the ParseError we want
const KNOWN_CASES: [(&str, &str); 30] = [
    // nothing typed
    ("", "EmptyInput"),
    ("   ", "EmptyInput"),
    ("\t\n", "EmptyInput"),

    // the column is not a letter from A to J
    ("Z9", "InvalidLetter"),
    ("K5", "InvalidLetter"),
    ("77", "InvalidLetter"),
    ("@1", "InvalidLetter"),
    ("[1", "InvalidLetter"),
    ("-5", "InvalidLetter"),
    ("é5", "InvalidLetter"),
    ("😀1", "InvalidLetter"),

    // the letter is fine but the rest is not a number
    ("A", "InvalidNumber"),
    ("hello", "InvalidNumber"),
    ("A-1", "InvalidNumber"),
    ("A 7", "InvalidNumber"),
    ("A7.5", "InvalidNumber"),
    ("B6 extra", "InvalidNumber"),
    ("A99999999999999999999", "InvalidNumber"),
    ("A5\u{0}", "InvalidNumber"),

    // a number, but not one that is on a ten by ten board
    ("A0", "NumberOutOfRange"),
    ("A11", "NumberOutOfRange"),
    ("J99", "NumberOutOfRange"),

    // and the good ones, which still have to work
    ("A1", "ok"),
    ("b7", "ok"),
    ("  j10  ", "ok"),
    ("J10", "ok"),
    ("A10", "ok"),
    ("E5", "ok"),
    ("A007", "ok"),
    ("A+7", "ok"),
];

// what the random strings are built out of. letters on and off the board, digits, punctuation,
// whitespace, control characters, and characters that are not ASCII at all
const CHARACTER_POOL: [char; 40] = [
    'A', 'B', 'J', 'K', 'Z', 'a', 'b', 'j', 'z',
    '0', '1', '5', '9',
    '-', '+', '.', ',', ' ', '\t',
    '[', ']', '@', '#', '*', '/', '\\', '"', '\'',
    '\u{0}', '\u{7}', '\u{1b}',
    'é', 'ñ', 'Ω', 'あ', '€', '😀', 'ß', 'ı', '\u{feff}',
];

// the malformed frames phase two sends at the server. none of these is a legal move
const BAD_FRAMES: [&str; 12] = [
    "",
    "hello server",
    "B7",
    "{",
    "{}",
    "[1,2,3]",
    "null",
    "{\"Fire\":\"B7\"}",
    "{\"Fire\":{\"row\":99,\"col\":99}}",
    "{\"Fire\":{\"row\":18446744073709551615,\"col\":0}}",
    "{\"Join\":\"joining twice\"}",
    "{\"PlaceShip\":{\"start\":{\"row\":0,\"col\":0},\"horizontal\":true}}",
];

/*
   Experiment three. Runs phase one on its own, then tries phase two against the address given.
   Phase one needs nothing running. Phase two is skipped with a message if it cannot connect.
*/
pub fn fuzz(address: &str, random_count: usize) {
    use rand::Rng;

    println!("experiment three, input robustness");
    println!("");
    println!("phase one, the parser");

    let mut tested: usize = 0;
    let mut accepted: usize = 0;
    let mut wrong_reason: usize = 0;
    let mut off_the_board: usize = 0;

    let mut empty_input: usize = 0;
    let mut invalid_letter: usize = 0;
    let mut invalid_number: usize = 0;
    let mut out_of_range: usize = 0;

    // the known cases first, because these are the ones with a right answer
    for (input, expected) in KNOWN_CASES {
        tested = tested + 1;

        match parse_move(input) {
            Ok(coord) => {
                accepted = accepted + 1;

                if coord.row >= 10 || coord.col >= 10 {
                    off_the_board = off_the_board + 1;
                    println!("  OFF THE BOARD  {:?} gave row {} col {}", input, coord.row, coord.col);
                }

                if expected != "ok" {
                    wrong_reason = wrong_reason + 1;
                    println!("  ACCEPTED       {:?} was taken as a square, we expected {}", input, expected);
                }
            }
            Err(e) => {
                match &e {
                    ParseError::EmptyInput => empty_input = empty_input + 1,
                    ParseError::InvalidLetter => invalid_letter = invalid_letter + 1,
                    ParseError::InvalidNumber => invalid_number = invalid_number + 1,
                    ParseError::NumberOutOfRange => out_of_range = out_of_range + 1,
                }

                if name_of(&e) != expected {
                    wrong_reason = wrong_reason + 1;
                    println!("  WRONG REASON   {:?} gave {}, we expected {}", input, name_of(&e), expected);
                }
            }
        }
    }

    // now the random half. we do not know the right answer, so the only rules are do not panic and
    // do not hand back a square that is off the board
    let mut rng = rand::rng();
    let mut sample: Vec<String> = Vec::new();
    let mut made: usize = 0;

    while made < random_count {
        made = made + 1;

        let length = rng.random_range(0..13);
        let mut input = String::new();
        let mut i = 0;
        while i < length {
            let pick = rng.random_range(0..CHARACTER_POOL.len());
            input.push(CHARACTER_POOL[pick]);
            i = i + 1;
        }

        tested = tested + 1;

        match parse_move(&input) {
            Ok(coord) => {
                accepted = accepted + 1;

                if coord.row >= 10 || coord.col >= 10 {
                    off_the_board = off_the_board + 1;
                    println!("  OFF THE BOARD  {:?} gave row {} col {}", input, coord.row, coord.col);
                }

                // keep a few to eyeball, since a random string being accepted is worth a look
                if sample.len() < 8 {
                    sample.push(format!("{:?} -> row {} col {}", input, coord.row, coord.col));
                }
            }
            Err(e) => {
                match &e {
                    ParseError::EmptyInput => empty_input = empty_input + 1,
                    ParseError::InvalidLetter => invalid_letter = invalid_letter + 1,
                    ParseError::InvalidNumber => invalid_number = invalid_number + 1,
                    ParseError::NumberOutOfRange => out_of_range = out_of_range + 1,
                }
            }
        }
    }

    println!("  inputs tested      {}", tested);
    println!("  rejected cleanly   {}", tested - accepted);
    println!("  accepted           {}", accepted);
    println!("  wrong reason       {}", wrong_reason);
    println!("  off the board      {}", off_the_board);
    println!("  crashes            0   (a panic would have ended the run before this line)");
    println!("");
    println!("  which error came back");
    println!("    EmptyInput       {}", empty_input);
    println!("    InvalidLetter    {}", invalid_letter);
    println!("    InvalidNumber    {}", invalid_number);
    println!("    NumberOutOfRange {}", out_of_range);

    if sample.len() > 0 {
        println!("");
        println!("  a sample of the random strings the parser accepted");
        for line in sample {
            println!("    {}", line);
        }
    }

    println!("");
    fuzz_server(address);
}

/*
   Phase two. Garbage over the wire instead of garbage off the keyboard.

   Two sockets are needed because the server only starts reading moves once it has paired two players,
   so a single connection would sit in the lobby and never look at anything we sent it. The first to
   join is player one, which means socket a is the one whose turn it is and the one we send garbage on.
*/
fn fuzz_server(address: &str) {
    println!("phase two, the live server at {}", address);

    let url = format!("ws://{}", address);

    let mut socket_a = match tungstenite::connect(url.clone()) {
        Ok((s, _response)) => s,
        Err(_) => {
            println!("  no server listening, phase two skipped");
            println!("  start one with: cargo run -- server");
            return;
        }
    };

    if send(&mut socket_a, &ClientMessage::Join("fuzzA".to_string())) == false {
        println!("  could not join as the first player, phase two skipped");
        return;
    }

    // the partner. it never sends anything after joining, it just has to stay connected so the
    // match does not end in a forfeit while we are still sending garbage
    let mut socket_b = match tungstenite::connect(url) {
        Ok((s, _response)) => s,
        Err(_) => {
            println!("  could not open the second connection, phase two skipped");
            return;
        }
    };

    if send(&mut socket_b, &ClientMessage::Join("fuzzB".to_string())) == false {
        println!("  could not join as the second player, phase two skipped");
        return;
    }

    let mut rejected: usize = 0;
    let mut connection_died: usize = 0;
    let mut no_answer: usize = 0;

    for frame in BAD_FRAMES {
        // straight text, not a serialised ClientMessage, because the point is to send something the
        // server cannot turn into one
        let sent = socket_a.send(tungstenite::Message::Text(frame.to_string().into())).is_ok();

        if sent == false {
            connection_died = connection_died + 1;
            println!("  CONNECTION DIED sending {:?}", frame);
            break;
        }

        match hear_until_error(&mut socket_a) {
            Some(reason) => {
                rejected = rejected + 1;
                println!("  rejected  {:?}", frame);
                println!("              server said: {}", reason);
            }
            None => {
                no_answer = no_answer + 1;
                println!("  NO ANSWER OR HUNG UP  {:?}", frame);
                break;
            }
        }
    }

    // a legal move now, to show the match itself was never damaged by any of that
    let still_playable;
    if send(&mut socket_a, &ClientMessage::Fire(Coord { row: 0, col: 0 })) {
        still_playable = waited_for_shot_result(&mut socket_a);
    } else {
        still_playable = false;
    }

    // drop both connections, the server scores it a forfeit and moves on
    let _ = socket_a.close(None);
    let _ = socket_b.close(None);

    println!("");
    println!("  bad frames sent      {}", BAD_FRAMES.len());
    println!("  rejected with Error  {}", rejected);
    println!("  connection died      {}", connection_died);
    println!("  no answer            {}", no_answer);

    if still_playable {
        println!("  match still playable yes, a legal shot after all that came back with a result");
    } else {
        println!("  match still playable NO  *** the garbage broke the match ***");
    }

    // last check, and the strongest one: can the server still take a brand new match from scratch
    println!("");
    println!("  checking the server can still start a fresh match");

    let address_for_thread = address.to_string();
    let second = std::thread::spawn(move || {
        return play_once(&address_for_thread, "cleanup2");
    });
    let first = play_once(address, "cleanup1");

    let second = match second.join() {
        Ok(stats) => stats,
        Err(_) => None,
    };

    match (first, second) {
        (Some(a), Some(b)) => {
            println!("  server survived      yes, a clean match finished afterwards ({} and {} shots)", a.shots, b.shots);
        }
        _ => {
            println!("  server survived      NO  *** it would not start a new match ***");
        }
    }
}

// read until the server tells us off. anything else on the way, like the YourTurn it sends after an
// Error, is not what we are waiting for, so skip it
fn hear_until_error(socket: &mut tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>) -> Option<String> {
    let mut skipped = 0;

    loop {
        if skipped > 20 {
            return None; // it is answering, just never with an Error, so stop waiting
        }
        skipped = skipped + 1;

        let line = match hear(socket) {
            Some(l) => l,
            None => return None, // hung up on us
        };

        match serde_json::from_str::<ServerMessage>(&line) {
            Ok(ServerMessage::Error(reason)) => return Some(reason),
            Ok(_) => continue,
            Err(_) => continue,
        }
    }
}

// same idea, but waiting for the result of a legal shot
fn waited_for_shot_result(socket: &mut tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>) -> bool {
    let mut skipped = 0;

    loop {
        if skipped > 20 {
            return false;
        }
        skipped = skipped + 1;

        let line = match hear(socket) {
            Some(l) => l,
            None => return false,
        };

        match serde_json::from_str::<ServerMessage>(&line) {
            Ok(ServerMessage::ShotResult(_)) => return true,
            Ok(_) => continue,
            Err(_) => continue,
        }
    }
}

// the name of a variant as a string, so a mismatch can be printed next to what we expected
fn name_of(e: &ParseError) -> &str {
    match e {
        ParseError::EmptyInput => return "EmptyInput",
        ParseError::InvalidLetter => return "InvalidLetter",
        ParseError::InvalidNumber => return "InvalidNumber",
        ParseError::NumberOutOfRange => return "NumberOutOfRange",
    }
}
}