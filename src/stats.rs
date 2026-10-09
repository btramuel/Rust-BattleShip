/*
   Purpose
     Keeps track of how each match went and remembers it between runs. Works out accuracy as a percentage,
     saves finished matches to a JSON file, and loads them back, including when the file is missing or damaged.

   Key Terms and Definitions
     accuracy - hits divided by shots, times 100. A player who has fired zero shots has 0% accuracy
     rather than a divide by zero crash.

     MatchResult - one finished game: who won, who lost, and the winner's shots and hits.
     This is written to a JSON file.

     JSON - a plain text format for storing structured data. history.json holds a list of MatchResults
     so the file can be opened and read in any text editor.

     serde - the crate that turns a MatchResult into JSON text and back again. 

     StatsError - error type for the two ways the file can go wrong: Io (couldn't read or write it)
     and Corrupt (it's there, but the contents aren't valid history).

     first run - history.json doesn't exist yet. Loading gives back an empty list.

   Inputs
     - main.rs - hands over a MatchResult at the end of a game, and hits and shots when it wants an accuracy
     - history.json - the saved matches from earlier runs, read back when a new result is saved

   Output
     - save_result() adds the new match to history.json, or returns a StatsError 
     - accuracy() returns back a percentage as an f64
     - load_history() returns every saved match as a Vec<MatchResult>
     - A corrupted history file is reported and left untouched, never overwritten, so one bad
       byte can't wipe out every earlier match

 */

use std::fmt; 
use std::fs; 
use std::io::ErrorKind;
use serde::{Deserialize, Serialize};
use serde_json::Result;

// saved matches json
const HISTORY_FILE: &str = "history.json";

// derive serialize & deserialize for serde read/write 
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchResult {
    pub winner: String,
    pub loser: String,
    pub shots: u32,
    pub hits: u32,
}

// error types for IO & corrupt files 
#[derive(Debug)]
pub enum StatsError { 
    Io(std::io::Error),
    Corrupt(serde_json::Error),
}

// display for StatsError 
impl fmt::Display for StatsError { 
    fn fmt(&self, f: &mut fmt:Formatter) -> fmt::Result {
        match self { 
            StatsError::Io(e) => write!(f, "Couldn't read or write {}: {}", HISTORY_FILE, e),
            StatsError::Corrupt(e) => write!(f, "{} is corrupted. ({})", HISTORY_FILE, e),
        }
    }
}

// calculates accuracy of shots 
pub fn accuracy(hits: u32, shots: u32) -> f64 {
    return;
}

// return every saved match, oldest to newest, missing file is an empty list 
pub fn load_history() -> Result<Vec<MatchResult>, StatsError> { 
    return;
}

// add a finished match to history file 
pub fn save_result(result: &MatchResult) -> Result<(), StatsError> { 
    return;
}

// loads match from file path 
fn load_from(path: & str) -> Result<Vec<MatchResult>, StatsError> {
    return; 
} 

// saves match to history file 
fn save_to(path: &str, result: &MatchResult) -> Result<(), StatsError>{
    return;
}