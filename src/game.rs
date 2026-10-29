/*
   Purpose
     Holds one whole match: both boards, both names, whose turn it is, and the shot counts. right now main.rs
     keeps all of that in loose variables inside the turn loop, so this pulls it into one struct that the game
     thread in net/server.rs can own instead of juggling eight variables. this file is also where the anti-cheat
     happens, because it's the only place that can see both boards at once and decide what each player gets told.

   Key Terms and Definitions
     Match - one game between two players. owns both boards, so nothing else needs to.

     player1_turn - a bool instead of a number. true means it's player one's go, and flipping it is the whole
     turn swap.

     view_for - builds the PlayerView for one player. this is the method that makes the anti-cheat real, because
     it draws the enemy board with hide_ships set to true, so what comes out is just dots and hit markers.
     the ship positions stay in here.

     is_over - the game ends when one fleet is gone, so this just asks both boards whether everything on them is sunk.

   Inputs
     - board.rs - Board and Shot
     - parse.rs - Coord
     - net/mod.rs - PlayerView, which is what view_for hands back

   Output
     - A Match the server can drive one move at a time
     - A PlayerView per player, safe to send over the network
*/