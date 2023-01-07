use super::game::*;
use super::game::computations::*;
use super::game::payout::PAYOUT_VALUES;

pub async fn recommend_position(game: &Game) -> (usize, String) {
    let mut as_board = game.as_board();
    let (i, data) = compute_best_uncover(&mut as_board).await;
    let (expected_value, _p_data) = parse_data(data);
    (i, format!("Expected Value: {:.2} MGP", expected_value))
}

pub fn recommend_line(game: &Game) -> (usize, String) {
    let (i, data) = compute_best_line(&mut game.as_board());
    let (expected_value, _p_data) = parse_data(data);
    (i, format!("Expected Value: {:.2} MGP", expected_value))
}

fn parse_data(data: [u32; 16]) -> (f64, [f64; 16]) {
    let mut n: f64 = 0.;
    let mut total:f64 = 0.;
    for i in 0..16 {
        n += data[i] as f64;
        total += (data[i] as f64) * (PAYOUT_VALUES[i+1] as f64);
    }
    let mut out_data = [0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.];
    for i in 0..16 {
        out_data[i] = (data[i] as f64) / n;
    }
    (total / n, out_data)
}