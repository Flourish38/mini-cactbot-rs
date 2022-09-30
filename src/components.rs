use std::time::Instant;

use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::prelude::interaction::message_component::MessageComponentInteraction;
use serenity::prelude::*;

pub async fn handle_component(ctx: Context, component: MessageComponentInteraction) -> Result<(), SerenityError> {
    // Add any custom components here
    match component.data.custom_id.as_str() {
        s if s.starts_with("X") => disabled_component(ctx, component).await,
        s if s.starts_with("numpad_") => numpad_component(ctx, component).await,
        "ping_refresh" => ping_refresh_component(ctx, component).await,
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