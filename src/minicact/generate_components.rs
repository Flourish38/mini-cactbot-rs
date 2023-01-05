use crate::generate_components::make_button;

use super::game::*;
use super::game::Action::*;
use super::game::payout::*;

use serenity::builder::CreateComponents;
use serenity::model::prelude::component::ButtonStyle;

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
    for j in 0..3 {
        components.create_action_row(|action_row| {
            for i in (3*j)..(3*j+3) {
                if game.used_numbers().contains(&i) {
                    make_button(action_row, format!("minicact_X_numpad_{}", i), ButtonStyle::Secondary, None, Some(" "));
                } else {
                    make_button(action_row, format!("minicact_numpad_{}", i), ButtonStyle::Primary, Some(NUMBER_EMOJI[i as usize]), None);
                }
            }
            action_row
        });
    }
    components
}

pub fn make_game_rows<'a>(components: &'a mut CreateComponents, game: &Game, recommendation: usize) -> &'a mut CreateComponents {
    for j in 0..3 {
        components.create_action_row(|action_row| {
            for i in (3*j)..(3*j+3) {
                let payout = matches!(game.next_action(), EnterPayout(_));
                // ugliest nest of if statements ever... but functional!
                let payout_style = if payout && POSITION_LINE_TABLE[recommendation][i as usize] {ButtonStyle::Success} else {ButtonStyle::Secondary};
                if let Some(k) = game.used_positions().iter().position(|a| a == &i) {
                    make_button(action_row, format!("minicact_X_game_{}", i), 
                    payout_style, 
                    Some(NUMBER_EMOJI[game.used_numbers()[k] as usize]), None);
                } else if payout{
                    make_button(action_row, format!("minicact_X_game_{}", i), payout_style, Some("🟡"), None);
                } else {
                    make_button(action_row, format!("minicact_game_{}", i), 
                    if i as usize == recommendation {ButtonStyle::Success} else {ButtonStyle::Primary}, 
                    Some("🟡"), None);
                }
            }
            action_row
        });
    }
    components
}

pub fn make_payout_dropdown<'a>(components: &'a mut CreateComponents) -> &'a mut CreateComponents {
    components.create_action_row(|action_row| {
        action_row.create_select_menu(|menu| {
            menu.custom_id("minicact_payouts")
                .placeholder("Enter your payout!")
                .options(|options| {
                    for i in 1..17 {
                        options.create_option(|option|{
                            option
                                .label(PAYOUT_VALUES[i])
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
        match action {
            Start => make_button(action_row, "minicact_X_undo", ButtonStyle::Secondary, Some("↩"), None),
            _ => make_button(action_row, "minicact_undo", ButtonStyle::Primary, Some("↩"), None)
        };
        match action {
            ChoosePosition(pos) => make_button(action_row, "minicact_X_last_input", ButtonStyle::Secondary, Some(POSITION_EMOJI[pos as usize]), None),
            RevealNumber(num) => make_button(action_row, "minicact_X_last_input", ButtonStyle::Secondary, Some(NUMBER_EMOJI[num as usize]), None),
            EnterPayout(p) => {
                if let Done = game.next_action() {
                    make_button(action_row, "minicact_last_input", ButtonStyle::Success, None, Some(p.to_string().as_str()))
                } else {
                    make_button(action_row, "minicact_X_last_input", ButtonStyle::Secondary, None, Some(p.to_string().as_str()))
                }
                
            },
            _ => make_button(action_row, "minicact_X_last_input", ButtonStyle::Secondary, None, Some(" "))
        };
        make_button(action_row, "minicact_reset", ButtonStyle::Primary, Some("🔄"), None)
    })
}