use rowifi_framework::prelude::*;

#[derive(Debug, FromArgs)]
pub struct TestArguments {}

pub async fn test(_ctx: CommandContext, _args: TestArguments) -> Result<(), RoError> {
    Ok(())
}
