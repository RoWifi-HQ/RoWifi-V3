use framework_new::prelude::*;
use twilight_model::id::UserId;

#[derive(Debug, FromArgs)]
pub struct TestArguments {
    pub user_id: Option<UserId>
}

pub async fn test(ctx: CommandContext, args: TestArguments) -> Result<(), RoError> {
    ctx.bot.http.create_message(ctx.msg.channel_id)
        .content(format!("{:?}", args.user_id))
        .unwrap()
        .await?;
    Ok(())
}
