use super::game::computations::*;
use super::game::payout::PAYOUT_VALUES;
use super::game::*;

// These functions both work the same. They calculate the best move, and then make the message content.

pub fn recommend_position(game: &Game) -> (usize, String) {
    let mut as_board = game.as_board();
    let (i, data) = compute_best_uncover(&mut as_board);
    let (expected_value, p_data) = parse_data(data);
    (
        i,
        format!(
            "{} Average Payout: {:.2} MGP",
            make_graph(p_data),
            expected_value
        ),
    )
}

pub fn recommend_line(game: &Game) -> (usize, String) {
    let (i, data) = compute_best_line(&mut game.as_board());
    let (expected_value, p_data) = parse_data(data);
    (
        i,
        format!(
            "{} Average Payout: {:.2} MGP",
            make_graph(p_data),
            expected_value
        ),
    )
}

// from an integer distribution, computes the mean payout and the actual probabilities of each payout.
fn parse_data(data: [u32; 16]) -> (f64, [f64; 16]) {
    let mut n: f64 = 0.;
    let mut total: f64 = 0.;
    for i in 0..16 {
        n += data[i] as f64;
        total += (data[i] as f64) * (PAYOUT_VALUES[i + 1] as f64);
    }
    let mut out_data = [
        0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    ];
    for i in 0..16 {
        out_data[i] = (data[i] as f64) / n;
    }
    (total / n, out_data)
}

// Zero-width character in index 0 for p r e c i s i o n
pub const REMAINDER_BARS: [char; 9] = ['​', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

// mmm, graphics...
// makes a unicode bar graph from the payout probabilities.
fn make_graph(data: [f64; 16]) -> String {
    let mut output = "```\n".to_string();
    let max_p = data
        .iter()
        .fold(0., |max, &val| if val > max { val } else { max });
    for (i, p) in data.iter().enumerate().rev() {
        if *p > 0. {
            // Rust has nice formatting options. This could have been a lot worse.
            output.push_str(
                format!(
                    "{:>5} | {} {:.1} %\n",
                    PAYOUT_VALUES[i + 1],
                    make_bar(p, max_p),
                    *p * 100.0
                )
                .as_str(),
            );
        }
    }
    output.push_str("```");
    output
}

// actually makes the bar string from the probability.
// max_p is needed to know how to scale the bar (100% should be full width, but so should 11% if it is the largest bar).
// These bars are as accurate as they can be, even using the 1/8 width bars.
fn make_bar(p: &f64, max_p: f64) -> String {
    let bar_width = (*p / max_p) * 26.0; // 26 characters is the max width that worked on my phone. Should still get feedback from other users.
    let full_bars = bar_width.floor();
    // because this uses .round(), it is basically impervious to floating point rounding errors.
    // If bar_width is x.999, it will add another full bar, and if it's x.000, it will add a zero-width character instead, which give the same bar length.
    let remainder_bar = ((bar_width - full_bars) * 8.0).round(); // This computes how many eighths of a bar are left over after full_bars full bars.
    let mut out_string = "█".repeat(full_bars as usize);
    out_string.push(REMAINDER_BARS[remainder_bar as usize]);
    out_string
}
