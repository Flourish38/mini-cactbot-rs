use serenity::builder::{CreateComponents, CreateActionRow};
use serenity::model::prelude::ReactionType;
use serenity::model::prelude::component::ButtonStyle;

pub fn make_button<D: ToString, C: ToString>(action_row: &mut CreateActionRow, custom_id: D, style: ButtonStyle, content: C) -> &mut CreateActionRow  {
    let emoji = emojis::get(content.to_string().as_str());
    action_row.create_button(|button| {
        button.custom_id(custom_id)
            .style(style);
        match emoji {
            Some(_) => button.emoji(ReactionType::Unicode(content.to_string())),
            None => button.label(content)
        }
    })
}

pub fn make_numpad_rows(components: &mut CreateComponents) -> &mut CreateComponents {
    components
        .create_action_row(|action_row| {
            make_button(action_row, "numpad_1", ButtonStyle::Primary, "1️⃣");
            make_button(action_row, "numpad_2", ButtonStyle::Primary, "2️⃣");
            make_button(action_row, "numpad_3", ButtonStyle::Primary, "3️⃣")
        })
        .create_action_row(|action_row| {
            make_button(action_row, "numpad_4", ButtonStyle::Primary, "4️⃣");
            make_button(action_row, "numpad_5", ButtonStyle::Primary, "5️⃣");
            make_button(action_row, "numpad_6", ButtonStyle::Primary, "6️⃣")
        })
        .create_action_row(|action_row| {
            make_button(action_row, "numpad_7", ButtonStyle::Primary, "7️⃣");
            make_button(action_row, "numpad_8", ButtonStyle::Primary, "8️⃣");
            make_button(action_row, "numpad_9", ButtonStyle::Primary, "9️⃣")
        })
}

pub fn make_reset_bar(components: &mut CreateComponents) -> &mut CreateComponents {
    components.create_action_row(|action_row| {
        make_button(action_row, "undo", ButtonStyle::Primary, "↩");
        make_button(action_row, "X_last_input", ButtonStyle::Secondary, " ");
        make_button(action_row, "reset", ButtonStyle::Primary, "🔄")
    })
}