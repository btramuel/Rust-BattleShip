/*
   Purpose
        The browser side of the Battleship game. Draws two 10x10 grids and a status line, sends the square
        the player clicks to the server, and shows whatever the server sends back. It holds no game rules.
        The server decides hits, misses and whose turn it is, and this file only displays the result.

   Key Terms and Definitions

        Board text: A board drawn by Board::render in board.rs 
        mine / enemy: The two boards as 10x10 arrays of single characters 
        myTurn: True when the server says it is our turn. ...
        awaiting: True between firing a shot and the server's reply 
        onMessage: The function that handles every message the server sends. 
        DOM: The browser's live tree of page elements. 
        Mock mode: Opening index.html?mock swaps the server for a fake opponent 

   Inputs
        - The name typed into the join form
        - Clicks on enemy grid cells
        - Server messages, as plain text or JSON (ServerMessage in net/mod.rs)
        - index.html, which supplies the empty containers this file fills in

   Output
        - Both grids and the status line, redrawn after every server message
        - "JOIN name" when the connection opens, then the clicked square, such as "B7"
        - Every message the server sends, logged to the console as RAW for debugging
*/

"use strict";

// settings 
const WS_URL = "ws://" + (location.hostname || "localhost") + ":8080";

const SIZE = 10; // board size
const COLS = "ABCDEFGHIJ" // game columns

const fireMsg = (r, c) => useJson ? JSON.stringify({ Fire: { row: r, col: c } }) : COLS[c] + (r + 1); // get grid position (e.g. B7)
const joinMsg = name => useJson ? JSON.stringify({ Join: name }) : "JOIN " + name;

const SEND_JSON = false;

const MOCK = new URLSearchParams(location.search).has("mock"); // mock server for testing 
const useJson = SEND_JSON && !MOCK;

const SYMBOLS = { ".": "water", "X": "hit", "O": "miss", "S": "ship" };

// server text, will be updated when server.rs is finished 
const YOUR_TURN = /your turn/i;
const THEIR_TURN = /enemy's turn/i;
const REJECTED = /^ERROR/i;

// game state variables 
const $ = id => document.getElementById(id);

let ws = null; // websocket connection
let mine = null, enemy = null; // boards 
let myTurn = false;
let awaiting = false;
let finished = false;
let unlockTimer = null;
let myName = "";
let resultNote = ""; // last shot information 

// write to status line 
function setStatus(text, kind) {
    const element = $("status");
    element.textContent = text;
    element.className = kind || "";
}

const canFire = () => enemy !== null && myTurn && !awaiting && !finished;

// render grid 
function buildGrid(container, clickable) {
    container.innerHTML = "";
    container.appendChild(document.createElement("div")).className = "lbl";

    // Letters A-J
    for (let c = 0; c < SIZE; c++) {
        const letter = document.createElement("div");
        letter.className = "lbl";
        letter.textContent = COLS[c];
        container.appendChild(letter);
    }

    // Numbers 1-10 and cells
    for (let r = 0; r < SIZE; r++) {
        const num = document.createElement("div");
        num.className = "lbl";
        num.textContent = r + 1;
        container.appendChild(num);

        for (let c = 0; c < SIZE; c++) {
            const button = document.createElement("button");
            button.type = "button";
            button.className = "cell";
            button.dataset.r = r;
            button.dataset.c = c;
            button.disabled = true;

            if (clickable) {
                button.addEventListener("click", () => fire(r, c));
            }

            container.appendChild(button);
        }
    }
}

// color in cells 
function paint(container, rows, isEnemy) {
    container.querySelectorAll(".cell").forEach(b => {
        const r = Number(b.dataset.r);
        const c = Number(b.dataset.c);

        const kind = SYMBOLS[(rows && rows[r] || [])[c]] || "water";

        const live = isEnemy && canFire() && kind === "water";

        b.className = "cell" + (kind === "water" ? "" : " " + kind) + (live ? " live" : "");
        b.disabled = !live;
    });

    if (isEnemy) container.classList.toggle("waiting", !canFire());
}

function render() {
    paint($("mine"), mine, false);
    paint($("enemy"), enemy, true);
}

const HEADER = /^\s*A B C D E F G H I J\s*$/;
const ROW = /^\s*\d+\s*\|(.*)$/;
const DIVIDER = /^\s*-{5,}\s*$/;

function splitMessage(text) {
    const boards = [], notes = [];
    let cur = null; // the board we are currently reading rows into, or null
    let label = ""; // the most recent heading line

    for (const line of String(text).split(/\r?\n/)) {

        // header line: a new board starts here
        if (HEADER.test(line)) {                                 
            cur = { rows: [], label: label };                       
            boards.push(cur);                                      
            continue;                                              
        }

        // a row line, and we are inside a board
        const m = line.match(ROW); // match returns null if no match, else an array:
        if (m && cur) {                                          
            const cells = m[1].trim().split(/\s+/);               
            if (cells.length === SIZE) cur.rows.push(cells);       
            continue;
        }

        if (DIVIDER.test(line)) continue;                       
        if (!line.trim()) { cur = null; continue; }             
        cur = null;

        
        if (/:\s*$/.test(line)) label = line;
        else notes.push(line.trim());
    }
    return { boards: boards, notes: notes };
}

// called once for every message the server sends
function onMessage(text) {
    // raw message
    console.log("RAW", JSON.stringify(text));

    // if the message is JSON, handle it separately and stop
    const parsed = tryJson(text);
    if (parsed !== null) return onServerMessage(parsed);

    const { boards, notes } = splitMessage(text);

    // Case 1: the message has both boards, so it is a game update
    if (boards.length >= 2) {
        let [a, b] = boards;
        const isEnemyLabel = label => /\b(enemy|opponent|their)/i.test(label);
        const isMyLabel = label => /\b(your|my|own)\b/i.test(label);
        if (isEnemyLabel(a.label) || isMyLabel(b.label)) [a, b] = [b, a];

        mine = a.rows;
        enemy = b.rows;
        awaiting = false; // the server answered, so the lock is released
        clearTimeout(unlockTimer); // cancel the 4 second safety timer (see fire())

        const note = notes.join(" ");                           
        if (YOUR_TURN.test(note)) myTurn = true;
        else if (THEIR_TURN.test(note)) myTurn = false;
        else if (!note) myTurn = true;                          

        if (/win|lose/i.test(note) && !/(your|their|opponent)'?s? (turn|go)/i.test(note)) finished = true;

        $("boards").hidden = false; // reveal the grids (hidden until now)
        render();
        setStatus(note || (myTurn ? "Your turn." : "Opponent's turn."), myTurn && !finished ? "turn" : "");
        return;                                                  
    }

    // Case 2: no boards in the message, so it is just text for the status line

    if (REJECTED.test(text)) {
        // our shot was refused: unlock the board so the player can try another square
        awaiting = false;
        clearTimeout(unlockTimer);
        render();
        setStatus(text.replace(/^ERROR\s*/, ""), "error");        // strip the "ERROR " prefix before showing it
    } else if (/^WELCOME/.test(text)) {
        setStatus("Welcome, " + text.slice(8) + ".");             // slice(8) drops the first 8 characters ("WELCOME ")
    } else if (/^WAITING/.test(text)) {
        setStatus("Waiting for an opponent to join.");
    } else {
        if (YOUR_TURN.test(text)) myTurn = true;
        else if (THEIR_TURN.test(text)) myTurn = false;
        render();
        setStatus(text.replace(/^MATCH FOUND - /, "Match found: "), myTurn ? "turn" : "");
    }
}

function tryJson(text) {
    const t = String(text).trim();
    if (t[0] !== "{" && t[0] !== '"') return null;
    try { return JSON.parse(t); }        
    catch (e) { return null; }
}

// turns a drawn board (the text from Board::render) into a 10x10 array, or null if there's no board in it
function rowsFrom(boardText) {
    const first = splitMessage(boardText).boards[0];
    return first ? first.rows : null;
}

// rebuilds the status line from what we know: the last shot result, plus whose turn it is
function refreshStatus() {
    const turnText = finished ? "" : (myTurn ? "Your turn." : "Opponent's turn.");
    setStatus((resultNote ? resultNote + " " : "") + turnText, myTurn && !finished ? "turn" : "");
}

// handles one decoded ServerMessage
function onServerMessage(msg) {
    // A bare string means a variant with no data ("YourTurn"). An object has one key, the variant name,
    // and its value is the data. Object.keys(obj) lists an object's keys.
    const kind = typeof msg === "string" ? msg : Object.keys(msg)[0];
    const data = typeof msg === "string" ? null : msg[kind];

    switch (kind) {                        
        case "Welcome":
            setStatus("Welcome, " + data + ".");
            break;

        case "WaitingForOpponent":
            myTurn = false;
            render();
            refreshStatus();
            break;

        case "YourTurn":
            myTurn = true;
            render();
            refreshStatus();
            break;

        case "ShotResult":
            if (data === "AlreadyFired") {
                // refused, so unlock and let them pick another square
                awaiting = false; clearTimeout(unlockTimer); render();
                setStatus("You already fired there. Try again.", "error");
            } else {
                resultNote = data === "Hit" ? "Hit!"
                    : data === "Miss" ? "Miss."
                        : data && data.Sunk ? "Hit! You sank their " + data.Sunk + "."
                            : "";
                refreshStatus();
            }
            break;

        case "ViewUpdate":
            mine = rowsFrom(data.my_board);
            enemy = rowsFrom(data.enemy_board);
            myTurn = !!data.my_turn;           // !! forces any value to true or false
            awaiting = false; clearTimeout(unlockTimer);
            $("boards").hidden = false;
            render();
            refreshStatus();
            break;

        case "Error":
            awaiting = false; clearTimeout(unlockTimer); render();
            setStatus(String(data), "error");
            break;

        case "GameOver":
            finished = true;
            render();
            setStatus(data.winner === myName ? "Game over. You win!" : "Game over. " + data.winner + " wins.");
            break;

        default:
            console.log("Unknown message type", kind);   // so a new kind of message shows up in the console instead of vanishing
    }
}

// networking 
// sends one text message to the server. In mock mode it goes to the fake server instead.
function send(text) {
    if (MOCK) return mockSend(text);
    if (ws && ws.readyState === WebSocket.OPEN) ws.send(text);   // only send if the connection is open
}

// runs when the player clicks an enemy cell (see buildGrid)
function fire(r, c) {
    if (!canFire()) return;                                     // ignore stray clicks
    send(fireMsg(r, c));
    resultNote = "";                                            // forget the previous shot's result
    awaiting = true;                                            // lock the board until the server answers
    render();
    setStatus("Shot sent. Waiting for the server.");

    unlockTimer = setTimeout(() => { awaiting = false; render(); }, 4000);
}

// opens the connection and registers handlers
function connect(name) {
    if (MOCK) return mockStart(name);
    setStatus("Connecting…");
    ws = new WebSocket(WS_URL);

    // connection is ready: the first thing the server wants is "JOIN name"
    ws.onopen = () => ws.send(joinMsg(name));

    // a message arrived. e.data is the text. String() makes sure it's text
    ws.onmessage = e => onMessage(String(e.data));

    // couldn't connect, or the connection failed
    ws.onerror = () => setStatus("Could not reach the server at " + WS_URL + ".", "error");

    // the connection closed. Only complain if the game wasn't already over
    ws.onclose = () => {
        if (!finished) { finished = true; render(); setStatus("Disconnected from the server.", "error"); }
    };
}

/* ---- Mock mode ----
Open index.html?mock to play against a fake server that lives inside this file. It uses the same text
format as Board::render so the screen can be tested before the real server is available.
The ships are hardcoded, and the fake opponent just fires at A1, B1, C1... in order. */

const MOCK_SHIPS_ME = new Set(["0,0", "0,1", "0,2", "3,5", "4,5", "5,5", "8,2", "8,3"]);
const MOCK_SHIPS_ENEMY = new Set(["1,1", "1,2", "1,3", "6,6", "7,6", "8,6", "9,0", "9,1"]);
const mockShotsOnMe = new Set(), mockShotsOnEnemy = new Set();   // squares already fired at, per board
let mockNextAi = 0; // next square number (0..99) the fake opponent fires at
    const TOTAL_CELLS = SIZE * SIZE;

    function allCellsDestroyed(ships, shots) {
    return [...ships].every(cell => shots.has(cell));
    }

    function nextMockAiShot() {
    while (mockNextAi < TOTAL_CELLS) {
        const ar = Math.floor(mockNextAi / SIZE);
        const ac = mockNextAi % SIZE;
        mockNextAi += 1;

        if (!mockShotsOnMe.has(ar + "," + ac)) return { ar, ac };
    }
    return null;
    }

function mockBoard(ships, shots, hide) {
    let out = "    A B C D E F G H I J\n   -------------------\n";
    for (let r = 0; r < SIZE; r++) {
        out += String(r + 1).padStart(2) + " |";                     
        for (let c = 0; c < SIZE; c++) {
            const k = r + "," + c, isShip = ships.has(k);             
            out += " " + (shots.has(k) ? (isShip ? "X" : "O") : (isShip && !hide ? "S" : "."));
        }
        out += "\n";
    }
    return out;
}

// builds a full "game update" message: both boards plus a line of text
function mockMessage(note) {
    return "Your board:\n" + mockBoard(MOCK_SHIPS_ME, mockShotsOnMe, false) +
        "\nEnemy board:\n" + mockBoard(MOCK_SHIPS_ENEMY, mockShotsOnEnemy, true) + "\n" + note;
}

function mockStart(name) {
    setTimeout(() => onMessage("WELCOME " + name), 100);
    setTimeout(() => onMessage("MATCH FOUND - you are playing Mock Opponent"), 400);
    setTimeout(() => onMessage(mockMessage("Your turn.")), 800);
}

function mockSend(text) {
    // regex with capture groups: one letter A-J, then digits. m[1] is the letter, m[2] the digits
    const m = text.trim().toUpperCase().match(/^([A-J])(\d+)$/);
    const r = m ? +m[2] - 1 : -1, c = m ? COLS.indexOf(m[1]) : -1;   
    if (!m || r < 0 || r >= SIZE) return setTimeout(() => onMessage("ERROR That is off the map."), 200);

    const k = r + "," + c;
    if (mockShotsOnEnemy.has(k)) return setTimeout(() => onMessage("ERROR You already fired there. Try again."), 200);

    mockShotsOnEnemy.add(k);
    const hit = MOCK_SHIPS_ENEMY.has(k);
    if (allCellsDestroyed(MOCK_SHIPS_ENEMY, mockShotsOnEnemy)) {
        onMessage(mockMessage("Game over. You win!"));
        return;
    }

    setTimeout(() => {
        const enemyShot = nextMockAiShot();
        if (!enemyShot) {
            onMessage(mockMessage("Game over. No more squares remain."));
            return;
        }

        const { ar, ac } = enemyShot;
        mockShotsOnMe.add(ar + "," + ac);
        const aHit = MOCK_SHIPS_ME.has(ar + "," + ac);
        if (allCellsDestroyed(MOCK_SHIPS_ME, mockShotsOnMe)) {
            onMessage(mockMessage("Game over. You lose."));
            return;
        }
        onMessage(mockMessage("You fired at " + text.trim().toUpperCase() + ": " + (hit ? "hit" : "miss") +
            ". Opponent fired at " + COLS[ac] + (ar + 1) + ": " + (aHit ? "hit" : "miss") + ". Your turn."));
    }, 600);
}

/* ---- Start ---- */
$("join").addEventListener("submit", e => {
    e.preventDefault(); // stop the browser's default form behavior (reloading the page)
    const name = $("name").value.trim(); // .value is what the user typed into the input box
    if (!name) return;
    myName = name;
    $("join").hidden = true;                                     
    buildGrid($("mine"), false);                                
    buildGrid($("enemy"), true);                                
    $("boards").hidden = false;
    connect(name);
});

