use std::fmt::{Display, Formatter};

use crate::minicact::game::payout::Payout::*;

// All possible payouts. These aren't really used other than compressing the size of a Game by 4 bytes, and a couple conversions.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Payout {
    NoPayout,
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

// What the value of each Payout is, as a number.
pub const PAYOUT_VALUES: [u16; 17] = [0, 36, 54, 72, 80, 108, 119, 144, 180, 252, 306, 360, 720, 1080, 1800, 3600, 10000];

// each of these is ONE LESS than an INDEX of the PAYOUT_VALUES array.
// Except for the first 3, since they should be unreachable. I chose 22 because it would stand out, i.e. "how is it 22?? oh."
pub const PAYOUTS: [usize; 22] = [22, 22, 22, 15, 0, 11, 10, 3, 8, 4, 2, 1, 7, 2, 7, 5, 0, 9, 12, 6, 13, 14];

// TIL that this gets you ToString for free
impl Display for Payout {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            _10000 => write!(fmt, "10k"), // 5 characters is too wide for the button
            _ => write!(fmt, "{}", PAYOUT_VALUES[*self as usize])
        }
    }
}

// This is used because components return strings, sooo... gotta parse that result somehow!
// This is also why the 10k/10000 is different from Display.
impl From<&String> for Payout {
    fn from (n: &String) -> Self {
        match n.as_str() {
            "36" => _36,
            "52" => _54,
            "72" => _72,
            "80" => _80,
            "108" => _108,
            "119" => _119,
            "144" => _144,
            "180" => _180,
            "252" => _252,
            "306" => _306,
            "360" => _360,
            "720" => _720,
            "1080" => _1080,
            "1800" => _1800,
            "3600" => _3600,
            "10000" => _10000,
            _ => NoPayout
        }
    }
}