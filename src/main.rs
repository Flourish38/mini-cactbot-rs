// based on https://github.com/serenity-rs/serenity/blob/current/examples/e14_slash_commands/src/main.rs
// you **shouldn't** need to modify this file at all, unless you want to use an interaction other than commands and components.
// in that case, modify interaction_create below and create a separate module for it in another file.

mod commands;
mod components;
mod generate_components;
mod minicact;
mod startup;

use commands::*;
use components::*;
use serenity::model::user::OnlineStatus;
use startup::startup;

use std::env;

use tokio::sync::mpsc;

use serenity::{async_trait, model::prelude::Activity};
use serenity::model::application::command::Command;
use serenity::model::application::interaction::Interaction;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;
use serenity::prelude::*;

use config::{Config, File, ConfigError};

use chrono::Local;

use lazy_static::lazy_static;

// Technically this initial vec is never used but it makes it so you don't need to use an expect() whenever you use the variable.
// Also, according to the docs, vecs of size 0 don't allocate any memory anyways, so it literally doesn't matter.
lazy_static! { static ref ADMIN_USERS: Mutex<Vec<UserId>> = Mutex::new(vec![]); }

// Unused by default, but useful in case you need it.
// If you put `use crate::CONFIG;` in another file, it will include this, and you will have access to the raw config values for your own use.
lazy_static! { static ref CONFIG: Mutex<Config> = Mutex::new(Config::default()); }

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                // Commands are implemented in src/commands.rs
                if let Err(why) = handle_command(ctx, command).await {
                    println!("{:?}\t Cannot respond to slash command: {}", Local::now(), why);
                };
            },
            Interaction::MessageComponent(component) => {
                // Components are implemented in src/components.rs
                if let Err(why) = handle_component(ctx, component).await {
                    println!("{:?}\t Cannot respond to message component: {}", Local::now(), why);
                }
            },
            _ => println!("{:?}\t Unimplemented interaction: {}", Local::now(), interaction.kind().num())
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{:?}\t {} is connected!", Local::now(), ready.user.name);
        ctx.set_presence(Some(Activity::playing("using /minicact_play")), OnlineStatus::Online).await;  // pretty awesome that this doesn't return errors.
        Command::set_global_application_commands(&ctx.http, create_commands)
        .await.expect("Failed to set application commands");
    }
}

fn build_config() -> Result<Config, ConfigError> {
    Config::builder()
        .add_source(File::with_name("config"))
        .set_default("admins", Vec::<u64>::new())?
        .set_override_option("token", env::var("DISCORD_TOKEN").ok())?
        .build()
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in your `config` file.
    let config = build_config().expect("Config failed");

    let token = config.get_string("token").expect("Token not found. Either:\n
                                                                    - put it in the `config` file (token = \"token\")\n
                                                                    - set environment variable DISCORD_TOKEN.\n");

    let admins = config.get_array("admins")
                                    .expect("Somehow failed to get admin list even though there is a default value??")
                                    .iter().map(|val| {
                                        UserId(val.clone().into_uint().expect("Failed to parse admin list entry into UserId"))
                                    }).collect::<Vec<UserId>>();

    if admins.is_empty() {
        println!("{:?}\t WARNING: No admin users specified! By default, any user will be able to shut down your bot.", Local::now());
    }

    *ADMIN_USERS.lock().await = admins;

    *CONFIG.lock().await = config;

    // Build our client.
    let mut client = Client::builder(token.clone(), GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Channel for the shutdown command to use later
    let (sender, mut receiver) = mpsc::channel(64);
    *SHUTDOWN_SENDER.lock().await = Some(sender);

    let shard_manager = client.shard_manager.clone();

    // Spawns a task that waits for the shutdown command, then shuts down the bot.
    tokio::spawn(async move {
        loop {
            // If this returns None, then something is wrong so we want to panic
            let b = receiver.recv().await.expect("Shutdown message pass error");
            // I have left open the possibility of using b=false for something "softer" in case you need it.
            if b {
                shard_manager.lock().await.shutdown_all().await;
                println!("{:?}\t Shutdown shard manager", Local::now());
                break;
            }
        }
    });

    startup().await;

    // Start the client.
    match client.start().await {
        Err(why) => println!("{:?}\t Client error: {}. Double-check your token, which is currently \"{}\".", Local::now(), why, token),
        Ok(_) => println!("{:?}\t Client shutdown cleanly", Local::now())
    }
    
}