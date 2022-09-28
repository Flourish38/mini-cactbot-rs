use serenity::builder::CreateComponents;
use serenity::model::prelude::ReactionType;
use serenity::model::prelude::component::ButtonStyle;

pub fn make_numpad_rows(components: &mut CreateComponents) -> &mut CreateComponents {
    components
        .create_action_row(|action_row| {
            action_row
                .create_button(|button| {
                    button.custom_id("numpad_1")
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::try_from("1️⃣").expect("emoji error"))  
                })
                .create_button(|button| {
                    button.custom_id("numpad_2")
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::try_from("2️⃣").expect("emoji error"))  
                })
                .create_button(|button| {
                    button.custom_id("numpad_3")
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::try_from("3️⃣").expect("emoji error"))  
                })
        })
        .create_action_row(|action_row| {
            action_row
                .create_button(|button| {
                    button.custom_id("numpad_4")
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::try_from("4️⃣").expect("emoji error"))  
                })
                .create_button(|button| {
                    button.custom_id("numpad_5")
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::try_from("5️⃣").expect("emoji error"))  
                })
                .create_button(|button| {
                    button.custom_id("numpad_6")
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::try_from("6️⃣").expect("emoji error"))  
                })
        })
        .create_action_row(|action_row| {
            action_row
                .create_button(|button| {
                    button.custom_id("numpad_7")
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::try_from("7️⃣").expect("emoji error"))  
                })
                .create_button(|button| {
                    button.custom_id("numpad_8")
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::try_from("8️⃣").expect("emoji error"))  
                })
                .create_button(|button| {
                    button.custom_id("numpad_9")
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::try_from("9️⃣").expect("emoji error"))  
                })
        })
}