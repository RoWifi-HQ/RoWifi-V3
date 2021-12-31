use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{BindType, Rankbind, Template},
    roblox::id::GroupId,
    id::RoleId,
};

#[derive(Debug, FromArgs)]
pub struct NewRankbind {
    #[arg(help = "The Group ID of your Roblox Group")]
    pub group_id: i64,
    #[arg(
        help = "Either a single rank id between 1-255 or a range of rank ids separated by a `-`. Ex. 25-55"
    )]
    pub rank_id: CreateType,
    #[arg(
        help = "The template that is used for the bind. Can be set to `N/A` or `auto` or `disable`"
    )]
    pub template: String,
    #[arg(help = "The number that tells the bot which bind to choose for the nickname")]
    pub priority: Option<i32>,
    #[arg(
        help = "The discord roles to add to the bind. To tell the bot to create roles, put `auto` ",
        rest
    )]
    pub discord_roles: Option<String>,
}

lazy_static! {
    pub static ref PREFIX_REGEX: Regex = Regex::new(r"\[(.*?)\]").unwrap();
}

pub async fn rankbinds_new(ctx: CommandContext, args: NewRankbind) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let rankbinds = ctx
        .bot
        .database
        .query::<Rankbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type = $2",
            &[&(guild_id), &BindType::Rank],
        )
        .await?;
    let server_roles = ctx.bot.cache.guild_roles(guild_id);

    let group_id = args.group_id;
    let rank_ids = args.rank_id;
    let template = args.template;
    let priority = args.priority.unwrap_or_default();

    let discord_roles_str = args.discord_roles.unwrap_or_default();
    let roles_to_add = discord_roles_str
        .split_ascii_whitespace()
        .collect::<Vec<_>>();

    let roblox_group = match ctx
        .bot
        .roblox
        .get_group_ranks(GroupId(group_id as u64))
        .await?
    {
        Some(g) => g,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Rankbinds Addition Failed")
                .color(Color::Red as u32)
                .description(format!("The group with id {} does not exist", group_id))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let roblox_ranks = match rank_ids {
        CreateType::Single(id) => roblox_group
            .roles
            .iter()
            .filter(|r| i64::from(r.rank) == id)
            .collect::<Vec<_>>(),
        CreateType::Multiple(min, max) => roblox_group
            .roles
            .iter()
            .filter(|r| i64::from(r.rank) >= min && i64::from(r.rank) <= max)
            .collect::<Vec<_>>(),
    };

    if roblox_ranks.is_empty() {
        let desc = match rank_ids {
            CreateType::Single(id) => format!("There is no rank with id {}", id),
            CreateType::Multiple(min, max) => {
                format!("There are no ranks with ids between {} and {}", min, max)
            }
        };

        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Rankbinds Addition Failed")
            .description(desc)
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let mut added = Vec::new();
    let mut modified = Vec::new();

    let mut database = ctx.bot.database.get().await?;
    let transaction = database.transaction().await?;

    for roblox_rank in roblox_ranks {
        let template_str = match template.as_str() {
            "auto" => match PREFIX_REGEX.captures(&roblox_rank.name) {
                Some(m) => format!("[{}] {{roblox-username}}", m.get(1).unwrap().as_str()),
                None => "{roblox-username}".into(),
            },
            "disable" => "{discord-name}".into(),
            "N/A" => "{roblox-username}".into(),
            _ => {
                if Template::has_slug(template.as_str()) {
                    template.clone()
                } else {
                    format!("{} {{roblox-username}}", template)
                }
            }
        };

        let mut roles = Vec::new();
        for role_to_add in &roles_to_add {
            if role_to_add.eq_ignore_ascii_case("auto") {
                let role = match server_roles
                    .iter()
                    .find(|r| r.name.eq_ignore_ascii_case(&roblox_rank.name))
                {
                    Some(r) => r.id,
                    None => {
                        let new_role = ctx
                            .bot
                            .http
                            .create_role(ctx.guild_id.unwrap().0)
                            .name(&roblox_rank.name)
                            .exec()
                            .await?
                            .model()
                            .await?;
                        RoleId(new_role.id)
                    }
                };
                roles.push(role);
            } else if let Some(resolved) = &ctx.resolved {
                roles.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
            } else if let Some(role_id) = parse_role(role_to_add) {
                if server_roles
                    .iter()
                    .any(|r| r.id == role_id)
                {
                    roles.push(role_id);
                }
            }
        }

        let rank_id = i64::from(roblox_rank.rank);
        let bind = Rankbind {
            // Don't care about the bind id here. The struct is only constructed to validate that all bind fields are being collected.
            bind_id: 0,
            group_id,
            group_rank_id: rank_id,
            roblox_rank_id: roblox_rank.id.0 as i64,
            priority,
            discord_roles: roles.into_iter().unique().collect::<Vec<_>>(),
            template: Template(template_str),
        };

        match rankbinds
            .iter()
            .find(|r| r.group_id == group_id && r.group_rank_id == rank_id)
        {
            Some(existing) => {
                let stmt = transaction.prepare_cached("UPDATE binds SET priority = $1, template = $2, discord_roles = $3 WHERE bind_id = $4").await?;
                transaction
                    .execute(
                        &stmt,
                        &[
                            &bind.priority,
                            &bind.template,
                            &bind.discord_roles,
                            &existing.bind_id,
                        ],
                    )
                    .await?;
                modified.push(bind);
            }
            None => {
                let stmt = transaction.prepare_cached("INSERT INTO binds(bind_type, guild_id, group_id, group_rank_id, roblox_rank_id, template, priority, discord_roles) VALUES($1, $2, $3, $4, $5, $6, $7, $8)").await?;
                transaction
                    .execute(
                        &stmt,
                        &[
                            &BindType::Rank,
                            &(guild_id),
                            &bind.group_id,
                            &bind.group_rank_id,
                            &bind.roblox_rank_id,
                            &bind.template,
                            &bind.priority,
                            &bind.discord_roles,
                        ],
                    )
                    .await?;
                added.push(bind);
            }
        }
    }

    transaction.commit().await?;

    let mut embed = EmbedBuilder::new()
        .default_data()
        .title("Binds Addition Sucessful")
        .color(Color::DarkGreen as u32)
        .description(format!(
            "Added {} rankbinds and modified {} rankbinds",
            added.len(),
            modified.len()
        ));

    let mut count = 0;
    for rb in &added {
        if count >= 12 {
            break;
        }
        let name = format!("Rank: {}", rb.group_rank_id);
        let desc = format!(
            "Template: {}\nPriority: {}\n Roles: {}",
            rb.template,
            rb.priority,
            rb.discord_roles
                .iter()
                .map(|r| format!("<@&{}>", r))
                .collect::<String>()
        );
        embed = embed.field(EmbedFieldBuilder::new(name, desc).inline().build());
        count += 1;
    }

    for rb in &modified {
        if count >= 12 {
            break;
        }
        let name = format!("Rank: {}", rb.group_rank_id);
        let desc = format!(
            "Template: {}\nPriority: {}\n Roles: {}",
            rb.template,
            rb.priority,
            rb.discord_roles
                .iter()
                .map(|r| format!("<@&{}>", r))
                .collect::<String>()
        );
        embed = embed.field(EmbedFieldBuilder::new(name, desc).inline().build());
        count += 1;
    }

    ctx.respond().embeds(&[embed.build()?])?.exec().await?;

    for rb in added {
        log_rankbind(&ctx, rb).await;
    }
    for rb in modified {
        log_rankbind(&ctx, rb).await;
    }

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
#[repr(i8)]
pub enum CreateType {
    Single(i64),
    Multiple(i64, i64),
}

impl FromArg for CreateType {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        if let Ok(r) = arg.parse::<i64>() {
            Ok(CreateType::Single(r))
        } else if let Some((min_rank, max_rank)) = extract_ids(arg) {
            Ok(CreateType::Multiple(min_rank, max_rank))
        } else {
            Err(ParseError("a number or a range (1-255)"))
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match &option.value {
            CommandOptionValue::Integer(value) => value.to_string(),
            CommandOptionValue::String(value) => value.to_string(),
            _ => unreachable!("NewRankbind unreached"),
        };

        CreateType::from_arg(&arg)
    }
}

pub async fn log_rankbind(ctx: &CommandContext, bind: Rankbind) {
    let name = format!("Group Id: {}", bind.group_id);
    let roles_str = bind
        .discord_roles
        .iter()
        .map(|r| format!("<@&{}> ", r))
        .collect::<String>();
    let desc = format!(
        "Rank Id: {}\nTemplate: `{}`\nPriority: {}\nDiscord Roles: {}",
        bind.group_rank_id, bind.template, bind.priority, roles_str
    );
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Rank Bind Addition")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(ctx.guild_id.unwrap(), log_embed).await;
}

fn extract_ids(rank_str: &str) -> Option<(i64, i64)> {
    let splits = rank_str.split('-').collect_vec();
    if splits.len() == 2 {
        if let Ok(r1) = splits[0].parse::<i64>() {
            if let Ok(r2) = splits[1].parse::<i64>() {
                return Some((r1, r2));
            }
        }
    }
    None
}
