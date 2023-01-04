use serenity::builder::CreateActionRow;
use serenity::model::prelude::ReactionType;
use serenity::model::prelude::component::ButtonStyle;

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