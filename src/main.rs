use dotenv::dotenv;
use rand::Rng;
use std::env;
use tracing::{error, info, warn};

use serenity::async_trait;
use serenity::model::application::command::{Command, CommandOptionType};
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::imgur::{Gallery, ImgurClient};

mod imgur;
mod meme;

struct ImgurGalleryWrapper;

impl TypeMapKey for ImgurGalleryWrapper {
    type Value = Gallery;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!pepe" {
            let data_read = ctx.data.read().await;
            let gallery: &Gallery = data_read.get::<ImgurGalleryWrapper>().unwrap();
            let mut rng = rand::rngs::OsRng;
            let idx = rng.gen_range(0..gallery.images_count);
            let image_link = &gallery.images[idx as usize].link;

            if let Err(why) = msg.channel_id.say(&ctx.http, image_link).await {
                error!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        let interactions = Command::get_global_application_commands(&ctx.http).await;
        info!(
            "I have the following global slash command(s): {:?}",
            interactions
        );

        let _ = Command::create_global_application_command(&ctx.http, |command| {
            command.name("pepe").description("Get a rare pepe")
        })
        .await;

        let _ = Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("meme")
                .description("Get a crispy meme")
                .create_option(|option| {
                    option
                        .name("subreddit")
                        .description("Subreddit to get a meme from")
                        .kind(CommandOptionType::String)
                        .required(false)
                })
        })
        .await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command.data.name);

            let content = match command.data.name.as_str() {
                "pepe" => {
                    let data_read = ctx.data.read().await;
                    let gallery: &Gallery = data_read.get::<ImgurGalleryWrapper>().unwrap();
                    let mut rng = rand::rngs::OsRng;
                    let idx = rng.gen_range(0..gallery.images_count);
                    let image_link = &gallery.images[idx as usize].link;
                    image_link.to_owned()
                }
                "meme" => {
                    let mut subreddit = None;

                    if let Some(option) = command.data.options.get(0) {
                        if let Some(CommandDataOptionValue::String(subreddit_option)) = &option.resolved {
                            subreddit = Some(subreddit_option);
                        }
                    }

                    match meme::gimme(subreddit).await {
                        Ok(meme) => meme.url,
                        Err(_e) => "Error.".to_string(),
                    }
                }
                _ => {
                    let content = "Unmatched application command name".to_string();
                    warn!("Warning: {}", content);
                    content
                }
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    // Configure the client with your Discord bot token in the environment.
    let discord_token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let application_id = env::var("APPLICATION_ID").expect("Expected a token in the environment");
    let imgur_client_id = env::var("IMGUR_CLIENT_ID").expect("Expected a token in the environment");

    // The Application Id is usually the Bot User Id.
    let application_id: u64 = application_id
        .parse()
        .expect("application id is not a valid id");

    // Build our client.
    let mut client = Client::builder(discord_token, GatewayIntents::empty())
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    {
        let imgur_client = ImgurClient::new(imgur_client_id);
        let gallery = imgur_client.get_gallery(String::from("SU4Qa")).await;
        let mut data = client.data.write().await;
        data.insert::<ImgurGalleryWrapper>(gallery.unwrap());
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
