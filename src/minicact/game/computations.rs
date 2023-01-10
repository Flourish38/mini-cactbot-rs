use super::board::*;
use super::payout::{PAYOUTS, PAYOUT_VALUES};
use super::PRECOMPUTED_BOARDS;
use super::super::generate_components::POSITION_LINE_TABLE;

use chrono::Local;

use async_recursion::async_recursion;

// returns a usize corresponding to the array index with the max expected return,
// and a [u32; 16] that, when divided by its sum, is a probability distribution over possible payouts (with optimal play).
#[async_recursion]
pub async fn compute_best_uncover(original_board: &mut Board) -> (usize, [u32; 16]) {
    // Since the dictionary stores the results from the "simplified" board,
    // we need to simplify the board before we store data in the dictionary.
    // `operation` is used to convert back later.
    let (mut board, operation) = original_board.simplify();
    let compressed = board.compress();
    let precomputed_boards = PRECOMPUTED_BOARDS.lock().await;
    if let Some((out_i, out_data)) = precomputed_boards.get(&compressed) {  // We have already computed and stored this board! That makes it easy.
        return (operation[*out_i], *out_data)
    }
    // If we don't drop this here, we would need some sort of re-entrant lock (since this function is recursive), which I don't know if it is by default.
    // Either way, this is probably good practice.
    drop(precomputed_boards);
    // At this point, we can be guaranteed that we are in the precomputation step, since the precomputation step fills the dictionary.
    // In case something goes horribly, horribly wrong though, the function will still compute the result.

    // This is used to know whether we have to compute lines next or not.
    let n = board.state.iter().filter(|&x| x != &255).count();
    let state_clone = board.state.clone();
    let empty_indices = state_clone.iter().enumerate().filter(|(_, &x)| x == 255).map(|(x, _)| x);
    // borrow-dodging using the original board instead of the simplified board
    let unused_nums = &original_board.unused_nums;
    // I feel like there must be a better way to do this but I don't really care atm lol
    let mut result:[[u32; 16]; 9] = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
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
                },
                _ => {
                    let (_, data) = compute_best_uncover(&mut board).await;
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
    if n == 0 {  // This code path is used during precomputation only.
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
        // We do not want particularly this to be in the dictionary, as it actually means something else.
        return (out, out_data)
    }
    let mut max_i = 9;
    let mut max = 0;
    for i in 0..9 {
        let mut total:u64 = 0;
        for j in 0..16 {
            total += (result[i][j] as u64) * (PAYOUT_VALUES[j+1] as u64);
        }
        if total > max {
            max_i = i;
            max = total;
        }
    }
    let mut precomputed_boards_2 = PRECOMPUTED_BOARDS.lock().await;
    precomputed_boards_2.insert(compressed, (max_i, result[max_i]));
    if n == 1 {  // This only happens during precomputation.
        println!("{:?}\t {}", Local::now(), precomputed_boards_2.len());
    }
    (operation[max_i], result[max_i])
}

// This function, similar to the above function, returns the index of the best line and a distribution over payouts if you choose that line.
pub fn compute_best_line(board: &mut Board) -> (usize, [u32; 16]) {
    let data = compute_best_line_rec(board);
    let mut max_i = 9;
    let mut max = 0;
    for i in 0..8 {
        let mut total:u32 = 0;
        for j in 0..16 {
            total += data[i][j] * (PAYOUT_VALUES[j+1] as u32);
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
pub fn compute_best_line_rec(board: &mut Board)  -> [[u32; 16]; 8] {
    let first_empty = board.state.iter().position(|&x| x==255);
    let mut output:[[u32; 16]; 8] = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    ];
    match first_empty {
        None => { // all spaces are filled (n=9), time to do the computation!
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
        },
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