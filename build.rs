use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    time::Instant,
};

use chrono::Local;

const POSITION_LINE_TABLE: [[bool; 9]; 8] = [
    [false, false, false, false, false, false, true, true, true], // bottom row
    [false, false, false, true, true, true, false, false, false], // middle row
    [true, true, true, false, false, false, false, false, false], // top row
    [true, false, false, false, true, false, false, false, true], // \ diagonal
    [true, false, false, true, false, false, true, false, false], // left column
    [false, true, false, false, true, false, false, true, false], // middle column
    [false, false, true, false, false, true, false, false, true], // right column
    [false, false, true, false, true, false, true, false, false], // / diagonal
];

use smallset::SmallSet;

// A Board, to be used for computation. Usually, you will want this to be mutable.
pub struct Board {
    pub state: [u8; 9],
    // A SmallSet is used for easy removing and re-adding of one number at a time.
    pub unused_nums: SmallSet<[u8; 9]>,
}

// all 8 board state transformations that preserve all relevant properties.
// This could probably be an enum somehow, but it doesn't really matter.
const DO_NOTHING: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];
const ROTATE_LEFT: [usize; 9] = [2, 5, 8, 1, 4, 7, 0, 3, 6]; // tHeSe aRe tHe oNlY 2 OpErAtIoNs tHaT ArEn't tHeIr oWn iNvErSe. WhO KnEw 😡😡😡
const ROTATE_RIGHT: [usize; 9] = [6, 3, 0, 7, 4, 1, 8, 5, 2]; // For some reason I thought that I needed to use the reverse operations for computing the recommendations, which was a terrible bug to track down because it only affected these two operations...
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
    // Also returns the operation used to transform the board.
    pub fn simplify(&self) -> (Board, &[usize; 9]) {
        let state = &self.state;
        // hell
        let operation = {
            let corner_min = min_4(state[0], state[2], state[6], state[8]);
            if corner_min != 255 {
                match corner_min {
                    n if n == state[0] => {
                        if state[6] < state[2]
                            || (state[2] == 255 && (state[3] < state[1])
                                || (state[1] == 255 && state[7] < state[5]))
                        {
                            &FLIP_ROTATE_TL
                        } else {
                            &DO_NOTHING
                        }
                    }
                    n if n == state[2] => {
                        if state[0] < state[8]
                            || (state[8] == 255 && (state[1] < state[5])
                                || (state[5] == 255 && state[3] < state[7]))
                        {
                            &FLIP_HORIZONTAL
                        } else {
                            &ROTATE_LEFT
                        }
                    }
                    n if n == state[6] => {
                        if state[8] < state[0]
                            || (state[0] == 255 && (state[7] < state[3])
                                || (state[3] == 255 && state[5] < state[1]))
                        {
                            &FLIP_VERTICAL
                        } else {
                            &ROTATE_RIGHT
                        }
                    }
                    n if n == state[8] => {
                        if state[2] < state[6]
                            || (state[6] == 255 && (state[5] < state[7])
                                || (state[7] == 255 && state[1] < state[3]))
                        {
                            &FLIP_ROTATE_TR
                        } else {
                            &ROTATE_180
                        }
                    }
                    _ => {
                        println!(
                            "Impossible state reached during board.simplify(): corners case.\n{:?}",
                            state
                        );
                        return (
                            Board {
                                state: state.clone(),
                                unused_nums: self.unused_nums.clone(),
                            },
                            &DO_NOTHING,
                        );
                    }
                }
            } else {
                let side_min = min_4(state[1], state[3], state[5], state[7]);
                if side_min != 255 {
                    match side_min {
                        n if n == state[1] => {
                            if state[5] < state[3] {
                                &FLIP_HORIZONTAL
                            } else {
                                &DO_NOTHING
                            }
                        }
                        n if n == state[3] => {
                            if state[1] < state[7] {
                                &FLIP_ROTATE_TL
                            } else {
                                &ROTATE_RIGHT
                            }
                        }
                        n if n == state[5] => {
                            if state[7] < state[1] {
                                &FLIP_ROTATE_TR
                            } else {
                                &ROTATE_LEFT
                            }
                        }
                        n if n == state[7] => {
                            if state[3] < state[5] {
                                &FLIP_VERTICAL
                            } else {
                                &ROTATE_180
                            }
                        }
                        _ => {
                            println!("Impossible state reached during board.simplify(): corners case.\n{:?}", state);
                            return (
                                Board {
                                    state: state.clone(),
                                    unused_nums: self.unused_nums.clone(),
                                },
                                &DO_NOTHING,
                            );
                        }
                    }
                } else {
                    // either just middle or empty board
                    &DO_NOTHING
                }
            }
        };
        // but hey, this part is nice and simple :P
        let mut output = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        for i in 0..9 {
            output[i] = state[operation[i]];
        }
        (
            Board {
                state: output,
                unused_nums: self.unused_nums.clone(),
            },
            operation,
        )
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
    pub fn compress(&self) -> u32 {
        let mut out: u32 = 0;
        let mut num: u8 = 0;
        for pos in 0..9 {
            if self.state[pos] != 255 {
                out |= 1 << (31 - pos);
                out |= (self.state[pos] as u32) << (12 - 4 * num);
                num += 1;
            }
        }
        out
    }
}

// What the value of each Payout is, as a number.
pub const PAYOUT_VALUES: [u16; 17] = [
    0, 36, 54, 72, 80, 108, 119, 144, 180, 252, 306, 360, 720, 1080, 1800, 3600, 10000,
];

// each of these is ONE LESS than an INDEX of the PAYOUT_VALUES array.
// Except for the first 3, since they should be unreachable. I chose 22 because it would stand out, i.e. "how is it 22?? oh."
pub const PAYOUTS: [usize; 22] = [
    22, 22, 22, 15, 0, 11, 10, 3, 8, 4, 2, 1, 7, 2, 7, 5, 0, 9, 12, 6, 13, 14,
];

// returns a usize corresponding to the array index with the max expected return,
// and a [u32; 16] that, when divided by its sum, is a probability distribution over possible payouts (with optimal play).
pub fn compute_best_uncover(
    original_board: &mut Board,
    precomputed_boards: &mut HashMap<u32, (usize, [u32; 16])>,
) -> (usize, [u32; 16]) {
    // Since the dictionary stores the results from the "simplified" board,
    // we need to simplify the board before we store data in the dictionary.
    // `operation` is used to convert back later.
    let (mut board, operation) = original_board.simplify();
    let compressed = board.compress();
    if let Some((out_i, out_data)) = precomputed_boards.get(&compressed) {
        // We have already computed and stored this board! That makes it easy.
        return (operation[*out_i], *out_data);
    }
    // At this point, we can be guaranteed that we are in the precomputation step, since the precomputation step fills the dictionary.
    // In case something goes horribly, horribly wrong though, the function will still compute the result.

    // This is used to know whether we have to compute lines next or not.
    let n = board.state.iter().filter(|&x| x != &255).count();
    let state_clone = board.state.clone();
    let empty_indices = state_clone
        .iter()
        .enumerate()
        .filter(|(_, &x)| x == 255)
        .map(|(x, _)| x);
    // borrow-dodging using the original board instead of the simplified board
    let unused_nums = &original_board.unused_nums;
    // I feel like there must be a better way to do this but I don't really care atm lol
    let mut result: [[u32; 16]; 9] = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    for i in empty_indices {
        for x in unused_nums.iter() {
            board.state[i] = *x;
            board.unused_nums.remove(x);
            match n {
                3 => {
                    let (_, data) = compute_best_line(&mut board);
                    for j in 0..16 {
                        result[i][j] += data[j];
                    }
                }
                _ => {
                    let (_, data) = compute_best_uncover(&mut board, precomputed_boards);
                    for j in 0..16 {
                        result[i][j] += data[j];
                    }
                }
            }
            board.unused_nums.insert(*x);
            // no need to set the state back to an empty tile yet, we're just going to change it again anyways
        }
        board.state[i] = 255;
    }
    if n == 0 {
        // This code path is used during precomputation only.
        // It is used to compute the probability of getting any given payout BEFORE you buy a ticket.
        // In order to do that, we have to add up *all* of the chances together, and take the average.
        // So, instead of returning what the function normally does,
        // This path DOES NOT return a usize intended for indexing.
        // Instead, the usize is what you would divide the [u32; 16] by to convert it into a probability distribution.
        // This is done in minicact.rs
        let mut out_data = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut out = 0;
        for i in 0..9 {
            for j in 0..16 {
                out_data[j] += result[i][j];
                out += result[i][j] as usize;
            }
        }
        // Put this in the dictionary so it does not need to be recomputed at runtime.
        precomputed_boards.insert(compressed, (out, out_data));
        return (out, out_data);
    }
    let mut max_i = 9;
    let mut max = 0;
    for i in 0..9 {
        let mut total: u64 = 0;
        for j in 0..16 {
            total += (result[i][j] as u64) * (PAYOUT_VALUES[j + 1] as u64);
        }
        if total > max {
            max_i = i;
            max = total;
        }
    }
    precomputed_boards.insert(compressed, (max_i, result[max_i]));
    (operation[max_i], result[max_i])
}

// This function, similar to the above function, returns the index of the best line and a distribution over payouts if you choose that line.
pub fn compute_best_line(board: &mut Board) -> (usize, [u32; 16]) {
    let data = compute_best_line_rec(board);
    let mut max_i = 9;
    let mut max = 0;
    for i in 0..8 {
        let mut total: u32 = 0;
        for j in 0..16 {
            total += data[i][j] * (PAYOUT_VALUES[j + 1] as u32);
        }
        if total > max {
            max_i = i;
            max = total;
        }
    }
    (max_i, data[max_i])
}

// This function has to be separate because we can't find the best line until we have gone down all 5 levels of depth and come back up.
// The bot doesn't need distributions for all 8 lines, but we do!
// This function returns a distribution over possible payouts for all 8 line choices.
pub fn compute_best_line_rec(board: &mut Board) -> [[u32; 16]; 8] {
    let first_empty = board.state.iter().position(|&x| x == 255);
    let mut output: [[u32; 16]; 8] = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    match first_empty {
        None => {
            // all spaces are filled (n=9), time to do the computation!
            for i in 0..8 {
                let mut total = 0;
                for j in 0..9 {
                    if POSITION_LINE_TABLE[i][j] {
                        total += board.state[j];
                    }
                }
                // sort of fun to think of that all of the numbers that come out of these functions are built from this line, 1 at a time.
                output[i][PAYOUTS[total as usize]] += 1;
            }
            output
        }
        Some(i) => {
            let unused_nums = board.unused_nums.clone();
            for x in unused_nums.iter() {
                board.state[i] = *x;
                board.unused_nums.remove(x);
                let deeper_output = compute_best_line_rec(board);
                for j in 0..8 {
                    for k in 0..16 {
                        output[j][k] += deeper_output[j][k];
                    }
                }
                board.unused_nums.insert(*x);
            }
            board.state[i] = 255;
            output
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let mut board = Board {
        state: [255, 255, 255, 255, 255, 255, 255, 255, 255],
        unused_nums: (0..9).into_iter().collect(),
    };
    let now = Instant::now();
    // This puts all of the things it computes into the dictionary and returns something that needs a little more computation.
    // explained better in computations.rs
    let mut precomputed_boards = HashMap::new();
    let (_, _) = compute_best_uncover(&mut board, &mut precomputed_boards);
    let elapsed = now.elapsed();
    println!(
        "cargo:warning={:?}\t Computed {} board states in {:.2?}.",
        Local::now(),
        precomputed_boards.len(),
        elapsed
    );
    let mut phf_map = phf_codegen::Map::new();
    for (key, value) in precomputed_boards {
        phf_map.entry(key, format!("{:?}", value).as_str());
    }
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    // println!("cargo:warning={}", path.to_str().unwrap());
    let mut file = BufWriter::new(File::create(&path).unwrap());
    write!(
        &mut file,
        "static PRECOMPUTED_BOARDS: phf::Map<u32, (usize, [u32; 16])> = {};\n",
        phf_map.build()
    )
    .unwrap();
}
