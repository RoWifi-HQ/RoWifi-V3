mod bind;
mod reset;
mod serverinfo;
mod update_mul;

use std::time::Duration;

use rowifi_framework::{bucket::BucketLayer, handler::CommandHandler, prelude::*};
use tower::ServiceBuilder;

use bind::bind;
use reset::reset;
use serverinfo::serverinfo;
use update_mul::{update_all, update_role};

pub fn group_config(cmds: &mut Vec<Command>) {
    let serverinfo_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["serverinfo"])
        .description("Command to view information about the server")
        .group("Miscellanous")
        .handler(serverinfo);

    let bucket = BucketLayer::new(Duration::from_secs(12 * 60 * 60), 3);

    let update_all_srv = ServiceBuilder::new()
        .layer(bucket.clone())
        .service(CommandHandler::new(update_all));
    let update_all_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["update-all"])
        .description("Command to update all members in the server")
        .group("Premium")
        .service(Box::new(update_all_srv));

    let update_role_srv = ServiceBuilder::new()
        .layer(bucket)
        .service(CommandHandler::new(update_role));
    let update_role_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["update-role"])
        .description("Command to update all members with a specific role in the server")
        .group("Premium")
        .service(Box::new(update_role_srv));

    let reset_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["reset"])
        .group("Administration")
        .description("Command to reset the bot for this server")
        .handler(reset);

    let bind_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["bind"])
        .group("Administration")
        .description("Command to create a bind for the server")
        .handler(bind);

    cmds.push(serverinfo_cmd);
    cmds.push(update_all_cmd);
    cmds.push(update_role_cmd);
    cmds.push(reset_cmd);
    cmds.push(bind_cmd);
}
