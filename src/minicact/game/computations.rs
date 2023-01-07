use super::board::*;
use super::payout::{PAYOUTS, PAYOUT_VALUES};
use super::PRECOMPUTED_BOARDS;
use super::super::generate_components::POSITION_LINE_TABLE;

use chrono::Local;

use async_recursion::async_recursion;

#[async_recursion]
pub async fn compute_best_uncover(original_board: &mut Board) -> (usize, [u32; 16]) {
    let (mut board, operation) = original_board.simplify();
    let compressed = board.compress();
    let precomputed_boards = PRECOMPUTED_BOARDS.lock().await;
    if let Some((out_i, out_data)) = precomputed_boards.get(&compressed) {
        return (operation[*out_i], *out_data)
    }
    drop(precomputed_boards);
    let n = board.state.iter().filter(|&x| x != &255).count();
    let state_clone = board.state.clone();
    let empty_indices = state_clone.iter().enumerate().filter(|(_, &x)| x == 255).map(|(x, _)| x);
    // borrow-dodging using the original board instead of the simplified board
    let unused_nums = &original_board.unused_nums;
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
        }
        board.state[i] = 255;
    }
    let mut out_data;
    let mut out_i;
    if n == 0 {
        out_data = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        out_i = 0;
        for i in 0..9 {
            for j in 0..16 {
                out_data[j] += result[i][j];
                out_i += result[i][j] as usize;
            }
        }
    } else {
        out_i = 9;
        let mut max = 0;
        for i in 0..9 {
            let mut total:u64 = 0;
            for j in 0..16 {
                total += (result[i][j] as u64) * (PAYOUT_VALUES[j+1] as u64);
            }
            if total > max {
                out_i = i;
                max = total;
            }
        }
        out_data = result[out_i];
    }
    let mut precomputed_boards_2 = PRECOMPUTED_BOARDS.lock().await;
    precomputed_boards_2.insert(compressed, (out_i, out_data));
    if n == 1 {
        println!("{:?}\t {}", Local::now(), precomputed_boards_2.len());
    }
    // if n == 0, we return the sum of out_data instead. This is ok because you never recommend a tile to uncover there anyways :P
    (if n == 0 { out_i } else { operation[out_i] }, out_data)
}

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