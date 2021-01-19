mod info;
mod update;
mod verify;

use framework_new::command::Command;
pub use info::*;
pub use update::*;
pub use verify::*;

pub fn update_config(cmds: &mut Vec<Command>) {
    let update_command = Command::builder()
        .names(&["update"])
        .description("Command to update an user")
        .handler(update);
    
    cmds.push(update_command);
}