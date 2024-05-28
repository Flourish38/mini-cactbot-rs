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
        s if s.starts_with("minicact_full_reset") => full_reset_component(ctx, component).await,
        "minicact_restart_simulation" => restart_simulation_component(ctx, component).await,
        _ => minicact_component(ctx, component).await
    }
}

async fn disabled_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    // truncates any errors/warnings that the user might have had
    let opt_i = component.message.content.find(component.user.mention().to_string().as_str());
    let mut s = component.message.content.clone();
    if let Some(i) = opt_i {
       s.truncate(i)
    }
    let content = format!("{}\n{} That button is currently disabled. If you made a mistake, press `Undo` ↩ / `Reset` 🔄", s, component.user.mention());
    component.create_interaction_response(ctx.http, |response| {
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content(content)
            })
    }).await
}

async fn create_minicact_response<'a>(component: &MessageComponentInteraction, ctx: &Context, game: &Game, desync: bool) -> Result<(), SerenityError> {
    let action = game.next_action();
    let (recommendation, mut content) = if let ChoosePosition(_) = action {
        match game.last_action() {
            EnterPayout(_) | Start => (255 as usize, "Enter the already revealed tile:".to_string()), // Can't recommend, haven't seen the first tile yet!
            _ => recommend_position(&game)
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
    if desync {
        content.push_str(format!("\n{} desync detected and fixed. Please double-check that everything is correct.", component.user.mention()).as_str());
    }
    component.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content(content)
                    .components(|components| {
                        match action {
                            ChoosePosition(_) => {make_game_rows(components, &game, recommendation);},
                            RevealNumber(_) => {make_numpad_rows(components, &game);},
                            EnterPayout(_) => {make_game_rows(components, &game, recommendation); make_payout_dropdown(components, &game);},
                            _ => ()  // in the Done case, this means that only the reset_bar will be printed. It handles this specially.
                        }
                        make_reset_bar(components, &game)
                    })  
                })
        }).await
}

async fn minicact_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    // custom_id contains game.index() to catch desyncs
    let (custom_id_index_index, _) = component.data.custom_id.char_indices().rev().nth(3).ok_or(SerenityError::Other("custom_id less than 4 characters???"))?;
    // this only works because I am 100% confident that custom_id is an ASCII string
    let custom_id_index:usize = component.data.custom_id[custom_id_index_index..(custom_id_index_index+2)].parse().map_err(|_| SerenityError::Other("custom_id index failed to parse!!"))?;
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game = handle_game_mut(active_games.get_mut(&component.user.id), &component, &ctx).await?;
    let action = game.next_action();
    if game.index() != custom_id_index {  // Desync guaranteed.
        println!("{:?}\t User {} with Id {} desynced from index {} to {}. Resyncing...", Local::now(), component.user.name, component.user.id, custom_id_index, game.index());
        return create_minicact_response(&component, &ctx, game, true).await
    }
    // This is not fun but it never panics!
    // It was genuinely a massive pain to do this with no .unwrap().
    let mut desync = false;
    match action {
        RevealNumber(_) | ChoosePosition(_) => {
            let num = component.data.custom_id.chars().last()
                .and_then(|x| x.to_digit(10))
                .ok_or(SerenityError::Other("Failed to convert last digit of custom_id during minicact_component??"))? as u8;
            match action {
                RevealNumber(_) if component.data.custom_id.contains("numpad") => game.set_number(num),
                ChoosePosition(_) if component.data.custom_id.contains("game") => game.set_position(num),
                _ => {
                    println!("{:?}\t User {} with Id {} desynced on action {:?}. Resyncing...", Local::now(), component.user.name, component.user.id, action);
                    desync = true;
                }
            }
        },
        EnterPayout(_) if component.data.custom_id.contains("payout") => {
            game.set_payout(component.data.values
                .first().ok_or(SerenityError::Other("Payout component didn't return a value??"))?
                .into());
        },
        _ => {
            println!("{:?}\t User {} with Id {} desynced on action {:?}. Resyncing...", Local::now(), component.user.name, component.user.id, action);
            desync = true;
        }
    }
    // conveniently, even if the user "desyncs" somehow, calling create_minicact_response will show them the correct game state.

    // Now that we have either mutated the board (or not), time to show the user!
    create_minicact_response(&component, &ctx, game, desync).await
}

async fn reset_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game = handle_game_mut(active_games.get_mut(&component.user.id), &component, &ctx).await?;
    game.reset();
    create_minicact_response(&component, &ctx, game, false).await
}

async fn undo_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game = handle_game_mut(active_games.get_mut(&component.user.id), &component, &ctx).await?;
    game.undo();
    create_minicact_response(&component, &ctx, game, false).await
}

// note that the only time this component IS NOT disabled is when the user has played ALL 3 games.
async fn last_input_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game = handle_game_mut(active_games.get_mut(&component.user.id), &component, &ctx).await?;
    let total = game.total_payout();
    let daily_payout_dist = DAILY_PAYOUT_DIST.lock().await;
    let percentile = daily_payout_dist.get(&total).ok_or(SerenityError::Other("Somehow total payout is not in daily_payout_dist??"))?.clone();
    drop(daily_payout_dist);
    let simulated = game.is_simulated();
    active_games.remove(&component.user.id);
    component.create_interaction_response(&ctx.http, |response|{
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content(format!("Thanks for using this bot! Feel free to dismiss this message.\nYour total payout is {} MGP, which is {:.2} percentile.", total, percentile))
                    .components(|components| {
                        components.create_action_row(|action_row| {
                            if simulated {
                                make_button(action_row, 
                                    "minicact_restart_simulation", 
                                    ButtonStyle::Primary, 
                                    Some("🔄"), 
                                    Some(" Play again?"), 
                                    false)
                            } else {
                                make_button(action_row, 
                                    "minicact_announce_results", 
                                    ButtonStyle::Primary, 
                                    Some("📢"), 
                                    Some(" Announce your results!"), 
                                    false)
                            }
                            
                        })
                    })  
            })
    }).await
}

// Don't want to recompile the regex every time, so I made it a static
// Regex that just matches a number surrounded by whitespace, with possible decimal point.
// It technically also matches a period, but all periods in the relevant message are not surrounded by whitespace so it is ok. 
lazy_static!{static ref MINICACT_REGEX: Regex = Regex::new(r"\s([\d.]+)\s").expect("MINICACT_REGEX errored on creation???????"); }

async fn announce_results_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut captures = MINICACT_REGEX.captures_iter(component.message.content.as_str());
    // These use a regex to find the data in the message, since it should already be there. No need to recompute it!
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

// These are both used in the case that the user typed /minicact_play and they already had a game started.

async fn restore_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let active_games = ACTIVE_GAMES.lock().await;
    let game= handle_game(active_games.get(&component.user.id), &component, &ctx).await?;
    create_minicact_response(&component, &ctx, game, false).await
}

async fn full_reset_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    let game = if component.data.custom_id.contains("sim") {Game::new_simulated()} else {Game::new()};
    create_minicact_response(&component, &ctx, &game, false).await?;
    active_games.insert(component.user.id, game);
    Ok(())
}

async fn restart_simulation_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    let mut active_games = ACTIVE_GAMES.lock().await;
    if active_games.contains_key(&component.user.id) {  // if user has an active game already, warn them so they don't lose any data unintentionally.
        return component.create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|message| {
                    message.content(format!("{} you already have a game started.\nWould you like to:\n> ↩ Restore your previous game\n> 🔄 Discard it and start from scratch", component.user.mention()))
                        .ephemeral(true)
                        .components(|components| {
                            components.create_action_row(|action_row| {
                                make_button(action_row, "minicact_restore", ButtonStyle::Primary, Some("↩"), Some(" Restore"), false);
                                make_button(action_row, "minicact_full_reset_sim", ButtonStyle::Primary, Some("🔄"), Some(" Discard"), false)
                            })
                        })
                })
        }).await
    }
    // Otherwise, we're good to go! Just make the default board.
    let game = Game::new_simulated();
    component.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content("Enter the already revealed tile:")
                    .ephemeral(true)
                    .components(|components| {
                        make_game_rows(components, &game, 255);
                        make_reset_bar(components, &game)
                    })
            })
    }).await?;
    // Rust is a beautiful language...
    // I have to make sure that the message returns successfully before I can put the game into active_games.
    active_games.insert(component.user.id, game);
    Ok(())
}

// These are necessary in case the user pushed a component but they did not have a game started.

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

// In the case that the user does not have a game, this lets them know to start a new one instead.
async fn removed_game_response(component: &MessageComponentInteraction, ctx: &Context) -> Result<(), SerenityError> {
    component.create_interaction_response(&ctx.http, |response|{
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|message| {
                message.content(format!("{} Your game is no longer being tracked, meaning that either you completed it elsewhere or the bot restarted.\nFeel free to dismiss this message. Use /minicact_play to start a new game.", component.user.mention()))
                    .components(|components| { components })  
            })
    }).await
}