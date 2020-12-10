use rowifi_framework::prelude::Context;
use std::error::Error;
use tokio::time::{interval, Duration};
use twilight_model::gateway::{
    payload::UpdateStatus,
    presence::{Activity, ActivityType, Status},
};

pub async fn activity(ctx: Context) {
    let mut interval = interval(Duration::from_secs(30 * 60));
    let mut show_members = false;
    loop {
        interval.tick().await;
        if let Err(err) = execute(&ctx, &mut show_members).await {
            tracing::error!(err = ?err, "Error in activity module: ")
        }
    }
}

async fn execute(ctx: &Context, show_members: &mut bool) -> Result<(), Box<dyn Error>> {
    let shards = ctx.cluster.shards();
    let (name, kind) = match show_members {
        true => (
            format!("{:?} members", ctx.stats.resource_counts.users.get()),
            ActivityType::Listening,
        ),
        false => (
            format!("{:?} servers", ctx.stats.resource_counts.guilds.get()),
            ActivityType::Watching,
        ),
    };
    let activity = Activity {
        application_id: None,
        assets: None,
        created_at: None,
        details: None,
        emoji: None,
        flags: None,
        id: None,
        instance: None,
        kind,
        name,
        party: None,
        secrets: None,
        state: None,
        timestamps: None,
        url: None,
    };
    let update = UpdateStatus::new(Some(vec![activity]), false, None, Status::Online);
    for shard in shards {
        shard.command(&update).await?;
    }
    *show_members = !*show_members;
    Ok(())
}
