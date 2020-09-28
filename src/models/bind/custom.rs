use serde::{Serialize, Deserialize, de::{Deserializer, Visitor, MapAccess, Error as DeError}};
use std::{collections::HashMap, sync::Arc, fmt};
use twilight_model::id::RoleId;

use super::Backup;
use crate::cache::CachedRole;
use crate::models::command::RoCommand;

#[derive(Serialize, Clone)]
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

    fn from_backup(bind: &Self::Bind, roles: &HashMap<String, RoleId>) -> Self {
        let mut discord_roles = Vec::new();
        for role_name in bind.discord_roles.iter() {
            let role = roles.get(role_name).unwrap().0 as i64;
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

impl<'de> Deserialize<'de> for CustomBind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier)]
        enum Field {
            #[serde(rename = "_id")] Id,
            DiscordRoles,
            Code,
            Prefix,
            Priority
        }

        struct CustomBindVisitor;

        impl<'de> Visitor<'de> for CustomBindVisitor {
            type Value = CustomBind;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("struct CustomBind")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut id = None;
                let mut discord_roles = None;
                let mut code = None::<String>;
                let mut prefix = None;
                let mut priority = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(DeError::duplicate_field("_id"));
                            }
                            id = Some(map.next_value()?);
                        },
                        Field::DiscordRoles => {
                            if discord_roles.is_some() {
                                return Err(DeError::duplicate_field("DiscordRoles"));
                            }
                            discord_roles = Some(map.next_value()?);
                        },
                        Field::Code => {
                            if code.is_some() {
                                return Err(DeError::duplicate_field("Coe"));
                            }
                            code = Some(map.next_value()?);
                        },
                        Field::Prefix => {
                            if prefix.is_some() {
                                return Err(DeError::duplicate_field("Prefix"));
                            }
                            prefix = Some(map.next_value()?);
                        },
                        Field::Priority => {
                            if priority.is_some() {
                                return Err(DeError::duplicate_field("Priority"));
                            }
                            priority = Some(map.next_value()?);
                        }
                    }
                }

                let id = id.ok_or_else(|| DeError::missing_field("Id"))?;
                let discord_roles = discord_roles.ok_or_else(|| DeError::missing_field("DiscordRoles"))?;
                let prefix = prefix.ok_or_else(|| DeError::missing_field("Prefix"))?;
                let priority = priority.ok_or_else(|| DeError::missing_field("Priority"))?;
                let code = code.ok_or_else(|| DeError::missing_field("Code"))?;
                let command = RoCommand::new(&code).unwrap();

                Ok(CustomBind {
                    id, discord_roles, prefix, priority, code: code.to_owned(), command
                })
            }
        }

        const FIELDS: &[&str] = &[
            "_id", "DiscordRoles", "Code", "Priority", "Prefix"
        ];

        deserializer.deserialize_struct("CustomBind", FIELDS, CustomBindVisitor)
    }
}