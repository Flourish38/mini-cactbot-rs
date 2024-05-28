pub mod commands;
pub mod components;
mod game;
mod generate_components;
mod recommendations;

use game::board::Board;
use game::computations::compute_best_uncover;

use std::collections::BTreeMap;
use std::time::Instant;

use tokio::sync::Mutex;

use chrono::Local;

use lazy_static::lazy_static;

use crate::minicact::game::payout::PAYOUT_VALUES;

// BTreeMap because it's sorted by default, which is useful here.
lazy_static! {
    pub static ref DAILY_PAYOUT_DIST: Mutex<BTreeMap<u16, f64>> = Mutex::new(BTreeMap::new());
}

pub async fn startup() {
    let mut board = Board {
        state: [255, 255, 255, 255, 255, 255, 255, 255, 255],
        unused_nums: (0..9).into_iter().collect(),
    };
    let (n, data) = compute_best_uncover(&mut board);
    // time to compute the probability of any given payout BEFORE you buy your scratch ticket, with optimal play.
    let mut p_data = [
        0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    ];
    for i in 0..16 {
        p_data[i] = (data[i] as f64) / (n as f64);
    }
    let mut daily_payout_dist = DAILY_PAYOUT_DIST.lock().await;
    // now, we just need to ""cube"" that probability distribution, to get all the combinations that give any given payout for the day.
    let mut now = Instant::now();
    for i in 0..16 {
        for j in 0..16 {
            for k in 0..16 {
                // The key is the value, so we can get the percentile from the payout.
                let key = PAYOUT_VALUES[i + 1] + PAYOUT_VALUES[j + 1] + PAYOUT_VALUES[k + 1];
                let oldvalue = daily_payout_dist.get(&key).unwrap_or(&0.).clone();
                daily_payout_dist.insert(key, p_data[i] * p_data[j] * p_data[k] + oldvalue);
            }
        }
    }
    let mut elapsed = now.elapsed();
    println!(
        "{:?}\t Computed {} payout options in {:.2?}.",
        Local::now(),
        daily_payout_dist.len(),
        elapsed
    );
    let mut total_p = 0.;
    now = Instant::now();
    // This works because BTreeMap produces sorted iters.
    for (_, v) in daily_payout_dist.iter_mut() {
        total_p += *v * 100.; // convert to percentile
        *v = total_p;
    }
    elapsed = now.elapsed();
    println!(
        "{:?}\t Updated {} payout percentiles in {:.2?}.",
        Local::now(),
        daily_payout_dist.len(),
        elapsed
    );
}
