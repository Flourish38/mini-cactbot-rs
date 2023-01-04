use std::fmt::{Display, Formatter};

use crate::minicact::game::payout::Payout::*;

#[derive(PartialEq, Clone, Copy)]
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

pub const PAYOUT_VALUES: [u16; 17] = [0, 36, 54, 72, 80, 108, 119, 144, 180, 252, 306, 360, 720, 1080, 1800, 3600, 10000];

// pub const PAYOUTS: [Payout; 25] = [NoPayout, NoPayout, NoPayout, NoPayout, NoPayout, NoPayout, _10000, _36, _720, _360, _80, _252, _108, _72, _54, _180, _72, _180, _119, _36, _306, _1080, _144, _1800, _3600];

// TIL that this gets you ToString for free
impl Display for Payout {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            _10000 => write!(fmt, "10k"), // 5 characters is too wide for the button
            _ => write!(fmt, "{}", PAYOUT_VALUES[*self as usize])
        }
    }
}

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