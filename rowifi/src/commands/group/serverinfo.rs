use itertools::Itertools;
use rowifi_database::postgres::Row;
use rowifi_framework::prelude::*;
use rowifi_models::{bind::BindType, FromRow};

struct BindCount {
    pub bind_type: BindType,
    pub count: i64,
}

impl FromRow for BindCount {
    fn from_row(row: Row) -> Result<Self, rowifi_database::postgres::Error> {
        let bind_type = row.try_get("bind_type")?;
        let count = row.try_get("count")?;

        Ok(Self { bind_type, count })
    }
}

pub async fn serverinfo(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;
    let rows = ctx
        .bot
        .database
        .query::<BindCount>(
            "SELECT bind_type, COUNT(*) AS count FROM binds WHERE guild_id = $1 GROUP BY bind_type",
            &[&(guild_id)],
        )
        .await?;

    let mut verification_roles = guild
        .verification_roles
        .iter()
        .map(|r| format!("<@&{}>", r))
        .join(" ");
    if verification_roles.is_empty() {
        verification_roles = "None".into();
    }
    let mut verified_roles = guild
        .verified_roles
        .iter()
        .map(|r| format!("<@&{}>", r))
        .join(" ");
    if verified_roles.is_empty() {
        verified_roles = "None".into();
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .field(EmbedFieldBuilder::new("Guild Id", guild_id.0.to_string()).inline())
        .field(
            EmbedFieldBuilder::new(
                "Member Count",
                ctx.bot.cache.member_count(guild_id).to_string(),
            )
            .inline(),
        )
        .field(EmbedFieldBuilder::new("Cluster Id", ctx.bot.cluster_id.to_string()).inline())
        .field(EmbedFieldBuilder::new("Tier", guild.kind.to_string()).inline())
        .field(EmbedFieldBuilder::new("Prefix", &guild.command_prefix).inline())
        .field(EmbedFieldBuilder::new("Verification Roles", verification_roles).inline())
        .field(EmbedFieldBuilder::new("Verified Role", verified_roles).inline())
        .field(
            EmbedFieldBuilder::new(
                "Rankbinds",
                rows.iter()
                    .find(|r| r.bind_type == BindType::Rank)
                    .map(|r| r.count)
                    .unwrap_or_default()
                    .to_string(),
            )
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Groupbinds",
                rows.iter()
                    .find(|r| r.bind_type == BindType::Group)
                    .map(|r| r.count)
                    .unwrap_or_default()
                    .to_string(),
            )
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Custombinds",
                rows.iter()
                    .find(|r| r.bind_type == BindType::Custom)
                    .map(|r| r.count)
                    .unwrap_or_default()
                    .to_string(),
            )
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Assetbinds",
                rows.iter()
                    .find(|r| r.bind_type == BindType::Asset)
                    .map(|r| r.count)
                    .unwrap_or_default()
                    .to_string(),
            )
            .inline(),
        )
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;
    Ok(())
}
