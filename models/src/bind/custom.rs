use serde::{
    de::{Error as DeError, IgnoredAny, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::fmt::{Formatter, Result as FmtResult};

use super::Template;

use crate::{id::RoleId, rolang::RoCommand, serialize_i64_as_string, FromRow};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Custombind {
    /// The global id of the bind
    #[serde(serialize_with = "serialize_i64_as_string")]
    pub bind_id: i64,
    /// The ID of the Custom Bind
    pub custom_bind_id: i32,
    /// The discord roles bound to the custombind
    pub discord_roles: Vec<RoleId>,
    /// The code of the bind
    pub code: String,
    /// The number that decides whether this bind is chosen for the nickname
    pub priority: i32,
    /// The format of the nickname if this bind is chosen
    pub template: Template,
    #[serde(skip_serializing)]
    pub command: RoCommand,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CustombindBackup {
    pub custom_bind_id: i32,
    pub discord_roles: Vec<String>,
    pub code: String,
    pub priority: i32,
    pub template: Template,
}

impl FromRow for Custombind {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let bind_id = row.try_get("bind_id")?;
        let custom_bind_id = row.try_get("custom_bind_id")?;
        let discord_roles = row.try_get("discord_roles")?;
        let code: String = row.try_get("code")?;
        let priority = row.try_get("priority")?;
        let template = row.try_get("template")?;
        let command = RoCommand::new(&code).unwrap();

        Ok(Self {
            bind_id,
            custom_bind_id,
            discord_roles,
            code,
            priority,
            template,
            command,
        })
    }
}

impl<'de> Deserialize<'de> for Custombind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            BindId,
            CustomBindId,
            DiscordRoles,
            Code,
            Priority,
            Template,
        }

        struct CustomBindVisitor;

        impl<'de> Visitor<'de> for CustomBindVisitor {
            type Value = Custombind;

            fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
                f.write_str("struct CustomBind")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut bind_id = None;
                let mut custom_bind_id = None;
                let mut discord_roles = None;
                let mut code = None::<String>;
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
                        Field::BindId => {
                            if bind_id.is_some() {
                                return Err(DeError::duplicate_field("bind_id"));
                            }
                            bind_id = Some(map.next_value()?);
                        }
                        Field::CustomBindId => {
                            if custom_bind_id.is_some() {
                                return Err(DeError::duplicate_field("custom_bind_id"));
                            }
                            custom_bind_id = Some(map.next_value()?);
                        }
                        Field::DiscordRoles => {
                            if discord_roles.is_some() {
                                return Err(DeError::duplicate_field("discord_roles"));
                            }
                            discord_roles = Some(map.next_value()?);
                        }
                        Field::Code => {
                            if code.is_some() {
                                return Err(DeError::duplicate_field("code"));
                            }
                            code = Some(map.next_value()?);
                        }
                        Field::Priority => {
                            if priority.is_some() {
                                return Err(DeError::duplicate_field("priority"));
                            }
                            priority = Some(map.next_value()?);
                        }
                        Field::Template => {
                            if template.is_some() {
                                return Err(DeError::duplicate_field("template"));
                            }
                            template = Some(map.next_value()?);
                        }
                    }
                }

                let bind_id = bind_id.ok_or_else(|| DeError::missing_field("bind_id"))?;
                let custom_bind_id =
                    custom_bind_id.ok_or_else(|| DeError::missing_field("custom_bind_id"))?;
                let discord_roles =
                    discord_roles.ok_or_else(|| DeError::missing_field("discord_roles"))?;
                let priority = priority.ok_or_else(|| DeError::missing_field("priority"))?;
                let code = code.ok_or_else(|| DeError::missing_field("code"))?;
                let template = template.ok_or_else(|| DeError::missing_field("template"))?;
                let command = RoCommand::new(&code).unwrap();

                Ok(Custombind {
                    bind_id,
                    custom_bind_id,
                    discord_roles,
                    code,
                    priority,
                    template,
                    command,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "bind_id",
            "custom_bind_id",
            "discord_roles",
            "code",
            "priority",
            "template",
        ];

        deserializer.deserialize_struct("Custombind", FIELDS, CustomBindVisitor)
    }
}
