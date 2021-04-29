use mongodb::bson::doc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rowifi_framework::prelude::*;
use sha2::{Digest, Sha512};
use tokio::time::{sleep, Duration};

pub fn api_config(cmds: &mut Vec<Command>) {
    let api_generate_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["generate"])
        .description("Command to generate an API key")
        .handler(api_generate);

    let api_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["api"])
        .description("Module to interact with API keys of the server")
        .group("Administration")
        .sub_command(api_generate_cmd)
        .handler(api_view);

    cmds.push(api_cmd);
}

#[derive(FromArgs)]
pub struct APIArguments {}

pub async fn api_view(ctx: CommandContext, _args: APIArguments) -> CommandResult {
    let embed = EmbedBuilder::new().default_data().title("API Module").unwrap()
        .description("The Module to interact with API keys of the server").unwrap()
        .field(EmbedFieldBuilder::new("Key Generation", "Run `!api generate` to generate a new API key. This key will be direct messaged to you. Please note running this command will invalidate your previous API key").unwrap())
        .build().unwrap();
    ctx.respond().embed(embed).await?;
    Ok(())
}

pub async fn api_generate(ctx: CommandContext, _args: APIArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let server = ctx.bot.cache.guild(guild_id).unwrap();
    let api_key = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect::<String>();

    let mut hasher = Sha512::new();
    hasher.update(api_key.as_bytes());
    let hash = hasher
        .finalize()
        .to_vec()
        .into_iter()
        .map(i32::from)
        .collect::<Vec<_>>();

    if let Ok(channel) = ctx.bot.http.create_private_channel(ctx.author.id).await {
        let msg = ctx.bot
            .http
            .create_message(channel.id)
            .content(format!(
                "Generated API Key for {}: `{}`. This will be deleted in 5 mins. Please make note of this key before it is deleted.",
                server.name, api_key
            ))
            .unwrap()
            .await?;
        let _ = ctx
            .respond()
            .content("The API key has been direct messaged to you")
            .await;
        sleep(Duration::from_secs(5 * 60)).await;
        ctx.bot.http.delete_message(channel.id, msg.id).await?;
    } else {
        ctx.respond()
            .content("I was unable to DM you the API key")
            .await?;
        return Ok(());
    }

    let filter = doc! {"_id": guild_id.0};
    let update = doc! {"$set": {"APIKey": hash}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(())
}
