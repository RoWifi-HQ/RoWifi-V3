mod info;
mod test;
mod update;
mod verify;

pub use info::{botinfo, support, userinfo};
use rowifi_framework::prelude::*;
pub use test::test;
pub use update::update;
pub use verify::verify;

use self::verify::verify_config;

pub fn user_config(cmds: &mut Vec<Command>) {
    let update_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["update", "getroles"])
        .description("Command to update an user")
        .group("User")
        .handler(update);

    let userinfo_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["userinfo"])
        .description("Command to view information about an user")
        .group("User")
        .handler(userinfo);

    let botinfo_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["botinfo"])
        .description("Command to view information about the bot")
        .group("User")
        .handler(botinfo);

    let support_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["support", "invite"])
        .description("View important links related to the bot")
        .group("User")
        .handler(support);

    let test_cmd = Command::builder()
        .level(RoLevel::Creator)
        .names(&["test"])
        .handler(test);

    cmds.push(update_cmd);
    cmds.push(userinfo_cmd);
    cmds.push(botinfo_cmd);
    cmds.push(support_cmd);
    cmds.push(test_cmd);

    verify_config(cmds);
}
