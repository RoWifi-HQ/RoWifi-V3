use serde::{
    de::{Deserializer, Error as DeError, IgnoredAny, MapAccess, Visitor},
    Deserialize, Serialize,
};
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter, Result as FmtResult},
};
use twilight_model::id::RoleId;

use super::{template::Template, Backup, Bind};
use crate::{roblox::user::PartialUser as RobloxUser, rolang::RoCommand, user::RoGuildUser};

#[derive(Serialize, Clone)]
pub struct CustomBind {
    /// The ID of the Custom Bind
    #[serde(rename = "_id")]
    pub id: i64,
    /// The discord roles bound to the custombind
    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>,
    /// The code of the bind
    #[serde(rename = "Code")]
    pub code: String,
    /// The prefix to set if the bind is chosen. Deprecated
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// The number that decides whether this bind is chosen for the nickname
    #[serde(rename = "Priority")]
    pub priority: i64,
    /// The format of the nickname if this bind is chosen
    #[serde(rename = "Template", skip_serializing_if = "Option::is_none")]
    pub template: Option<Template>,
    #[serde(skip_serializing)]
    pub command: RoCommand,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupCustomBind {
    #[serde(rename = "_id")]
    pub id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<String>,

    #[serde(rename = "Code")]
    pub code: String,

    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    #[serde(rename = "Priority")]
    pub priority: i64,

    #[serde(rename = "Template", skip_serializing_if = "Option::is_none")]
    pub template: Option<Template>,
}

impl Debug for CustomBind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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
    type BackupBind = BackupCustomBind;

    fn to_backup(&self, roles: &HashMap<RoleId, String>) -> Self::BackupBind {
        let mut discord_roles = Vec::new();
        for role_id in &self.discord_roles {
            if let Some(role) = roles.get(&RoleId(*role_id as u64)) {
                discord_roles.push(role.clone());
            }
        }

        BackupCustomBind {
            id: self.id,
            discord_roles,
            code: self.code.clone(),
            priority: self.priority,
            prefix: self.prefix.clone(),
            template: self.template.clone(),
        }
    }

    fn from_backup(bind: &Self::BackupBind, roles: &HashMap<String, RoleId>) -> Self {
        let mut discord_roles = Vec::new();
        for role_name in &bind.discord_roles {
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
            command,
            template: bind.template.clone(),
        }
    }
}

impl Bind for CustomBind {
    fn nickname(
        &self,
        roblox_user: &RobloxUser,
        user: &RoGuildUser,
        discord_username: &str,
        discord_nick: &Option<String>,
    ) -> String {
        if let Some(template) = &self.template {
            return template.nickname(roblox_user, user, discord_username);
        } else if let Some(prefix) = &self.prefix {
            if prefix.eq_ignore_ascii_case("N/A") {
                return roblox_user.name.clone();
            } else if prefix.eq_ignore_ascii_case("disable") {
                return discord_nick.clone().unwrap_or_else(|| discord_username.to_string());
            }
            return format!("{} {}", prefix, roblox_user.name);
        }
        roblox_user.name.clone()
    }

    fn priority(&self) -> i64 {
        self.priority
    }
}

impl<'de> Deserialize<'de> for CustomBind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier)]
        enum Field {
            #[serde(rename = "_id")]
            Id,
            DiscordRoles,
            Code,
            Prefix,
            Priority,
            Template,
        }

        struct CustomBindVisitor;

        impl<'de> Visitor<'de> for CustomBindVisitor {
            type Value = CustomBind;

            fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
                f.write_str("struct CustomBind")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut id = None;
                let mut discord_roles = None;
                let mut code = None::<String>;
                let mut prefix = None;
                let mut priority = None;
                let mut template = None;

                loop {
                    let key = match map.next_key() {
                        Ok(Some(key)) => key,
                        Ok(None) => break,
                        Err(_) => {
                            map.next_value::<IgnoredAny>()?;
                            continue;
                        }
                    };

                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(DeError::duplicate_field("_id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Field::DiscordRoles => {
                            if discord_roles.is_some() {
                                return Err(DeError::duplicate_field("DiscordRoles"));
                            }
                            discord_roles = Some(map.next_value()?);
                        }
                        Field::Code => {
                            if code.is_some() {
                                return Err(DeError::duplicate_field("Code"));
                            }
                            code = Some(map.next_value()?);
                        }
                        Field::Prefix => {
                            if prefix.is_some() {
                                return Err(DeError::duplicate_field("Prefix"));
                            }
                            prefix = Some(map.next_value()?);
                        }
                        Field::Priority => {
                            if priority.is_some() {
                                return Err(DeError::duplicate_field("Priority"));
                            }
                            priority = Some(map.next_value()?);
                        }
                        Field::Template => {
                            if template.is_some() {
                                return Err(DeError::duplicate_field("Template"));
                            }
                            template = Some(map.next_value()?);
                        }
                    }
                }

                let id = id.ok_or_else(|| DeError::missing_field("Id"))?;
                let discord_roles =
                    discord_roles.ok_or_else(|| DeError::missing_field("DiscordRoles"))?;
                let priority = priority.ok_or_else(|| DeError::missing_field("Priority"))?;
                let code = code.ok_or_else(|| DeError::missing_field("Code"))?;
                let command = RoCommand::new(&code).unwrap();

                Ok(CustomBind {
                    id,
                    discord_roles,
                    code,
                    prefix,
                    priority,
                    template,
                    command,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "_id",
            "DiscordRoles",
            "Code",
            "Priority",
            "Prefix",
            "Template",
        ];

        deserializer.deserialize_struct("CustomBind", FIELDS, CustomBindVisitor)
    }
}
