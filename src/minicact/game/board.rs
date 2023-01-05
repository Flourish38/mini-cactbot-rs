use smallset::SmallSet;

pub struct Board {
    pub state: [u8; 9],
    pub unused_nums: SmallSet<[u8; 9]>
}

// all 8 board state transformations that preserve all relevant properties.
const DO_NOTHING: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];
const ROTATE_LEFT: [usize; 9] = [2, 5, 8, 1, 4, 7, 0, 3, 6];
const ROTATE_RIGHT: [usize; 9] = [6, 3, 0, 7, 4, 1, 8, 5, 2];
const ROTATE_180: [usize; 9] = [8, 7, 6, 5, 4, 3, 2, 1, 0];
const FLIP_HORIZONTAL: [usize; 9] = [2, 1, 0, 5, 4, 3, 8, 7, 6];
const FLIP_VERTICAL: [usize; 9] = [6, 7, 8, 3, 4, 5, 0, 1, 2];
const FLIP_ROTATE_TL: [usize; 9] = [0, 3, 6, 1, 4, 7, 2, 5, 8];
const FLIP_ROTATE_TR: [usize; 9] = [8, 5, 2, 7, 4, 1, 6, 3, 0];

fn min_4(x1: u8, x2: u8, x3: u8, x4: u8) -> u8 {
    x1.min(x2).min(x3).min(x4)
}

impl Board {
    // processes the board so it will always be in the same orientation.
    // This reduces the number of precomputed boards by a factor of something >5.
    // Also returns the inverse operation needed to reverse the transformation.
    fn simplify(&self) -> (Board, &[usize; 9]) {
        let state = &self.state;
        // hell
        let operation = {
            let corner_min = min_4(state[0], state[2], state[6], state[8]);
            if corner_min != 255 {
                match corner_min {
                    n if n == state[0] => if state[6] < state[2] || (state[2] == 255 && (state[3] < state[1]) || (state[1] == 255 && state[7] < state[5])) {&FLIP_ROTATE_TL} else {&DO_NOTHING},
                    n if n == state[2] => if state[0] < state[8] || (state[8] == 255 && (state[1] < state[5]) || (state[5] == 255 && state[3] < state[7])) {&FLIP_HORIZONTAL} else {&ROTATE_LEFT},
                    n if n == state[6] => if state[8] < state[0] || (state[0] == 255 && (state[7] < state[3]) || (state[3] == 255 && state[5] < state[1])) {&FLIP_VERTICAL} else {&ROTATE_RIGHT},
                    n if n == state[8] => if state[2] < state[6] || (state[6] == 255 && (state[5] < state[7]) || (state[7] == 255 && state[1] < state[3])) {&FLIP_ROTATE_TR} else {&ROTATE_180},
                    _ => panic!("Impossible state reached")
                }
            } else {
                let side_min = min_4(state[1], state[3], state[5], state[7]);
                if side_min != 255 {
                    match side_min {
                        n if n == state[1] => if state[5] < state[3] {&FLIP_HORIZONTAL} else {&DO_NOTHING},
                        n if n == state[3] => if state[1] < state[7] {&FLIP_ROTATE_TL} else {&ROTATE_RIGHT},
                        n if n == state[5] => if state[7] < state[1] {&FLIP_ROTATE_TR} else {&ROTATE_LEFT},
                        n if n == state[7] => if state[3] < state[5] {&FLIP_VERTICAL} else {&ROTATE_180},
                        _ => panic!("Impossible state reached")
                    }
                } else {  // either just middle or nothing
                    &DO_NOTHING
                }
            } 
        };
        // but hey, this part is nice and simple :P
        let mut output = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        for i in 0..9 {
            output[i] = state[operation[i]];
        }
        (Board { state: output, unused_nums: self.unused_nums.clone() }, match operation {
            &ROTATE_LEFT => &ROTATE_RIGHT,  // these are the only 2 operations that aren't their own inverse. who knew
            &ROTATE_RIGHT => &ROTATE_LEFT,
            o => o
        })
    }

    // compresses the board state into a u32. The format is 9 bits of a mask for each position, then 7 unused bits, 
    // then 4 groups of 4 bits corresponding to the numbers in each position, IN ORDER.
    // for example, the board [0, 255, 255, 3, 255, 6, 255, 8, 255]
    // would be compressed as 0b 100101010 0000000 0000 0011 0110 1000 = 0x95000368.
    // the board [3, 0, 255, 255, 255, 255, 255, 255, 255]
    // would be compressed as 110000000 0000000 0011 0000 0000 0000.
    // Note that those last TWO (out of three!) groups of 4 zeros do not correspond to an actual number.
    // While I am documenting the format, in general you should never decompress these. Think of it more as an unscrambled hash function.
    // it is HIGHLY recommended you do not call this on unsimplified boards.
    fn compress(&self) -> u32 {
        let mut out: u32 = 0;
        let mut num:u8 = 0;
        for pos in 0..9 {
            if self.state[pos] != 255 {
                out |= 1 << (31 - pos);
                out |= (self.state[pos] as u32) << (12 - 4*num);
                num += 1;
            }
        }
        out
    }

    pub fn simplify_compress(&self) -> (u32, &[usize; 9]) {
        let state = &self.state;
        // hell
        let operation = {
            let corner_min = min_4(state[0], state[2], state[6], state[8]);
            if corner_min != 255 {
                match corner_min {
                    n if n == state[0] => if state[6] < state[2] || (state[2] == 255 && (state[3] < state[1]) || (state[1] == 255 && state[7] < state[5])) {&FLIP_ROTATE_TL} else {&DO_NOTHING},
                    n if n == state[2] => if state[0] < state[8] || (state[8] == 255 && (state[1] < state[5]) || (state[5] == 255 && state[3] < state[7])) {&FLIP_HORIZONTAL} else {&ROTATE_LEFT},
                    n if n == state[6] => if state[8] < state[0] || (state[0] == 255 && (state[7] < state[3]) || (state[3] == 255 && state[5] < state[1])) {&FLIP_VERTICAL} else {&ROTATE_RIGHT},
                    n if n == state[8] => if state[2] < state[6] || (state[6] == 255 && (state[5] < state[7]) || (state[7] == 255 && state[1] < state[3])) {&FLIP_ROTATE_TR} else {&ROTATE_180},
                    _ => panic!("Impossible state reached")
                }
            } else {
                let side_min = min_4(state[1], state[3], state[5], state[7]);
                if side_min != 255 {
                    match side_min {
                        n if n == state[1] => if state[5] < state[3] {&FLIP_HORIZONTAL} else {&DO_NOTHING},
                        n if n == state[3] => if state[1] < state[7] {&FLIP_ROTATE_TL} else {&ROTATE_RIGHT},
                        n if n == state[5] => if state[7] < state[1] {&FLIP_ROTATE_TR} else {&ROTATE_LEFT},
                        n if n == state[7] => if state[3] < state[5] {&FLIP_VERTICAL} else {&ROTATE_180},
                        _ => panic!("Impossible state reached")
                    }
                } else {  // either just middle or nothing
                    &DO_NOTHING
                }
            } 
        };
        // but hey, this part is nice and simple :P
        let mut simple_state = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        for i in 0..9 {
            simple_state[i] = state[operation[i]];
        }

        // compress part
        let mut out: u32 = 0;
        let mut num:u8 = 0;
        for pos in 0..9 {
            if simple_state[pos] != 255 {
                out |= 1 << (31 - pos);
                out |= (self.state[pos] as u32) << (12 - 4*num);
                num += 1;
            }
        }
        (out, match operation {
            &ROTATE_LEFT => &ROTATE_RIGHT,  // these are the only 2 operations that aren't their own inverse. who knew
            &ROTATE_RIGHT => &ROTATE_LEFT,
            o => o
        })
    }
}
