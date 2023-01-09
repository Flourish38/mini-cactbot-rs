use crate::generate_components::make_button;
use super::game::*;
use super::game::Action::*;
use super::generate_components::*;
use super::recommendations::*;
use super::DAILY_PAYOUT_DIST;

use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::message_component::MessageComponentInteraction;
use serenity::prelude::*;

use lazy_static::lazy_static;

use regex::Regex;

use chrono::Local;

pub async fn handle_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    // Add any custom components here
    match component.data.custom_id.as_str() {
        s if s.contains("X") => disabled_component(ctx, component).await,
        "minicact_reset" => reset_component(ctx, component).await,
        "minicact_undo" => undo_component(ctx, component).await,
        "minicact_last_input" => last_input_component(ctx, component).await,
        "minicact_announce_results" => announce_results_component(ctx, component).await,
        "minicact_restore" => restore_component(ctx, component).await,
        "minicact_full_reset" => full_reset_component(ctx, component).await,
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
    }).await.into()
}

async fn create_minicact_response<'a>(component: &MessageComponentInteraction, ctx: &Context, game: &Game) -> Result<(), SerenityError> {
    let action = game.next_action();
    let (recommendation, content) = if let ChoosePosition(_) = action {
        match game.last_action() {
            EnterPayout(_) | Start => (255 as usize, "Enter the already revealed tile:".to_string()),
            _ => recommend_position(&game).await
        }
        
    } else if let EnterPayout(_) = action {
        recommend_line(game)
    } else {
        let opt_i = component.message.content.find(component.user.mention().to_string().as_str());  // finds if the user hit a disabled button last time
        let mut s = component.message.content.clone();
        if let Some(i) = opt_i {
            s.truncate(i)
        };
        (0, s)  // the zero does nothing, because we have guaranteed that we are in the RevealNumber case.
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
    let game = handle_game_mut(active_games.get_mut(&component.user.id), &component, &ctx).await?;
    let action = game.next_action();
    // This is not fun but it never panics!
    match action {
        RevealNumber(_) | ChoosePosition(_) => {
            let num = component.data.custom_id.chars().last()
                .and_then(|x| x.to_digit(10))
                .ok_or(SerenityError::Other("Failed to convert last digit of custom_id during minicact_component??"))? as u8;
            match action {
                RevealNumber(_) if component.data.custom_id.contains("numpad") => game.set_number(num),
                ChoosePosition(_) if component.data.custom_id.contains("game") => game.set_position(num),
                _ => println!("{:?}\t User {} with Id {} desynced on action {:?}. Resyncing...", Local::now(), component.user.name, component.user.id, action)
            }
        },
        EnterPayout(_) if component.data.custom_id.contains("payout") => {
            game.set_payout(component.data.values
                .first().ok_or(SerenityError::Other("Payout component didn't return a value??"))?
                .into());
        },
        _ => println!("{:?}\t User {} with Id {} desynced on action {:?}. Resyncing...", Local::now(), component.user.name, component.user.id, action)
    }

    create_minicact_response(&component, &ctx, game).await
}

async fn reset_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game = handle_game_mut(active_games.get_mut(&component.user.id), &component, &ctx).await?;
    game.reset();
    create_minicact_response(&component, &ctx, game).await
}

async fn undo_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game = handle_game_mut(active_games.get_mut(&component.user.id), &component, &ctx).await?;
    game.undo();
    create_minicact_response(&component, &ctx, game).await
}

async fn last_input_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game = handle_game_mut(active_games.get_mut(&component.user.id), &component, &ctx).await?;
    let total = game.total_payout();
    let daily_payout_dist = DAILY_PAYOUT_DIST.lock().await;
    let percentile = daily_payout_dist.get(&total).ok_or(SerenityError::Other("Somehow total payout is not in daily_payout_dist??"))?.clone();
    drop(daily_payout_dist);
    active_games.remove(&component.user.id);
    component.create_interaction_response(&ctx.http, |response|{
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content(format!("Thanks for using this bot! Feel free to dismiss this message.\nYour total payout is {} MGP, which is {:.2} percentile.", total, percentile))
                    .components(|components| {
                        components.create_action_row(|action_row| {
                            make_button(action_row, "minicact_announce_results", ButtonStyle::Primary, Some("📢"), Some(" Announce your results!"))
                        })
                    })  
            })
    }).await
}

// Don't want to recompile the regex every time, so I made it a static
lazy_static!{static ref MINICACT_REGEX: Regex = Regex::new(r"\s([\d.]+)\s").expect("MINICACT_REGEX errored on creation???????"); }

async fn announce_results_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut captures = MINICACT_REGEX.captures_iter(component.message.content.as_str());
    let total = captures.next().and_then(|x| x.get(1)).ok_or(SerenityError::Other("Couldn't find total in results message!!"))?.as_str();
    let percentile = captures.next().and_then(|x| x.get(1)).ok_or(SerenityError::Other("Couldn't find percentile in results message!!"))?.as_str();
    component.create_interaction_response(&ctx.http, |response|{
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.components(|components| { components })  
            })
    }).await?;
    component.create_followup_message(&ctx.http, |message| {
        message.content(format!("{} earned {} MGP from Mini Cactpot today, which is {} percentile!", component.user.mention(), total, percentile))
    }).await?;
    Ok(())
}

async fn restore_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let active_games = ACTIVE_GAMES.lock().await;
    let game= handle_game(active_games.get(&component.user.id), &component, &ctx).await?;
    create_minicact_response(&component, &ctx, game).await
}

async fn full_reset_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game = handle_game_mut(active_games.get_mut(&component.user.id), &component, &ctx).await?;
    while !matches!(game.last_action(), Start) {
        game.reset();
    }
    create_minicact_response(&component, &ctx, game).await
}

async fn handle_game_mut<'a>(maybe_game: Option<&'a mut Game>, component: &MessageComponentInteraction, ctx: &Context) -> Result<&'a mut Game, SerenityError> {
    match maybe_game {
        Some(game) => Ok(game),
        None => {
            removed_game_response(component, ctx).await?;
            println!("{:?}\t Failed to get game for user {} with Id {} while attempting {}. Probably fine.", Local::now(), component.user.name, component.user.id, component.data.custom_id);
            Err(SerenityError::Other("Failed to get game for user. Probably fine."))
        }
    }
}

async fn handle_game<'a>(maybe_game: Option<&'a Game>, component: &MessageComponentInteraction, ctx: &Context) -> Result<&'a Game, SerenityError> {
    match maybe_game {
        Some(game) => Ok(game),
        None => {
            removed_game_response(component, ctx).await?;
            println!("{:?}\t Failed to get game for user {} with Id {} while attempting {}. Probably fine.", Local::now(), component.user.name, component.user.id, component.data.custom_id);
            Err(SerenityError::Other("Failed to get game for user. Probably fine."))
        }
    }
}

async fn removed_game_response(component: &MessageComponentInteraction, ctx: &Context) -> Result<(), SerenityError> {
    component.create_interaction_response(&ctx.http, |response|{
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content(format!("{} Your game is no longer being tracked, meaning that either you completed it elsewhere or the bot restarted.\nFeel free to dismiss this message.", component.user.mention()))
                    .components(|components| { components })  
            })
    }).await
}