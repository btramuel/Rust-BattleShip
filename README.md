# Rust Battleship — ITCS 4102/5102 Term Project

**Team Watch Dogs**

| Name | Student ID | Section | Files owned |
| --- | --- | --- | --- |
| Brian Tramuel | 801441165 | ITCS 5102 | `main.rs`, `game.rs`, `demo.rs`, `net/mod.rs`, README |
| Rebecca Carpenter | 801427942 | ITCS 5102 | `stats.rs`, `web/` |
| Lee Tropezado | 801421216 | ITCS 4102 | `parse.rs`, `ai.rs` |
| Mason Paccadolmi | 801411781 | ITCS 4102 | `board.rs`, `net/server.rs` |

Language: **Rust** , with HTML, CSS and JavaScript for the browser client.

Two player Battleship. It plays locally in the terminal, against a computer
opponent, and online between two machines over WebSocket, with the server
holding both boards so neither player can see the other's fleet.

## How to run it

```
cargo run                        # local two-player game in the terminal
cargo run -- ai                  # play against the computer
cargo run -- server              # start the game server on 127.0.0.1:8080
cargo run -- server 0.0.0.0:8080 # start it so another machine on the network can join
cargo run -- demo                # scripted showcase of the language requirements
```

For a terminal game, enter two names, place five ships each, a starting square
such as `B7`, then `h` or `v`, or type `random`, then take turns firing.
Columns are A–J and rows are 1–10.

For a networked game, start the server, then open `src/web/index.html` in a
browser on each machine and enter a name. The first two players to join are
paired into a match. Fleets are placed randomly for networked games.

## Use cases

| # | Use case | Status | Where |
| --- | --- | --- | --- |
| 1 | Ship placement, manual or random, rejecting overlaps and out-of-bounds ships | Complete | `board.rs`, `main.rs` |
| 2 | Firing, returning hit, miss, sunk, or already fired | Complete | `board.rs` |
| 3 | Turn order and win detection | Complete | `main.rs`, `game.rs` |
| 4 | Play against the computer, hunting around a hit | Complete | `ai.rs`, `main.rs` |
| 5 | Online two-player over WebSocket | Complete | `net/server.rs` |
| 6 | Server-side anti-cheat and disconnect handling | Complete | `game.rs`, `net/server.rs` |
| 7 | Match history and accuracy, surviving a restart | Complete | `stats.rs` |

### How the anti-cheat works

The server owns both boards for the whole match and never sends a board to a
client. After every move it builds a separate view per player with `view_for`
in `game.rs`, that player's own fleet in full, and the opponent's waters drawn
with the ships hidden. What crosses the network is a grid of dots, hit markers
and miss markers. An un hit ship square is never in the message, so there is
nothing to find by inspecting the traffic.

## File layout

```
src/
  main.rs        Brian    entry point, turn loop, placement prompts, vs-computer mode
  game.rs        Brian    Match struct owning both boards; view_for builds the per-player view
  demo.rs        Brian    scripted showcase of the language requirements
  net/mod.rs     Brian    message protocol: ClientMessage, ServerMessage, PlayerView
  net/server.rs  Mason    WebSocket server, lobby, match thread
  board.rs       Mason    Board and Ship, placement, firing, rendering
  parse.rs       Lee      "B7" into a Coord, ParseError
  ai.rs          Lee      computer opponent, hunt and target
  stats.rs       Rebecca  accuracy, MatchResult, JSON save and load
  web/           Rebecca  browser client: two grids, click to fire, status line
```

`src/net/client.rs` is empty. It was reserved for a scripted test client and
was not written.

## Language requirements

### Four data types, two built in methods each

| Type | Methods | Where |
| --- | --- | --- |
| `String` | `.trim()`, `.to_uppercase()` | `parse.rs` → `parse_move()` |
| `char` | `.to_ascii_uppercase()`, `.is_ascii_alphabetic()` | `demo.rs` → `program_one()`; `parse.rs` also uses `.is_ascii_uppercase()` |
| `u32` | `.max()`, `.checked_sub()` | `demo.rs` → `program_one()`; counts come from `main.rs` |
| `f64` | `.round()`, `.clamp()` | `demo.rs` → `program_one()`; the `f64` comes from `stats.rs` → `accuracy()` |

### Data structures and control structures

Structs: `Ship` and `Board` in `board.rs`, `Coord` in `parse.rs`, `Match` in
`game.rs`, `MatchResult` in `stats.rs`, `PlayerView` in `net/mod.rs`.
Enums: `Shot`, `ParseError`, `StatsError`, `ClientMessage`, `ServerMessage`.
Arrays: the 10x10 grid and a second 10x10 array tracking shots, both in
`board.rs`; the AI's fired squares grid in `ai.rs`.
`Vec`: the ships on a board, the match history, the AI's target queue.

Control structures: `while` for the turn loops, `for` over the grid in
`place_ship` and `render`, `loop` in `read_coord` re prompting until the input
parses, `if`/`else` throughout, and `match` on `Result`, `Shot` and `Option` in
every file.

### Error handling

Rust has no exceptions and no try/catch. Anything that can fail returns a
`Result`, and the compiler will not build the program until the caller handles
both outcomes. We wrote two error enums with `Display` implementations:
`ParseError` in `parse.rs`, empty input, bad letter, bad number, out of range
and `StatsError` in `stats.rs`, file I/O failure, corrupted file. The `?`
operator is used in `stats.rs` → `save_to()`.

Three places it matters:

- A bad square during a game prints the reason and re prompts. It does not
  crash and does not cost the player their turn.
- A ship that would overlap or run off the edge is rejected with a message
  naming the ship, and the player is asked for that ship again.
- A corrupted `history.json` is reported and left untouched rather than
  overwritten, so one bad byte cannot erase every earlier match. Saves are
  written to a temporary file and renamed into place for the same reason.
- Over the network, malformed JSON, an out of bounds square, or a move sent out
  of turn is rejected by the server without ending the match.

`cargo run -- demo` shows the parser cases by feeding it `""`, `"Z9"`, `"B99"`,
`"77"` and `"hello"`, and by writing a corrupted history file on purpose and
then cleaning it up.

## Dependencies

| Crate | Used for |
| --- | --- |
| `serde`, `serde_json` | the match history file and the WebSocket message protocol |
| `rand` | random ship placement |
| `tungstenite` | the WebSocket server |

Everything else, the grid, the ships, the error types, the game loops, the
threading is the Rust standard library and our own code.

## Build status

`cargo build` completes with no errors. Two dead code warnings remain, for
`Match::winner` and the two name fields on `Match`, which the networked server
tracks separately.
