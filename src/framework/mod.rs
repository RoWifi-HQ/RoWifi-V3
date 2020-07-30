mod context;
mod structures;

use std::{collections::HashMap, default::Default, sync::Arc};
use tokio::sync::{Mutex, RwLock};

use structures::{CommandGroup, Bucket};
use typemap_rev::TypeMap;

#[derive(Default)]
pub struct Framework {
    data: Arc<RwLock<TypeMap>>,
    groups: Vec<&'static CommandGroup>,
    buckets: Mutex<HashMap<String, Bucket>>,
    //help: &'static HelpCommand
}

impl Framework {
    pub fn new() -> Self {
        Framework::default()
    }
}