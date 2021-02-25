use rowifi_framework::prelude::*;

#[derive(FromArgs)]
pub struct RankbindsCreateArguments {}

pub async fn rankbinds_create(ctx: CommandContext, _: RankbindsCreateArguments) -> CommandResult {
    Ok(())
}