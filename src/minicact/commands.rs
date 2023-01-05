use crate::commands::{nyi_command, send_interaction_response_message};
use super::game::*;
use super::generate_components::*;

use serenity::builder::CreateApplicationCommands;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::*;

pub async fn handle_command(ctx: Context, command:ApplicationCommandInteraction) -> Result<(), SerenityError> {
    // Add any custom commands here
    match command.data.name.as_str() {
        "minicact_play" => play_command(ctx, command).await,
        _ => nyi_command(ctx, command).await
    }
}

pub fn create_commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
    // DON'T FORGET to add your custom commands here!!
    commands
        .create_application_command(|command| {
            command.name("minicact_play").description("Play the game!")
        })
}

async fn play_command(ctx: Context, command: ApplicationCommandInteraction) -> Result<(), SerenityError> { 
    let mut active_games = ACTIVE_GAMES.lock().await;
    if active_games.contains_key(&command.user.id) {
        return send_interaction_response_message(&ctx, &command, "You already have a game started!", true).await
    }
    let game = Game::new();
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