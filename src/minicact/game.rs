pub mod payout;
mod board;
pub mod computations;

use std::collections::HashMap;

use Action::*;

use payout::*;
use payout::Payout::*;
use board::*;

use serenity::model::id::UserId;
use serenity::prelude::*;

use lazy_static::lazy_static;
use smallset::SmallSet;

lazy_static! { pub static ref ACTIVE_GAMES: Mutex<HashMap<UserId, Game>> = Mutex::new(HashMap::new()); }

lazy_static! { static ref PRECOMPUTED_BOARDS: Mutex<HashMap<u32, (usize, [f64; 9])>> = Mutex::new(HashMap::new()); }

pub struct Game {
    index: u8,
    position_history: [u8; 12],
    number_history: [u8; 12],
    payout_history: [Payout; 3],
}

pub enum Action {
    Start,
    ChoosePosition(u8),
    RevealNumber(u8),
    EnterPayout(Payout),
    Done
}

impl Game{
    pub fn new() -> Game {
        Game {
            index: 0,
            position_history: [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
            number_history: [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
            payout_history: [NoPayout, NoPayout, NoPayout], 
        }
    }

    fn index(&self) -> usize {
        self.index.into()
    }

    pub fn next_action(&self) -> Action {
        let i = self.index();
        if i == 12 && self.payout_history[2] != NoPayout {
            Done
        } else if i > 0 && i % 4 == 0 && self.payout_history[i/4 - 1] == NoPayout {
            EnterPayout(NoPayout)
        } else if self.position_history[i] != 255 {
            RevealNumber(255)
        } else {
            ChoosePosition(255)
        }
    }

    pub fn last_action(&self) -> Action {
        let i = self.index();
        if i > 0 && i % 4 == 0 && self.payout_history[i/4 - 1] != NoPayout && (i == 12 || self.position_history[i] == 255) {
            EnterPayout(self.payout_history[i/4 - 1])
        } else if i < 12 && self.position_history[i] != 255 {
            ChoosePosition(self.position_history[i])
        } else if i == 0 {
            Start
        } else {
            RevealNumber(self.number_history[i-1])
        }
    }

    pub fn set_position(&mut self, position: u8) {
        self.position_history[self.index()] = position;
    }

    pub fn set_number(&mut self, number: u8) {
        self.number_history[self.index()] = number;
        self.index += 1;
    }

    pub fn set_payout(&mut self, payout: Payout){
        self.payout_history[self.index()/4 - 1] = payout;
    }

    pub fn undo(&mut self) {
        let i = self.index();
        match self.last_action() {
            EnterPayout(_) => self.payout_history[i/4 - 1] = NoPayout,
            ChoosePosition(_) => self.position_history[i] = 255,
            RevealNumber(_) => {
                self.number_history[i-1] = 255;
                self.index -= 1
            },
            _ => ()
        };
    }

    pub fn used_numbers(&self) -> &[u8] {
        let i = self.index();
        &self.number_history[(if let EnterPayout(_) = self.next_action() {i - 4} else {i - i%4})..i]
    }

    pub fn used_positions(&self) -> &[u8] {
        let i = self.index();
        &self.position_history[(if let EnterPayout(_) = self.next_action() {i - 4} else {i - i%4})..i]
    }

    pub fn total_payout(&self) -> u16 {
        let mut output: u16 = 0;
        for p in self.payout_history {
            output += PAYOUT_VALUES[p as usize];
        }
        output
    }

    pub fn reset(&mut self) {
        self.undo();
        while let Done | ChoosePosition(_) | RevealNumber(_) = self.last_action() {
            self.undo();
        }
    }

    pub fn as_board(&self) -> Board {
        let mut state: [u8; 9] = [255, 255, 255, 255, 255, 255, 255, 255, 255];
        let nums = self.used_numbers();
        let pos = self.used_positions();
        for i in 0..pos.len() {
            state[pos[i] as usize] = nums[i];
        }
        let unused_nums: SmallSet<[u8; 9]> = (0..9).into_iter().filter(|&x| !self.used_numbers().contains(&x)).collect();
        Board { state: state, unused_nums: unused_nums}
    }
}

