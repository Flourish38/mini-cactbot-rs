use super::board::*;
use super::payout::PAYOUTS;
use super::PRECOMPUTED_BOARDS;
use super::super::generate_components::POSITION_LINE_TABLE;

use chrono::Local;

use async_recursion::async_recursion;

#[async_recursion]
pub async fn compute_best_uncover(original_board: &mut Board) -> [f64; 9] {
    let (mut board, operation) = original_board.simplify();
    let compressed = board.compress();
    let precomputed_boards = PRECOMPUTED_BOARDS.lock().await;
    if let Some(x) = precomputed_boards.get(&compressed) {
        let mut output = [0., 0., 0., 0., 0., 0., 0., 0., 0.];
        for i in 0..9 {
            output[i] = x[operation[i]];
        }
        return output
    }
    drop(precomputed_boards);
    let n = board.state.iter().filter(|&x| x != &255).count();
    let state_clone = board.state.clone();
    let empty_indices = state_clone.iter().enumerate().filter(|(_, &x)| x == 255).map(|(x, _)| x);
    // borrow-dodging using the original board instead of the simplified board
    let unused_nums = &original_board.unused_nums;
    let mut result = [0., 0., 0., 0., 0., 0., 0., 0., 0.];
    for i in empty_indices {
        for x in unused_nums.iter() {
            board.state[i] = *x;
            board.unused_nums.remove(x);
            result[i] += if n == 3 {  // adding one number means we need to look at the best line next
                compute_best_line(&mut board).iter().fold(-1.0, |max, &val| if val > max {val} else {max})
            } else {
                compute_best_uncover(&mut board).await.iter().fold(-1.0, |max, &val| if val > max {val} else {max})
            };
            board.unused_nums.insert(*x);
        }
        board.state[i] = 255;
        result[i] /= unused_nums.len() as f64;
    }
    let mut output = [0., 0., 0., 0., 0., 0., 0., 0., 0.];
    for i in 0..9 {
        output[i] = result[operation[i]];
    }
    let mut precomputed_boards_2 = PRECOMPUTED_BOARDS.lock().await;
    precomputed_boards_2.insert(compressed, result.clone());
    if n == 1 {
        println!("{:?}\t {}", Local::now(), precomputed_boards_2.len());
    }
    output
}

pub fn compute_best_line(board: &mut Board) -> [f64; 8] {
    let n = board.state.iter().filter(|&x| x != &255).count();
    assert!(n >= 4);
    let first_empty = board.state.iter().position(|&x| x==255);
    match first_empty {
        None => { // all spaces are filled (n=9), time to do the computation!
            let mut output = [0., 0., 0., 0., 0., 0., 0., 0.];
            for i in 0..8 {
                let mut total = 0;
                for j in 0..9 {
                    if POSITION_LINE_TABLE[i][j] {
                        total += board.state[j];
                    }
                }
                output[i] = PAYOUTS[total as usize];
            }
            output
        },
        Some(i) => {
            let unused_nums = board.unused_nums.clone();
            let mut output = [0., 0., 0., 0., 0., 0., 0., 0.];
            for x in unused_nums.iter() {
                board.state[i] = *x;
                board.unused_nums.remove(x);
                let deeper_output = compute_best_line(board);
                for j in 0..8 {
                    output[j] += deeper_output[j];
                }
                board.unused_nums.insert(*x);
            }
            board.state[i] = 255;
            for j in 0..8 {
                output[j] /= unused_nums.len() as f64;
            }
            output
        }
    }
}