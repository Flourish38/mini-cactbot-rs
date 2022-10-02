use crate::game::*;
use crate::game::Action::*;
use crate::generate_components::*;

use std::time::Instant;

use lazy_static::lazy_static;
use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::message_component::MessageComponentInteraction;
use serenity::prelude::*;

use regex::Regex;

pub async fn handle_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    // Add any custom components here
    match component.data.custom_id.as_str() {
        s if s.starts_with("X") => disabled_component(ctx, component).await,
        s if s.starts_with("numpad_") => numpad_component(ctx, component).await,
        s if s.starts_with("minicact_") => minicact_component(ctx, component).await,
        "reset" => reset_component(ctx, component).await,
        "undo" => undo_component(ctx, component).await,
        "last_input" => last_input_component(ctx, component).await,
        "ping_refresh" => ping_refresh_component(ctx, component).await,
        "announce_results" => announce_results_component(ctx, component).await,
        _ => nyi_component(ctx, component).await
    }
}

async fn nyi_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    component.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|message| {
                message.content("Component interaction not yet implemented.").ephemeral(true)
            })
    })
    .await
}

async fn disabled_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut content = component.user.mention().to_string();
    content.push_str(" Greyed out buttons do not do anything. If you made a mistake, press `Undo` ↩ / `Reset` 🔄");
    component.create_interaction_response(ctx.http, |response| {
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content(content)
            })
    }).await
}

async fn ping_refresh_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let start_time = Instant::now();
    // Use awaiting the defer as a delay to calculate the ping.
    // This gives very inconsistent results, but imo is probably closer to what you want than a heartbeat ping.
    component.defer(&ctx.http).await?;
    let mut duration = start_time.elapsed().as_millis().to_string();
    duration.push_str(" ms");
    // This does not remove the refresh component from the original message.
    component.edit_original_interaction_response(&ctx.http, |response| {
        response.content(duration)
    }).await?;
    Ok(())
}

async fn numpad_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    // let num = component.data.custom_id.chars().last().unwrap(); //.to_digit(10).unwrap();
    component.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content(&component.data.custom_id)
            })
    }).await
}

async fn create_minicact_response<'a>(component: &MessageComponentInteraction, ctx: &Context, game: &Game) -> Result<(), SerenityError> {
    component.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content("")
                    .components(|components| {
                        match game.next_action() {
                            ChoosePosition(_) => {make_game_rows(components, &game);},
                            RevealNumber(_) => {make_numpad_rows(components, &game);},
                            EnterPayout(_) => {make_game_rows(components, &game); make_payout_dropdown(components);},
                            _ => ()
                        }
                        make_reset_bar(components, &game)
                    })  
                })
        }).await
}

async fn minicact_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game: &mut Game = active_games.get_mut(&component.user.id).expect("Game not in active game list!!");
    let mut num = 0;
    let action = game.next_action();
    if let RevealNumber(_) | ChoosePosition(_) = action {
        num = component.data.custom_id.chars().last().unwrap().to_digit(10).unwrap().try_into().unwrap(); // TODO
    }
    
    match action {
        RevealNumber(_) => game.set_number(num),
        ChoosePosition(_) => game.set_position(num),
        EnterPayout(_) => game.set_payout(component.data.values.first().unwrap().into()),
        _ => ()
    };

    create_minicact_response(&component, &ctx, game).await
}

async fn reset_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game: &mut Game = active_games.get_mut(&component.user.id).expect("Game not in active list!!");
    game.reset();
    create_minicact_response(&component, &ctx, game).await
}

async fn undo_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game: &mut Game = active_games.get_mut(&component.user.id).expect("Game not in active list!!");
    game.undo();
    create_minicact_response(&component, &ctx, game).await
}

async fn last_input_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game: &mut Game = active_games.get_mut(&component.user.id).expect("Game not in active list!!");
    let total = game.total_payout();
    active_games.remove(&component.user.id);
    component.create_interaction_response(&ctx.http, |response|{
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content(format!("Thanks for using this bot! Feel free to dismiss this message.\nYour total payout of {} MGP is {} percentile.", total, "123"))
                    .components(|components| {
                        components.create_action_row(|action_row| {
                            make_button(action_row, "announce_results", ButtonStyle::Primary, Some("📢"), Some(" Announce your results!"))
                        })
                    })  
            })
    }).await
}

// Don't want to recompile the regex every time, so I made it a static
lazy_static!{static ref MINICACT_REGEX: Regex = Regex::new(r"\s([0-9.]+)\s").unwrap(); }

async fn announce_results_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut captures = MINICACT_REGEX.captures_iter(component.message.content.as_str());
    let total = captures.next().unwrap().get(1).unwrap().as_str();
    let percentile = captures.next().unwrap().get(1).unwrap().as_str();
    component.create_interaction_response(&ctx.http, |response|{
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content(format!("Thanks for using this bot! Feel free to dismiss this message.\nYour total payout of {} MGP is {} percentile.", total, percentile))
                    .components(|components| { components })  
            })
    }).await?;
    component.create_followup_message(&ctx.http, |message| {
        message.content(format!("{} earned {} MGP from Mini Cactpot today, which is {} percentile!", component.user.mention(), total, percentile))
    }).await?;
    Ok(())
}