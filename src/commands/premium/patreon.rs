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
pub async fn premium_patreon(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let premium_already = ctx.database.get_premium(msg.author.id.0).await?.is_some();
    let premium_user: PremiumUser;
    let (patreon_id, tier) = ctx.patreon.get_patron(msg.author.id.0).await?;
    if patreon_id.is_none() {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
            .title("Patreon Linking Failed").unwrap()
            .description("Patreon Account was not found for this Discord Account. Please make sure your Discord Account is linked to your patreon account").unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    }
    if tier.is_none() {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
            .title("Patreon Linking Failed").unwrap()
            .description("You were not found to be a member of any tier").unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    }
    if tier.unwrap() == 4014582 {
        premium_user = PremiumUser {discord_id: msg.author.id.0 as i64, patreon_id: Some(patreon_id.unwrap() as i64), premium_type: PremiumType::Alpha, discord_servers: Vec::new()};
    } else if tier.unwrap() == 4656839 {
        premium_user = PremiumUser {discord_id: msg.author.id.0 as i64, patreon_id: Some(patreon_id.unwrap() as i64), premium_type: PremiumType::Beta, discord_servers: Vec::new()};
    } else {return Ok(());}
    
    ctx.database.add_premium(premium_user, premium_already).await?;
    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Patreon Linking Successful").unwrap()
        .description("Your patreon account has successfully been registered with our database").unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}