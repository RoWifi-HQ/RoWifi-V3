use rowifi_framework::{parser::ParseError, prelude::*, structures::HelpCommand, CommandMap};
use itertools::Itertools;
use twilight_embed_builder::EmbedFieldBuilder;

pub static HELP_COMMAND: HelpCommand = HelpCommand {
    fun: help,
    name: "help",
};

#[command]
pub async fn help(
    ctx: &Context,
    msg: &Message,
    args: Arguments<'fut>,
    commands: &[(&'static Command, CommandMap)],
) -> CommandResult {
    if args.as_str().is_empty() {
        global_help(ctx, msg, commands).await?;
    } else {
        specific_help(ctx, msg, args, commands).await?;
    }
    Ok(())
}

async fn global_help(
    ctx: &Context,
    msg: &Message,
    commands: &[(&'static Command, CommandMap)],
) -> Result<(), RoError> {
    let mut embed = EmbedBuilder::new()
        .default_data()
        .title("Help")
        .unwrap()
        .description("Listing all top-level commands")
        .unwrap();

    let groups = commands.iter().group_by(|c| c.0.options.group);
    for (group, commands) in &groups {
        if let Some(group) = group {
            let commands = commands
                .filter(|c| !c.0.options.hidden)
                .map(|m| format!("`{}`", m.0.options.names[0]))
                .join(" ");
            embed = embed.field(EmbedFieldBuilder::new(group, commands).unwrap());
        }
    }

    let embed = embed.build().unwrap();
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}

async fn specific_help(
    ctx: &Context,
    msg: &Message,
    args: Arguments<'_>,
    commands: &[(&'static Command, CommandMap)],
) -> Result<(), RoError> {
    let mut embed = EmbedBuilder::new().default_data().title("Help").unwrap();
    let mut last = Err(ParseError::UnrecognisedCommand(None));
    for (_command, map) in commands {
        let res = parse_command(args.clone(), map);
        if res.is_ok() {
            last = res;
            break;
        }
        last = res;
    }

    if let Ok(command) = last {
        embed = embed
            .description(format!(
                "`{}`: {}",
                command.options.names[0],
                command.options.desc.unwrap_or("None")
            ))
            .unwrap();
        if command.options.names.len() > 1 {
            let aliases = command.options.names[1..]
                .iter()
                .map(|a| format!("`{}`", a))
                .join(" ");
            embed = embed.field(EmbedFieldBuilder::new("Aliases", aliases).unwrap());
        }
        if let Some(usage) = command.options.usage {
            embed = embed.field(EmbedFieldBuilder::new("Usage", format!("`{}`", usage)).unwrap());
        }
        if !command.options.examples.is_empty() {
            let examples = command
                .options
                .examples
                .iter()
                .map(|e| format!("`{}`", e))
                .join("\n");
            embed = embed.field(EmbedFieldBuilder::new("Examples", examples).unwrap());
        }
        if !command.options.sub_commands.is_empty() {
            let subs = command
                .options
                .sub_commands
                .iter()
                .filter(|c| !c.options.hidden)
                .map(|c| format!("`{}`", c.options.names[0]))
                .join(", ");
            embed = embed.field(EmbedFieldBuilder::new("Subcommands", subs).unwrap());
        }
        let embed = embed.build().unwrap();

        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
    } else {
        global_help(ctx, msg, commands).await?;
    }
    Ok(())
}

fn parse_command(mut args: Arguments, map: &CommandMap) -> Result<&'static Command, ParseError> {
    if let Some(arg) = args.next() {
        if let Some((cmd, map)) = map.get(arg) {
            if map.is_empty() {
                return Ok(cmd);
            }

            return match parse_command(args, &map) {
                Err(ParseError::UnrecognisedCommand(Some(_))) => Ok(cmd),
                res => res,
            };
        }
        return Err(ParseError::UnrecognisedCommand(Some(arg.to_string())));
    }
    Err(ParseError::UnrecognisedCommand(Some(
        "Arguments ended".into(),
    )))
}
