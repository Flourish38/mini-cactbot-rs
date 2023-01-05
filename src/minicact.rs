pub mod commands;
pub mod components;
mod game;
mod generate_components;
mod recommendations;

use game::board::Board;
use game::computations::compute_best_uncover;
use game::PRECOMPUTED_BOARDS;

use std::time::Instant;

pub async fn startup() {
    let mut board = Board { 
        state: [255, 255, 255, 255, 255, 255, 255, 255, 255],
        unused_nums: (0..9).into_iter().collect()
    };
    println!("{:?}\t Computing all possible boards...", Instant::now());
    let now = Instant::now();
    compute_best_uncover(&mut board).await;
    let elapsed = now.elapsed();
    let precomputed_boards = PRECOMPUTED_BOARDS.lock().await;
    println!("{:?}\t Computed {} board states in {:.2?}.", Instant::now(), precomputed_boards.len(), elapsed);
}