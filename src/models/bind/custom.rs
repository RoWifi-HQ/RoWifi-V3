use async_trait::async_trait;
use serde::{Serialize, Deserialize, Deserializer};
use std::{collections::HashMap, sync::Arc, fmt};
use twilight_model::id::{RoleId, GuildId};

use super::Backup;
use crate::{cache::CachedRole, framework::context::Context};
use crate::models::command::RoCommand;

#[derive(Serialize)]
pub struct CustomBind {
    #[serde(rename = "_id")]
    pub id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>,

    #[serde(rename = "Code")]
    pub code: String,

    #[serde(rename = "Prefix")]
    pub prefix: String,

    #[serde(rename = "Priority")]
    pub priority: i64,

    #[serde(skip_serializing)]
    pub command: RoCommand
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupCustomBind {
    #[serde(rename = "_id")]
    pub id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<String>,

    #[serde(rename = "Code")]
    pub code: String,

    #[serde(rename = "Prefix")]
    pub prefix: String,

    #[serde(rename = "Priority")]
    pub priority: i64,
}

impl<'de> Deserialize<'de> for CustomBind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Deserialize)]
        pub struct EncodedCustombind {
            #[serde(rename = "_id")]
            pub id: i64,

            #[serde(rename = "DiscordRoles")]
            pub discord_roles: Vec<i64>,

            #[serde(rename = "Code")]
            pub code: String,

            #[serde(rename = "Prefix")]
            pub prefix: String,

            #[serde(rename = "Priority")]
            pub priority: i64
        }

        let input = EncodedCustombind::deserialize(deserializer)?;
        let command = RoCommand::new(&input.code).map_err(serde::de::Error::custom)?;

        Ok(CustomBind {
            id: input.id,
            discord_roles: input.discord_roles,
            code: input.code,
            prefix: input.prefix,
            priority: input.priority,
            command
        })
    }
}

impl fmt::Debug for CustomBind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomBind")
            .field("Id", &self.id)
            .field("Discord Roles", &self.discord_roles)
            .field("Code", &self.code)
            .field("Prefix", &self.prefix)
            .field("Priority", &self.priority)
            .finish()
    }
}

#[async_trait]
impl Backup for CustomBind {
    type Bind = BackupCustomBind;

    fn to_backup(&self, roles: &HashMap<RoleId, Arc<CachedRole>>) -> Self::Bind {
        let mut discord_roles = Vec::new();
        for role_id in self.discord_roles.iter() {
            if let Some(role) = roles.get(&RoleId(*role_id as u64)) {
                discord_roles.push(role.name.clone());
            }
        }

        BackupCustomBind {
            id: self.id,
            discord_roles,
            code: self.code.clone(),
            priority: self.priority,
            prefix: self.prefix.clone()
        }
    }

    async fn from_backup(ctx: &Context, guild_id: GuildId, bind: Self::Bind, roles: &Vec<Arc<CachedRole>>) -> Self {
        let mut discord_roles = Vec::new();
        for role_name in bind.discord_roles {
            let role = match roles.iter().find(|r| r.name.eq_ignore_ascii_case(&role_name)) {
                Some(r) => r.id.0 as i64,
                None => {
                    let role = ctx.http.create_role(guild_id).name(role_name).await.expect("Error creating a role");
                    role.id.0 as i64
                }
            };
            discord_roles.push(role);
        }

        let command = RoCommand::new(&bind.code).unwrap();

        CustomBind {
            id: bind.id,
            discord_roles,
            code: bind.code.clone(),
            priority: bind.priority,
            prefix: bind.prefix.clone(),
            command
        }
    }
}