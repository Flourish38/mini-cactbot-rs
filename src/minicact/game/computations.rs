use super::board::*;
use super::payout::PAYOUTS;
use super::PRECOMPUTED_BOARDS;
use super::super::generate_components::POSITION_LINE_TABLE;

pub fn compute_best_line(board: &mut Board) -> [f64; 8] {
    let n = board.state.iter().filter(|&x| x != &255).count();
    assert!(n >= 4);
    let first_empty = board.state.iter().position(|&n| n==255);
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
                board.n_unused -= 1;
                board.unused_nums.remove(x);
                let deeper_output = compute_best_line(board);
                for j in 0..8 {
                    output[j] += deeper_output[j];
                }
                board.n_unused += 1;
                board.unused_nums.insert(*x);
            }
            board.state[i] = 255;
            for j in 0..8 {
                output[j] /= board.n_unused as f64;
            }
            output
        }
    }
}