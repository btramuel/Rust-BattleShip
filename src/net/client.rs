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