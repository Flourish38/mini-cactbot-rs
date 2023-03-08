use crate::commands::nyi_command;
use crate::generate_components::make_button;
use super::game::*;
use super::generate_components::*;

use serenity::builder::CreateApplicationCommands;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;
use serenity::model::prelude::component::ButtonStyle;

pub async fn handle_command(ctx: Context, command:ApplicationCommandInteraction) -> Result<(), SerenityError> {
    // Add any custom commands here
    match command.data.name.as_str() {
        "minicact_play" => play_command(ctx, command, false).await,
        "minicact_simulate" => play_command(ctx, command, true).await,
        _ => nyi_command(ctx, command).await
    }
}

pub fn create_commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
    // DON'T FORGET to add your custom commands here!!
    commands
        .create_application_command(|command| {
            command.name("minicact_play").description("Play the game!")
        })
        .create_application_command(|command| {
            command.name("minicact_simulate").description("Play a simulated game!")
        })
        
}

async fn play_command(ctx: Context, command: ApplicationCommandInteraction, simulate: bool) -> Result<(), SerenityError> { 
    let mut active_games = ACTIVE_GAMES.lock().await;
    if active_games.contains_key(&command.user.id) {  // if user has an active game already, warn them so they don't lose any data unintentionally.
        return command.create_interaction_response(&ctx.http, |response| {
            response.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(format!("{} you already have a game started.\nWould you like to:\n> ↩ Restore your previous game\n> 🔄 Discard it and start from scratch", command.user.mention()))
                        .ephemeral(true)
                        .components(|components| {
                            components.create_action_row(|action_row| {
                                make_button(action_row, "minicact_restore", ButtonStyle::Primary, Some("↩"), Some(" Restore"), false);
                                make_button(action_row, "minicact_full_reset", ButtonStyle::Primary, Some("🔄"), Some(" Discard"), false)
                            })
                        })
                })
        }).await
    }
    // Otherwise, we're good to go! Just make the default board.
    let game = if simulate {Game::new_simulated()} else {Game::new()};
    command.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::ChannelMessageWithSource)
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
    active_games.insert(command.user.id, game);
    Ok(())
}