use crate::framework::prelude::*;
use crate::models::user::PremiumUser;

pub static PREMIUM_TRANSFER_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["transfer"],
    desc: Some("Command to transfer your premium"),
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static PREMIUM_TRANSFER_COMMAND: Command = Command {
    fun: premium_transfer,
    options: &PREMIUM_TRANSFER_OPTIONS,
};

#[command]
pub async fn premium_transfer(
    ctx: &Context,
    msg: &Message,
    mut args: Arguments<'fut>,
) -> CommandResult {
    let premium_user = ctx.database.get_premium(msg.author.id.0).await?;
    if let Some(premium_user) = premium_user {
        if premium_user.premium_owner.is_some() {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Premium Transfer Failed")
                .unwrap()
                .description("You may not transfer a premium that you do not own")
                .unwrap()
                .build()
                .unwrap();
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
        let to_transfer_id = match args.next().map(|a| a.parse::<i64>()) {
            Some(Ok(s)) => s,
            _ => {
                let embed = EmbedBuilder::new()
                    .default_data()
                    .color(Color::Red as u32)
                    .unwrap()
                    .title("Premium Transfer Failed")
                    .unwrap()
                    .description("You must specify a user id to transfer to")
                    .unwrap()
                    .build()
                    .unwrap();
                let _ = ctx
                    .http
                    .create_message(msg.channel_id)
                    .embed(embed)
                    .unwrap()
                    .await?;
                return Ok(());
            }
        };
        if ctx
            .database
            .get_premium(to_transfer_id as u64)
            .await?
            .is_some()
        {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Premium Transfer Failed")
                .unwrap()
                .description("You cannot transfer premium to a user who already has premium")
                .unwrap()
                .build()
                .unwrap();
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }

        let new_premium_user = PremiumUser {
            discord_id: to_transfer_id,
            patreon_id: None,
            discord_servers: Vec::new(),
            premium_type: premium_user.premium_type,
            premium_owner: Some(premium_user.discord_id),
            premium_patreon_owner: premium_user.patreon_id,
        };
        ctx.database.delete_premium(msg.author.id.0).await?;
        ctx.database.add_premium(new_premium_user, false).await?;

        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::DarkGreen as u32)
            .unwrap()
            .title("Premium Transfer Successful")
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
    } else if let Some(transferred_premium_user) = ctx
        .database
        .get_transferred_premium(msg.author.id.0)
        .await?
    {
        ctx.database
            .delete_premium(transferred_premium_user.discord_id as u64)
            .await?;

        let premium_user = PremiumUser {
            discord_id: msg.author.id.0 as i64,
            patreon_id: transferred_premium_user.premium_patreon_owner,
            discord_servers: Vec::new(),
            premium_type: transferred_premium_user.premium_type,
            premium_owner: None,
            premium_patreon_owner: None,
        };
        ctx.database.add_premium(premium_user, false).await?;

        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::DarkGreen as u32)
            .unwrap()
            .title("Premium Transfer Successful")
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
    } else {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Premium Transfer Failed")
            .unwrap()
            .description("You do not have a premium subscription")
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
    }
    Ok(())
}
