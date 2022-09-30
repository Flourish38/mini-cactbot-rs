use serenity::prelude::*;

enum Payout {
    _36,
    _54,
    _72,
    _80,
    _108,
    _119,
    _144,
    _180,
    _252,
    _306,
    _360,
    _720,
    _1080,
    _1800,
    _3600,
    _10000
}

struct Game {
    tiles_revealed: u8,
    position_history: [u8; 12],
    number_history: [u8; 12],
    games_played: u8,
    score_history: [Payout; 3],
}