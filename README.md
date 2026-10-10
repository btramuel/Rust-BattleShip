# Rust Battleship — ITCS 4102/5102

Team Watch Dogs

| Name | Role in this submission |
| --- | --- |
| Brian | `main.rs`, `demo.rs`, README |
| Rebecca | `board.rs` |
| Lee | `parse.rs` |
| Mason | `stats.rs` |

Language: **Rust** 
Repository: https://github.com/btramuel/Rust-BattleShip

As discussed in class, we submitted a single program covering all three concepts instead of three separate ones. The program is the first working part of our term project so the three requirements are demonstrated inside real game code rather than in throwaway examples.

## How to run it

```
cargo run -- demo    # prints every requirement with a label 
cargo run            # plays a full two-player game in the terminal
```

`cargo run -- demo` is a scripted run that calls the same functions the game uses and prints each requirement under its own heading. It takes no input and produces the same output every time.

To play a game, enter two names, place five ships each (a starting square like
`B7`, then `h` or `v`), then take turns firing. Columns are A–J, rows are 1–10.

### Program 1 — four data types, two built in methods each (8 total)

| Type | Methods | Where |
| --- | --- | --- |
| `String` | `.trim()`, `.to_uppercase()` | `parse.rs` → `parse_move()`, line 76 |
| `char` | `.to_ascii_uppercase()`, `.is_ascii_alphabetic()` | `demo.rs` → `program_one()`; `parse.rs` also uses `.is_ascii_uppercase()` at line 92 |
| `u32` | `.max()`, `.checked_sub()` | `demo.rs` → `program_one()`; shot and hit counts come from `main.rs` |
| `f64` | `.round()`, `.clamp()` | `demo.rs` → `program_one()`; the `f64` itself is produced by `stats.rs` → `accuracy()` |

`parse_move()` cleans raw input with `.trim().to_uppercase()` on one line, which
is how `" b7 "` becomes a usable square. `accuracy()` divides hits by shots to
produce the `f64` that `.round()` and `.clamp()` then operate on.

### Program 2 — two data structures, two control structures

**Data structures**

| Structure | Where |
| --- | --- |
| `struct Ship` | `board.rs` — name, size, hit count |
| `struct Board` | `board.rs` — owns the grid, the ship list, and the shot record |
| `struct Coord` | `parse.rs` — a row and column pair |
| `struct MatchResult` | `stats.rs` — one finished game |
| Array `[[Option<usize>; 10]; 10]` | `board.rs` — the 10x10 grid; a second `[[bool; 10]; 10]` tracks shots |
| `Vec<Ship>` | `board.rs` — the ships placed on a board |
| `Vec<MatchResult>` | `stats.rs` — match history loaded from JSON |

**Control structures**

| Structure | Where |
| --- | --- |
| `while` loop | `main.rs` — the turn loop, and `place_fleet()` stepping through the fleet |
| `for` loop | `board.rs` — `place_ship()` checking each square, `render()` walking the grid |
| `loop` | `main.rs` — `read_coord()`, which re-prompts until the input parses |
| `if` / `else` | `main.rs` — deciding whose turn it is and which board is the target |
| `match` | all four files — on `Result`, on `Shot`, and on `Option` |

### Program 3 — error handling

Rust has no exceptions and no try/catch. Instead, any function that can fail returns a `Result`, and the compiler will not build the program until the caller handles both the success and the failure case. That is Rust's equivalent of exception handling, and it is what this program demonstrates.

| Piece | Where |
| --- | --- |
| `enum ParseError` with four variants + `Display` | `parse.rs` — empty input, bad letter, bad number, number out of range |
| `enum StatsError` with two variants + `Display` | `stats.rs` — file I/O failure and corrupted file |
| `Result` return types | `parse_move()`, `place_ship()`, `save_result()`, `load_history()` |
| `?` operator | `stats.rs` → `save_to()`, line 120 |

Three places where this matters in practice:

- Typing a bad square during a game prints the reason and re prompts. It does not crash and does not cost the player their turn.
- A ship that would overlap or hang off the edge is rejected with a message naming the ship, and the player is asked for that ship again.
- A corrupted `history.json` is reported and **left untouched** rather than overwritten, so one bad byte cannot erase every earlier match. Saves are written to a temporary file and renamed into place for the same reason.

`cargo run -- demo` shows all of these by feeding deliberately bad input
(`""`, `"Z9"`, `"B99"`, `"77"`, `"hello"`) through the parser and by creating a
corrupted history file on purpose, then cleaning it up.

## File layout

```
src/
  main.rs        Brian    entry point, turn loop, ship placement prompts
  demo.rs        Brian    scripted requirement showcase
  board.rs       Rebecca  Board and Ship, placement, firing, rendering
  parse.rs       Lee      "B7" into a Coord, ParseError
  stats.rs       Mason    accuracy, MatchResult, JSON save and load
  game.rs                 reserved for later milestones
  ai.rs                   reserved for the computer opponent
  net/                    reserved for the networked two-player mode
```

The empty files are placeholders for use cases we are building later in the term; they are not part of this submission and are not compiled.

## Dependencies

`serde` and `serde_json`, used only to read and write the match history file.
Everything else — the grid, the ships, the error types, the game loop — is the
Rust standard library and our own code.

## Build status

`cargo build` completes with no errors and no warnings.
