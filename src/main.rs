use dotenv::dotenv;
use rand::Rng;
use std::env;
use tracing::{error, info, warn};

use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        guild::Guild,
        id::GuildId,
        interactions::InteractionData,
        interactions::{ApplicationCommand, Interaction, InteractionResponseType},
    },
    prelude::*,
};

use crate::imgur::{Gallery, ImgurClient};

mod imgur;

struct ImgurGalleryWrapper;

impl TypeMapKey for ImgurGalleryWrapper {
    type Value = Gallery;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        info!("Cache is ready!");
        for guild in guilds {
            info!("Guild: {:?}", guild);
            let _ = guild
                .create_application_command(&ctx.http, |a| {
                    a.name("pepe").description("Get a rare pepe")
                })
                .await;
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if is_new {
            let result = guild
                .create_application_command(&ctx.http, |a| {
                    a.name("pepe").description("Get a rare pepe")
                })
                .await;
            info!("create ApplicationCommand result: {:?}", result);
        }
    }

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
        let interactions = ApplicationCommand::get_global_application_commands(&ctx.http).await;
        info!(
            "I have the following global slash command(s): {:?}",
            interactions
        );
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Some(data) = &interaction.data {
            match data {
                InteractionData::ApplicationCommand(command) => {
                    info!("ApplicationCommand: {:?}", command);
                    match command.name.as_str() {
                        "pepe" => {
                            let data_read = ctx.data.read().await;
                            let gallery: &Gallery = data_read.get::<ImgurGalleryWrapper>().unwrap();
                            let mut rng = rand::rngs::OsRng;
                            let idx = rng.gen_range(0..gallery.images_count);
                            let image_link = &gallery.images[idx as usize].link;

                            let result = interaction
                                .create_interaction_response(&ctx.http, |response| {
                                    response
                                        .kind(InteractionResponseType::ChannelMessageWithSource)
                                        .interaction_response_data(|m| m.content(image_link))
                                })
                                .await;

                            info!("CreateInteractionResponse result: {:?}", result);
                        }
                        _ => {
                            warn!("Unmatched application command name")
                        }
                    }
                }
                InteractionData::MessageComponent(_) => {
                    warn!("Matched MessageComponent")
                }
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
    let mut client = Client::builder(discord_token)
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
