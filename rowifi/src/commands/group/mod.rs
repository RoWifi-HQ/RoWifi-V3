mod serverinfo;
mod setup;
mod update_mul;

use rowifi_framework::prelude::*;

pub use serverinfo::serverinfo;
pub use setup::setup;
pub use update_mul::{update_all, update_role};

pub fn group_config(cmds: &mut Vec<Command>) {
    let serverinfo_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["serverinfo"])
        .description("Command to view information about the server")
        .group("Miscellanous")
        .handler(serverinfo);

    let setup_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["setup"])
        .description("Command to setup or reset your server settings in the database")
        .group("Administration")
        .handler(setup);

    let update_all_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["update-all"])
        .description("Command to update all members in the server")
        .group("Premium")
        .handler(update_all);

    let update_role_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["update-role"])
        .description("Command to update all members with a specific role in the server")
        .group("Premium")
        .handler(update_role);

    cmds.push(serverinfo_cmd);
    cmds.push(setup_cmd);
    cmds.push(update_all_cmd);
    cmds.push(update_role_cmd);
}
