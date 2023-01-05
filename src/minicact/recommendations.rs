use super::game::*;
use super::game::computations::*;

pub async fn recommend_position(game: &Game) -> (usize, String) {
    let mut as_board = game.as_board();
    let spots = compute_best_uncover(&mut as_board).await;
    let max = spots.iter().fold(-1.0, |max, &val| if val > max { val } else { max });
    (spots.iter().position(|&x| x == max).unwrap(), format!("Expected Value: {:.2} MGP", max))
}

pub fn recommend_line(game: &Game) -> (usize, String) {
    let lines = compute_best_line(&mut game.as_board());
    let max = lines.iter().fold(-1.0, |max, &val| if val > max { val } else { max }); // can't do normal argmax because f64 isn't orderable.....
    // the only way for this unwrap to fail is if there isn't a max value in the array, which is just not possible at all.
    (lines.iter().position(|&x| x == max).unwrap(), format!("Expected Value: {:.2} MGP", max))
}