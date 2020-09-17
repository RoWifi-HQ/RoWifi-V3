mod backup;
mod settings;
mod types;

use itertools::Itertools;
use serde::{Serialize, Deserialize, de::{Deserializer, Visitor, MapAccess, Error as DeError}};
use std::{default::Default, collections::HashMap, sync::Arc, fmt};
use twilight_model::id::{RoleId, GuildId};

use super::{bind::*, blacklist::*};
use crate::cache::CachedRole;
use crate::framework::context::Context;

pub use backup::*;
pub use settings::*;
pub use types::*;

#[derive(Debug, Serialize, Default)]
pub struct RoGuild {
    #[serde(rename = "_id")]
    pub id: i64,

    #[serde(rename = "Prefix")]
    pub command_prefix: Option<String>,

    #[serde(rename = "Settings")]
    pub settings: GuildSettings,

    #[serde(rename = "VerificationRole")]
    pub verification_role: i64,

    #[serde(rename = "VerifiedRole")]
    pub verified_role: i64,

    #[serde(rename = "RankBinds")]
    pub rankbinds: Vec<RankBind>,

    #[serde(rename = "GroupBinds")]
    pub groupbinds: Vec<GroupBind>,

    #[serde(rename = "CustomBinds")]
    #[serde(default)]
    pub custombinds: Vec<CustomBind>,

    #[serde(rename = "AssetBinds")]
    #[serde(default)]
    pub assetbinds: Vec<AssetBind>,

    #[serde(rename = "Blacklists")]
    #[serde(default)]
    pub blacklists: Vec<Blacklist>,

    #[serde(rename = "DisabledChannels")]
    #[serde(default)]
    pub disabled_channels: Vec<i64>,

    #[serde(skip_serializing)]
    pub all_roles: Vec<i64>
}

impl RoGuild {
    pub fn to_backup(&self, user_id: i64, name: &str, roles: &HashMap<RoleId, Arc<CachedRole>>) -> BackupGuild {
        let rankbinds = self.rankbinds.iter().map(|r| r.to_backup(roles)).collect_vec();
        let groupbinds = self.groupbinds.iter().map(|g| g.to_backup(roles)).collect_vec();
        let custombinds = self.custombinds.iter().map(|c| c.to_backup(roles)).collect_vec();
        let assetbinds = self.assetbinds.iter().map(|a| a.to_backup(roles)).collect_vec();

        BackupGuild {
            id: bson::oid::ObjectId::new(),
            user_id,
            name: name.to_string(),
            command_prefix: self.command_prefix.clone(),
            settings: self.settings.clone(),
            verification_role: roles.get(&RoleId(self.verification_role as u64)).map(|r| r.name.clone()),
            verified_role: roles.get(&RoleId(self.verified_role as u64)).map(|r| r.name.clone()),
            rankbinds,
            groupbinds,
            custombinds,
            assetbinds,
            blacklists: self.blacklists.clone()
        }
    }

    pub async fn from_backup(backup: BackupGuild, ctx: &Context, guild_id: GuildId, existing_roles: &Vec<Arc<CachedRole>>) -> Self {
        let mut names_to_ids = HashMap::<String, RoleId>::new();

        let all_roles = backup.rankbinds.iter()
            .flat_map(|r| r.discord_roles.iter().map(|r| r.clone()))
            .chain(backup.groupbinds.iter().flat_map(|g| g.discord_roles.iter().map(|r| r.clone())))
            .chain(backup.custombinds.iter().flat_map(|c| c.discord_roles.iter().map(|r| r.clone())))
            .chain(backup.assetbinds.iter().flat_map(|a| a.discord_roles.iter().map(|r| r.clone())))
            .unique()
            .collect::<Vec<String>>();
        for role_name in all_roles {
            if let Some(r) = existing_roles.iter().find(|r| r.name.eq_ignore_ascii_case(&role_name)) {
                names_to_ids.insert(role_name, r.id);
            } else {
                let role = ctx.http.create_role(guild_id).name(role_name).await.expect("Error creating a role");
                names_to_ids.insert(role.name, role.id);
            }
        }

        let rankbinds = backup.rankbinds.iter().map(|bind| RankBind::from_backup(bind, &names_to_ids)).collect_vec();
        let groupbinds = backup.groupbinds.iter().map(|bind| GroupBind::from_backup(bind, &names_to_ids)).collect_vec();
        let custombinds = backup.custombinds.iter().map(|bind| CustomBind::from_backup(bind, &names_to_ids)).collect_vec();
        let assetbinds = backup.assetbinds.iter().map(|bind| AssetBind::from_backup(bind, &names_to_ids)).collect_vec();

        let verification_role = if let Some(verification_name) = backup.verification_role {
            if let Some(r) = names_to_ids.get(&verification_name) {
                r.0 as i64
            } else if let Some(r) = existing_roles.iter().find(|e| e.name.eq(&verification_name)) {
                r.id.0 as i64
            } else {
                let role = ctx.http.create_role(guild_id).name(verification_name).await.expect("Error creating a role");
                role.id.0 as i64
            }
        } else {
            0
        };

        let verified_role = if let Some(verified_name) = backup.verified_role {
            if let Some(r) = names_to_ids.get(&verified_name) {
                r.0 as i64
            } else if let Some(r) = existing_roles.iter().find(|e| e.name.eq(&verified_name)) {
                r.id.0 as i64
            } else {
                let role = ctx.http.create_role(guild_id).name(verified_name).await.expect("Error creating a role");
                role.id.0 as i64
            }
        } else {
            0
        };

        let all_roles = rankbinds.iter()
            .flat_map(|r| r.discord_roles.iter().cloned())
            .chain(groupbinds.iter().flat_map(|g| g.discord_roles.iter().cloned()))
            .chain(custombinds.iter().flat_map(|c| c.discord_roles.iter().cloned()))
            .chain(assetbinds.iter().flat_map(|a| a.discord_roles.iter().cloned()))
            .unique()
            .collect_vec();

        Self {
            id: guild_id.0 as i64,
            command_prefix: backup.command_prefix,
            settings: backup.settings,
            verification_role,
            verified_role,
            rankbinds,
            groupbinds,
            custombinds,
            assetbinds,
            blacklists: backup.blacklists,
            disabled_channels: Vec::new(),
            all_roles
        }
    }
}

impl<'de> Deserialize<'de> for RoGuild{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier)]
        enum Field {
            #[serde(rename = "_id")] Id,
            Prefix,
            Settings,
            VerificationRole,
            VerifiedRole,
            RankBinds,
            GroupBinds,
            CustomBinds,
            AssetBinds,
            Blacklists,
            DisabledChannels
        }

        struct RoGuildVisitor;

        impl<'de> Visitor<'de> for RoGuildVisitor {
            type Value = RoGuild;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("struct RoGuild")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut id = None;
                let mut prefix = None;
                let mut settings = None;
                let mut verification_role = None;
                let mut verified_role = None;
                let mut rankbinds = None::<Vec<RankBind>>;
                let mut groupbinds = None::<Vec<GroupBind>>;
                let mut custombinds = None::<Vec<CustomBind>>;
                let mut assetbinds = None::<Vec<AssetBind>>;
                let mut blacklists = None;
                let mut disabled_channels = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(DeError::duplicate_field("_id"));
                            }
                            id = Some(map.next_value()?);
                        },
                        Field::Prefix => {
                            if prefix.is_some() {
                                return Err(DeError::duplicate_field("Prefix"));
                            }
                            prefix = Some(map.next_value()?);
                        },
                        Field::Settings => {
                            if settings.is_some() {
                                return Err(DeError::duplicate_field("Settings"));
                            }
                            settings = Some(map.next_value()?);
                        },
                        Field::VerificationRole => {
                            if verification_role.is_some() {
                                return Err(DeError::duplicate_field("VerificationRole"));
                            }
                            verification_role = Some(map.next_value()?);
                        },
                        Field::VerifiedRole => {
                            if verified_role.is_some() {
                                return Err(DeError::duplicate_field("VerifiedRole"));
                            }
                            verified_role = Some(map.next_value()?);
                        },
                        Field::RankBinds => {
                            if rankbinds.is_some() {
                                return Err(DeError::duplicate_field("RankBinds"));
                            }
                            rankbinds = Some(map.next_value()?);
                        },
                        Field::GroupBinds => {
                            if groupbinds.is_some() {
                                return Err(DeError::duplicate_field("GroupBinds"));
                            }
                            groupbinds = Some(map.next_value()?);
                        },
                        Field::CustomBinds => {
                            if custombinds.is_some() {
                                return Err(DeError::duplicate_field("CustomBinds"));
                            }
                            custombinds = Some(map.next_value()?);
                        },
                        Field::AssetBinds => {
                            if assetbinds.is_some() {
                                return Err(DeError::duplicate_field("AssetBinds"));
                            }
                            assetbinds = Some(map.next_value()?);
                        },
                        Field::Blacklists => {
                            if blacklists.is_some() {
                                return Err(DeError::duplicate_field("Blacklists"));
                            }
                            blacklists = Some(map.next_value()?);
                        },
                        Field::DisabledChannels => {
                            if disabled_channels.is_some() {
                                return Err(DeError::duplicate_field("DisabledChannels"));
                            }
                            disabled_channels = Some(map.next_value()?);
                        }
                    }
                }

                let id = id.ok_or_else(|| DeError::missing_field("Id"))?;
                let prefix = prefix.ok_or_else(|| DeError::missing_field("Prefix"))?;
                let settings = settings.ok_or_else(|| DeError::missing_field("Settings"))?;
                let verification_role = verification_role.ok_or_else(|| DeError::missing_field("VerificationRole"))?;
                let verified_role = verified_role.ok_or_else(|| DeError::missing_field("VerifiedRole"))?;
                let rankbinds = rankbinds.ok_or_else(|| DeError::missing_field("RankBinds"))?;
                let groupbinds = groupbinds.ok_or_else(|| DeError::missing_field("GroupBinds"))?;
                let custombinds = custombinds.ok_or_else(|| DeError::missing_field("CustomBinds"))?;
                let assetbinds = assetbinds.ok_or_else(|| DeError::missing_field("AssetBinds"))?;
                let blacklists = blacklists.ok_or_else(|| DeError::missing_field("Blacklists"))?;
                let disabled_channels = disabled_channels.ok_or_else(|| DeError::missing_field("DisabledChannels"))?;
                let all_roles = rankbinds.iter()
                    .flat_map(|r| r.discord_roles.iter().cloned())
                    .chain(groupbinds.iter().flat_map(|g| g.discord_roles.iter().cloned()))
                    .chain(custombinds.iter().flat_map(|c| c.discord_roles.iter().cloned()))
                    .chain(assetbinds.iter().flat_map(|a| a.discord_roles.iter().cloned()))
                    .unique()
                    .collect_vec();
                Ok(RoGuild {
                    id,
                    command_prefix: prefix,
                    settings,
                    verification_role,
                    verified_role,
                    rankbinds,
                    groupbinds,
                    custombinds,
                    assetbinds,
                    blacklists,
                    disabled_channels,
                    all_roles
                })
            }
        }

        const FIELDS: &[&str] = &[
            "_id", "Prefix", "Settings", "VerificationRole", "VerifiedRole", "RankBinds", "GroupBinds", "CustomBinds", "AssetBinds",
            "Blacklists", "DisabledChannels"
        ];

        deserializer.deserialize_struct("RoGuild", FIELDS, RoGuildVisitor)
    }
}