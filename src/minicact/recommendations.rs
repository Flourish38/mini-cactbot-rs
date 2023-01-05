use super::game::*;
use super::game::computations::compute_best_line;

pub fn recommend_position(game: &Game) -> (usize, String) {
    (1, "a".to_string())
}

pub fn recommend_line(game: &Game) -> (usize, String) {
    let lines = compute_best_line(&mut game.as_board());
    let max = lines.iter().fold(-1.0, |max, &val| if val > max { val } else { max }); // can't do normal argmax because f64 isn't orderable.....
    // the only way for this unwrap to fail is if there isn't a max value in the array, which is just not possible at all.
    (lines.iter().position(|&x| x == max).unwrap(), format!("Expected Value: {:.2} MGP", max))
}