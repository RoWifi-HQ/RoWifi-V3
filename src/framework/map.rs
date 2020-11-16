use std::collections::HashMap;
use std::sync::Arc;

use super::Command;

#[derive(Debug, Default)]
pub struct CommandMap {
    cmds: HashMap<String, (&'static Command, Arc<CommandMap>)>,
}

impl CommandMap {
    pub fn new(cmds: &[&'static Command]) -> Self {
        let mut map = Self::default();
        for cmd in cmds {
            let sub_map = Arc::new(Self::new(&cmd.options.sub_commands));
            for name in cmd.options.names {
                let name = name.to_lowercase();
                map.cmds.insert(name, (*cmd, sub_map.clone()));
            }
        }
        map
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<(&'static Command, Arc<CommandMap>)> {
        self.cmds.get(name).cloned()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cmds.is_empty()
    }
}
