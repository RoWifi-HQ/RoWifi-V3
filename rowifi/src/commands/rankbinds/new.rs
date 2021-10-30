use itertools::Itertools;
use lazy_static::lazy_static;
use mongodb::bson::doc;
use regex::Regex;
use rowifi_framework::prelude::*;
use rowifi_models::discord::id::RoleId;
use rowifi_models::{
    bind::{RankBind, Template},
    roblox::id::GroupId,
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
    pub priority: Option<i64>,
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
    let mut guild = ctx.bot.database.get_guild(guild_id.0.get()).await?;
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
            ctx.respond().embeds(&[embed]).exec().await?;
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
        ctx.respond().embeds(&[embed]).exec().await?;
        return Ok(());
    }

    let mut added = Vec::new();
    let mut modified = Vec::new();

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
                    Some(r) => r.id.0.get() as i64,
                    None => {
                        let new_role = ctx
                            .bot
                            .http
                            .create_role(ctx.guild_id.unwrap())
                            .name(&roblox_rank.name)
                            .exec()
                            .await?
                            .model()
                            .await?;
                        new_role.id.0.get() as i64
                    }
                };
                roles.push(role);
            } else if let Some(role_id) = parse_role(role_to_add) {
                if server_roles
                    .iter()
                    .any(|r| r.id == RoleId::new(role_id).unwrap())
                {
                    roles.push(role_id as i64);
                }
            }
        }

        let rank_id = i64::from(roblox_rank.rank);
        let bind = RankBind {
            group_id,
            rank_id,
            rbx_rank_id: roblox_rank.id.0 as i64,
            prefix: None,
            priority,
            discord_roles: roles,
            template: Some(Template(template_str)),
        };

        match guild
            .rankbinds
            .iter()
            .find_position(|r| r.group_id == group_id && r.rank_id == rank_id)
        {
            Some((pos, _)) => {
                guild.rankbinds[pos] = bind.clone();
                modified.push(bind);
            }
            None => {
                guild.rankbinds.push(bind.clone());
                added.push(bind);
            }
        }
    }

    ctx.bot.database.add_guild(&guild, true).await?;
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
        let name = format!("Rank: {}", rb.rank_id);
        let nick = if let Some(template) = &rb.template {
            format!("Template: `{}`\n", template)
        } else {
            String::default()
        };
        let desc = format!(
            "{}Priority: {}\n Roles: {}",
            nick,
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
        let name = format!("Rank: {}", rb.rank_id);
        let nick = if let Some(template) = &rb.template {
            format!("Template: `{}`\n", template)
        } else {
            String::default()
        };
        let desc = format!(
            "{}Priority: {}\n Roles: {}",
            nick,
            rb.priority,
            rb.discord_roles
                .iter()
                .map(|r| format!("<@&{}>", r))
                .collect::<String>()
        );
        embed = embed.field(EmbedFieldBuilder::new(name, desc).inline().build());
        count += 1;
    }

    ctx.respond()
        .embeds(&[embed.build().unwrap()])
        .exec()
        .await?;

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

pub async fn log_rankbind(ctx: &CommandContext, bind: RankBind) {
    let name = format!("Group Id: {}", bind.group_id);
    let roles_str = bind
        .discord_roles
        .iter()
        .map(|r| format!("<@&{}> ", r))
        .collect::<String>();
    let desc = format!(
        "Rank Id: {}\nTemplate: `{}`\nPriority: {}\nDiscord Roles: {}",
        bind.rank_id,
        bind.template.unwrap(),
        bind.priority,
        roles_str
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
