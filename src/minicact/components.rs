use crate::generate_components::make_button;
use super::game::*;
use super::game::Action::*;
use super::generate_components::*;
use super::recommendations::*;

use lazy_static::lazy_static;
use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::message_component::MessageComponentInteraction;
use serenity::prelude::*;

use regex::Regex;

pub async fn handle_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    // Add any custom components here
    match component.data.custom_id.as_str() {
        s if s.contains("X") => disabled_component(ctx, component).await,
        "minicact_reset" => reset_component(ctx, component).await,
        "minicact_undo" => undo_component(ctx, component).await,
        "minicact_last_input" => last_input_component(ctx, component).await,
        "minicact_announce_results" => announce_results_component(ctx, component).await,
        _ => minicact_component(ctx, component).await
    }
}

async fn disabled_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let content = if component.message.content.ends_with("🔄") {
        component.message.content.clone()
    } else {
        format!("{}\n{} That button is currently disabled. If you made a mistake, press `Undo` ↩ / `Reset` 🔄", component.message.content, component.user.mention())
    };
    component.create_interaction_response(ctx.http, |response| {
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content(content)
            })
    }).await
}

async fn create_minicact_response<'a>(component: &MessageComponentInteraction, ctx: &Context, game: &Game) -> Result<(), SerenityError> {
    let action = game.next_action();
    let (recommendation, content) = if let ChoosePosition(_) = action {
        recommend_position(&game)
    } else if let EnterPayout(_) = action {
        recommend_line(game)
    } else {
        let opt_i = component.message.content.find(component.user.mention().to_string().as_str());  // finds if the user hit a disabled button last time
        let mut s = component.message.content.clone();
        if let Some(i) = opt_i {
            s.truncate(i)
        };
        (0, s)
    };
    component.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content(content)
                    .components(|components| {
                        match action {
                            ChoosePosition(_) => {make_game_rows(components, &game, recommendation);},
                            RevealNumber(_) => {make_numpad_rows(components, &game);},
                            EnterPayout(_) => {make_game_rows(components, &game, recommendation); make_payout_dropdown(components);},
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
                            make_button(action_row, "minicact_announce_results", ButtonStyle::Primary, Some("📢"), Some(" Announce your results!"))
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