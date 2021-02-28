use chrono::{DateTime, Duration, Utc};
use image::{png::PngEncoder, ColorType};
use mongodb::bson::doc;
use plotters::prelude::*;
use rowifi_framework::prelude::{Color as DiscordColor, *};
use rowifi_models::guild::GuildType;
use std::io::Cursor;

#[derive(FromArgs)]
pub struct ViewArguments {
    #[arg(help = "The ID of the group whose analytics is to be viewed")]
    pub group_id: i64,
    #[arg(help = "The Duration of the graph")]
    pub duration: Option<ViewDuration>,
}

pub struct ViewDuration(pub Duration);

pub async fn analytics_view(ctx: CommandContext, args: ViewArguments) -> CommandResult {
    let guild = ctx
        .bot
        .database
        .get_guild(ctx.guild_id.unwrap().0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(DiscordColor::Red as u32)
            .unwrap()
            .title("Analytics Viewing Failed")
            .unwrap()
            .description("This module may only be used in Beta Tier Servers")
            .unwrap()
            .build()
            .unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let group_id = args.group_id;
    if !guild.registered_groups.contains(&group_id) {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(DiscordColor::Red as u32)
            .unwrap()
            .title("Analytics Viewing failed")
            .unwrap()
            .description("You may only view groups that are registered with this server")
            .unwrap()
            .build()
            .unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let server = ctx.bot.cache.guild(ctx.guild_id.unwrap()).unwrap();

    let view_duration = args
        .duration
        .unwrap_or_else(|| ViewDuration(Duration::days(7)));
    let start_time = Utc::now() - view_duration.0;
    let filter = doc! {"groupId": group_id, "timestamp": {"$gte": start_time}};
    let group_data = ctx.bot.database.get_analytics_membercount(filter).await?;

    if group_data.len() <= 2 {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(DiscordColor::Red as u32)
            .unwrap()
            .title("Analytics Viewing failed")
            .unwrap()
            .description("There is not enough usable data to generate data. Please give the bot 24 hours to collect enough data or use another timeframe")
            .unwrap()
            .build()
            .unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let min_timestamp = group_data.iter().map(|g| g.timestamp).min().unwrap().0;
    let max_timestamp = group_data.iter().map(|g| g.timestamp).max().unwrap().0;
    let mut min_members = group_data.iter().map(|g| g.member_count).min().unwrap();
    let mut max_members = group_data.iter().map(|g| g.member_count).max().unwrap();
    let diff = max_members - min_members;
    min_members -= diff / 10;
    max_members += diff / 10;
    let iterator = group_data.iter().map(|g| (g.timestamp.0, g.member_count));

    let mut buffer = vec![0_u8; 1024 * 768 * 3];
    {
        let root_drawing_area =
            BitMapBackend::with_buffer(&mut buffer, (1024, 768)).into_drawing_area();
        root_drawing_area.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&root_drawing_area)
            .caption(server.name.clone(), ("Arial", 30))
            .margin(10)
            .set_label_area_size(LabelAreaPosition::Left, 40)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(min_timestamp..max_timestamp, min_members..max_members)
            .unwrap();

        chart
            .configure_mesh()
            .x_label_formatter(&|x: &DateTime<Utc>| x.date().naive_utc().to_string())
            .draw()
            .unwrap();

        chart.draw_series(LineSeries::new(iterator, &RED)).unwrap();
    }

    let mut bytes = Vec::new();
    let img = PngEncoder::new(Cursor::new(&mut bytes));
    img.encode(&buffer, 1024, 768, ColorType::Rgb8).unwrap();

    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .attachment("analytics.png", bytes)
        .await?;
    Ok(())
}

impl FromArg for ViewDuration {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        let mut arg = arg.to_string();
        if let Some(dur) = arg.pop() {
            if let Ok(num) = arg.parse::<i64>() {
                match dur {
                    'h' => return Ok(ViewDuration(Duration::hours(num))),
                    'd' => return Ok(ViewDuration(Duration::days(num))),
                    'm' => return Ok(ViewDuration(Duration::days(30 * num))),
                    'y' => return Ok(ViewDuration(Duration::days(365 * num))),
                    _ => {}
                }
            }
        }
        Err(ParseError("a time duration such as `30d` `2m` `1h`"))
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match option {
            CommandDataOption::Integer { value, .. } => value.to_string(),
            CommandDataOption::String { value, .. } => value.to_string(),
            _ => unreachable!("ViewDuration unreached"),
        };
        Self::from_arg(&arg)
    }
}
