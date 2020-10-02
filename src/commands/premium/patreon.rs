use crate::framework::prelude::*;
use crate::models::user::{PremiumType, PremiumUser};

pub static PREMIUM_PATREON_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["patreon"],
    desc: Some("Command to link/update patreon status"),
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static PREMIUM_PATREON_COMMAND: Command = Command {
    fun: premium_patreon,
    options: &PREMIUM_PATREON_OPTIONS
};

#[command]
pub async fn premium_patreon(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let author = match args.next().map(|a| a.parse::<u64>()) {
        Some(Ok(s)) => s,
        _ => msg.author.id.0
    };
    let premium_already = ctx.database.get_premium(author).await?.is_some();
    let premium_user: PremiumUser;
    let (patreon_id, tier) = ctx.patreon.get_patron(author).await?;
    if patreon_id.is_none() {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
            .title("Patreon Linking Failed").unwrap()
            .description("Patreon Account was not found for this Discord Account. Please make sure your Discord Account is linked to your patreon account").unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
        return Ok(());
    }
    if tier.is_none() {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
            .title("Patreon Linking Failed").unwrap()
            .description("You were not found to be a member of any tier").unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
        return Ok(());
    }

    let patreon_id = patreon_id.unwrap().parse::<i64>().unwrap();
    let tier = tier.unwrap().parse::<i64>().unwrap();
    if tier == 4014582 {
        premium_user = PremiumUser {discord_id: author as i64, patreon_id: Some(patreon_id), premium_type: PremiumType::Alpha, discord_servers: Vec::new(), premium_owner: None, premium_patreon_owner: None};
    } else if tier == 4656839 {
        premium_user = PremiumUser {discord_id: author as i64, patreon_id: Some(patreon_id), premium_type: PremiumType::Beta, discord_servers: Vec::new(), premium_owner: None, premium_patreon_owner: None};
    } else {return Ok(());}
    
    ctx.database.add_premium(premium_user, premium_already).await?;
    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Patreon Linking Successful").unwrap()
        .description("Your patreon account has successfully been registered with our database").unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}