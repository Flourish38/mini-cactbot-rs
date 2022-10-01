use crate::game::payout::Payout::*;

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

pub const PAYOUTS: [Payout; 25] = [NoPayout, NoPayout, NoPayout, NoPayout, NoPayout, NoPayout, _10000, _36, _720, _360, _80, _252, _108, _72, _54, _180, _72, _180, _119, _36, _306, _1080, _144, _1800, _3600];

impl From<Payout> for u16 {
    fn from(payout: Payout) -> Self {
        match payout {
            NoPayout => 0,
            _36 => 36,
            _54 => 54,
            _72 => 72,
            _80 => 80,
            _108 => 108,
            _119 => 119,
            _144 => 144,
            _180 => 180,
            _252 => 252,
            _306 => 306,
            _360 => 360,
            _720 => 720,
            _1080 => 1080,
            _1800 => 1800,
            _3600 => 3600,
            _10000 => 10000
        }
    }
}