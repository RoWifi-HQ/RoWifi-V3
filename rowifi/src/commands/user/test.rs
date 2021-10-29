use rowifi_framework::prelude::*;

#[derive(Debug, FromArgs)]
pub struct TestArguments {}

#[allow(clippy::unused_async)]
pub async fn test(_ctx: CommandContext, _args: TestArguments) -> Result<(), RoError> {
    Ok(())
}
