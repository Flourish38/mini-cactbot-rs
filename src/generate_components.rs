use crate::game::*;
use crate::game::Action::*;
use crate::game::payout::*;

use serenity::builder::{CreateComponents, CreateActionRow};
use serenity::model::prelude::ReactionType;
use serenity::model::prelude::component::ButtonStyle;

const NUMBER_EMOJI: &'static [&'static str; 10] = &[
    "🟡", "1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣"
];

const POSITION_EMOJI: &'static [&'static str; 10] = &[
    "🟦", "↖", "⬅", "↙", "⬆", "🇽", "⬇", "↗", "➡", "↘"
];

pub fn make_button<'a, D: ToString>(action_row: &'a mut CreateActionRow, custom_id: D, style: ButtonStyle, emoji: Option<&str>, label: Option<&str>) -> &'a mut CreateActionRow  {
    action_row.create_button(|button| {
        button.custom_id(custom_id)
            .style(style);
        if let Some(s) = emoji {
            button.emoji(ReactionType::Unicode(s.to_string()));
        }
        if let Some(s) = label {
            button.label(s);
        }
        button
    })
}

pub fn make_numpad_rows<'a>(components: &'a mut CreateComponents, game: &Game) -> &'a mut CreateComponents {
    for j in 0..3 {
        components.create_action_row(|action_row| {
            for i in (3*j+1)..(3*j+4) {
                if game.used_numbers().contains(&i) {
                    make_button(action_row, format!("X_numpad_{}", i), ButtonStyle::Secondary, None, Some(" "));
                } else {
                    make_button(action_row, format!("numpad_{}", i), ButtonStyle::Primary, Some(NUMBER_EMOJI[<usize as From<u8>>::from(i)]), None);
                }
            }
            action_row
        });
    }
    components
}

pub fn make_game_rows<'a>(components: &'a mut CreateComponents, game: &Game) -> &'a mut CreateComponents {
    for j in 1..4{
        components.create_action_row(|action_row| {
            for i in (j..(j+7)).step_by(3) {  // I'm keeping the column-major ordering from julia for easier integration
                if let Some(k) = game.used_positions().iter().position(|a| a == &i) {
                    make_button(action_row, format!("X_game_{}", i), ButtonStyle::Secondary, Some(NUMBER_EMOJI[<usize as From<u8>>::from(game.used_numbers()[k])]), None);
                } else {
                    make_button(action_row, format!("game_{}", i), ButtonStyle::Primary, Some("🟡"), None);
                }
            }
            action_row
        });
    }
    components
}

pub fn make_reset_bar<'a>(components: &'a mut CreateComponents, game: &Game) -> &'a mut CreateComponents {
    components.create_action_row(|action_row| {
        make_button(action_row, "undo", ButtonStyle::Primary, Some("↩"), None);
        match game.last_action() {
            ChoosePosition(pos) => make_button(action_row, "X_last_input", ButtonStyle::Secondary, Some(POSITION_EMOJI[<usize as From<u8>>::from(pos)]), None),
            RevealNumber(num) => make_button(action_row, "X_last_input", ButtonStyle::Secondary, Some(NUMBER_EMOJI[<usize as From<u8>>::from(num)]), None),
            EnterPayout(p) => make_button(action_row, "X_last_input", ButtonStyle::Secondary, None, Some(<u16 as From<Payout>>::from(p).to_string().as_str())),
            _ => make_button(action_row, "X_last_input", ButtonStyle::Secondary, None, Some(" "))
        };
        make_button(action_row, "reset", ButtonStyle::Primary, Some("🔄"), None)
    })
}