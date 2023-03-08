use crate::generate_components::make_button;

use super::game::*;
use super::game::Action::*;
use super::game::payout::*;
use super::game::computations::compute_best_line;

use rand::seq::index::sample_weighted;
use serenity::builder::CreateComponents;
use serenity::model::prelude::component::ButtonStyle;

use rand::seq::IteratorRandom;

// damn, I miss one-indexing... Julia my beloved D:
const NUMBER_EMOJI: [&'static str; 9] = [
    "1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣"
];

const POSITION_EMOJI: [&'static str; 9] = [
    "↖", "⬆", "↗", "⬅", "🇽", "➡", "↙",  "⬇", "↘"
];

pub const POSITION_LINE_TABLE: [[bool; 9]; 8] = [
    [false, false, false, false, false, false, true, true, true],  // bottom row
    [false, false, false, true, true, true, false, false, false],  // middle row
    [true, true, true, false, false, false, false, false, false],  // top row
    [true, false, false, false, true, false, false, false, true],  // \ diagonal
    [true, false, false, true, false, false, true, false, false],  // left column
    [false, true, false, false, true, false, false, true, false],  // middle column
    [false, false, true, false, false, true, false, false, true],  // right column
    [false, false, true, false, true, false, true, false, false],  // / diagonal
];

pub fn make_numpad_rows<'a>(components: &'a mut CreateComponents, game: &Game) -> &'a mut CreateComponents {
    let chosen_i = if game.is_simulated() {
        (0..9)
            .filter(|x| !game.used_numbers().contains(x))
            .choose(&mut rand::thread_rng())
            .expect("Somehow there are 9 used numbers???")
    } else {255};
    
    for j in 0..3 {
        components.create_action_row(|action_row| {
            for i in (3*j)..(3*j+3) {
                make_button(action_row,
                    format!("minicact_numpad_{:02}_{}", game.index(), i),
                    if i == chosen_i {ButtonStyle::Secondary} else {ButtonStyle::Primary},
                    Some(NUMBER_EMOJI[i as usize]),
                    None,
                    game.used_numbers().contains(&i));
            }
            action_row
        });
    }
    components
}

pub fn make_game_rows<'a>(components: &'a mut CreateComponents, game: &Game, recommendation: usize) -> &'a mut CreateComponents {
    let chosen_i = if game.is_simulated() && matches!(game.last_action(), Start | EnterPayout(_)) {
        (0..9)
            .choose(&mut rand::thread_rng())
            .expect("Failed to choose a number?????")
    } else {255};

    for j in 0..3 {
        components.create_action_row(|action_row| {
            for i in (3*j)..(3*j+3) {
                let payout = matches!(game.next_action(), EnterPayout(_));
                // ugliest nest of if statements ever... but functional!
                // update: this is not even the ugliest nest of if statements in this project anymore. See minicact_component().

                // if payout is true, then recommendation is guaranteed to be valid (i.e. not 255).
                let payout_style = if payout && POSITION_LINE_TABLE[recommendation][i as usize] {ButtonStyle::Success} else {ButtonStyle::Primary};
                if let Some(k) = game.used_positions().iter().position(|a| a == &i) {  // if the game is using position i already.
                    make_button(action_row, 
                        format!("minicact_game_{:02}_{}", game.index(), i), 
                        payout_style, 
                        Some(NUMBER_EMOJI[game.used_numbers()[k] as usize]), 
                        None, 
                        true);  // the emoji corresponding to the number at position i.
                } else if payout{
                    make_button(action_row, 
                        format!("minicact_X_game_{:02}_{}", game.index(), i), 
                        ButtonStyle::Primary, 
                        Some("🟡"), 
                        None, 
                        true);
                } else {
                    make_button(action_row, 
                        format!("minicact_game_{:02}_{}", game.index(), i), 
                    match i{
                        _ if i as usize == recommendation => ButtonStyle::Success,
                        _ if i == chosen_i => ButtonStyle::Secondary,
                        _ => ButtonStyle::Primary
                    }, 
                    Some("🟡"), 
                    None,
                false);
                }
            }
            action_row
        });
    }
    components
}

pub fn make_payout_dropdown<'a>(components: &'a mut CreateComponents, game: &Game) -> &'a mut CreateComponents {
    let chosen_i = if game.is_simulated() {
        let (_, payout_dist) = compute_best_line(&mut game.as_board());
        sample_weighted(&mut rand::thread_rng(), 16, |x| payout_dist[x], 1).expect("No Possible Payouts???").iter().next().expect("No result??") + 1
    } else {255};

    components.create_action_row(|action_row| {
        action_row.create_select_menu(|menu| {
            menu.custom_id(format!("minicact_payouts_{:02}__", game.index())) // double underscore at the end so the index is the same number of characters from the end.
                .placeholder("Enter your payout!")
                .options(|options| {
                    for i in 1..17 {
                        options.create_option(|option|{
                            option
                                .label(format!("{}{}", if chosen_i == i {">"} else {""}, PAYOUT_VALUES[i]))
                                .value(PAYOUT_VALUES[i])
                        });
                    }
                    options
                })
        })
    })
}

pub fn make_reset_bar<'a>(components: &'a mut CreateComponents, game: &Game) -> &'a mut CreateComponents {
    let action = game.last_action();
    components.create_action_row(|action_row| {
        make_button(action_row, 
            "minicact_undo", 
            ButtonStyle::Primary, 
            Some("↩"), 
            None, 
            matches!(action, Start));
        match action {
            ChoosePosition(pos) => make_button(action_row, "minicact_last_input", ButtonStyle::Secondary, Some(POSITION_EMOJI[pos as usize]), None, true),
            RevealNumber(num) => make_button(action_row, "minicact_last_input", ButtonStyle::Secondary, Some(NUMBER_EMOJI[num as usize]), None, true),
            EnterPayout(p) => {
                if let Done = game.next_action() {  // Hey look, the user just took their last action!! Enable the button, which will confirm that they are done.
                    make_button(action_row, "minicact_last_input", ButtonStyle::Success, None, Some(p.to_string().as_str()), false)
                } else {
                    make_button(action_row, "minicact_last_input", ButtonStyle::Secondary, None, Some(p.to_string().as_str()), true)
                }
                
            },
            _ => make_button(action_row, "minicact_last_input", ButtonStyle::Secondary, None, None, true)
        };
        make_button(action_row, "minicact_reset", ButtonStyle::Primary, Some("🔄"), None, false)
    })
}