use std::collections::HashMap;
use std::sync::Arc;

use super::*;

#[derive(Debug)]
pub enum Map {
    WithPrefixes(GroupMap),
    Prefixless(GroupMap, CommandMap)
}

pub trait ParseMap {
    type Storage;
    
    fn get(&self, n: &str) -> Option<Self::Storage>;
    fn is_empty(&self) -> bool;
}

#[derive(Debug, Default)]
pub struct CommandMap {
    cmds: HashMap<String, (&'static Command, Arc<CommandMap>)>
}

#[derive(Debug, Default)]
pub struct GroupMap {
    groups: HashMap<&'static str, (&'static CommandGroup, Arc<GroupMap>, Arc<CommandMap>)>
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
}

impl ParseMap for CommandMap {
    type Storage = (&'static Command, Arc<CommandMap>);

    #[inline]
    fn get(&self, name: &str) -> Option<Self::Storage> {
        self.cmds.get(name).cloned()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.cmds.is_empty()
    }
}

impl GroupMap {
    pub fn new(groups: &[&'static CommandGroup]) -> Self {
        let mut map = Self::default();
        for group in groups {
            let subgroups = Arc::new(Self::new(&group.options.sub_groups));
            let commands = Arc::new(CommandMap::new(&group.options.commands));

            for prefix in group.options.prefixes {
                map.groups.insert(&prefix, (*group, subgroups.clone(), commands.clone()));
            }
        }
        map
    }
}

impl ParseMap for GroupMap {
    type Storage = (&'static CommandGroup, Arc<GroupMap>, Arc<CommandMap>);

    #[inline]
    fn get(&self, name: &str) -> Option<Self::Storage> {
        self.groups.get(&name).cloned()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
}