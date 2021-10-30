mod backup;
mod settings;
mod types;

use itertools::Itertools;
use mongodb::bson::oid::ObjectId;
use serde::{
    de::{Deserializer, Error as DeError, IgnoredAny, MapAccess, Visitor},
    Deserialize, Serialize,
};
use std::{collections::HashMap, default::Default, fmt, sync::Arc};
use twilight_http::Client as DiscordClient;
use twilight_model::id::{ChannelId, GuildId, RoleId};

use super::{
    bind::{AssetBind, Backup, CustomBind, GroupBind, RankBind},
    blacklist::Blacklist,
    events::EventType,
};

pub use backup::*;
pub use settings::*;
pub use types::*;

#[derive(Debug, Serialize, Default, Clone)]
pub struct RoGuild {
    /// The id of the guild
    #[serde(rename = "_id")]
    pub id: i64,
    /// The prefix that is to be used by every command run in the guild
    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub command_prefix: Option<String>,
    /// The struct containing [GuildSettings]
    #[serde(rename = "Settings")]
    pub settings: GuildSettings,
    /// The role meant for unverified users in the guild
    #[serde(rename = "VerificationRole", skip_serializing_if = "Option::is_none")]
    pub verification_role: Option<i64>,
    /// The role meant for verified users in the guild
    #[serde(rename = "VerifiedRole", skip_serializing_if = "Option::is_none")]
    pub verified_role: Option<i64>,
    /// The array containing all the [RankBind] of the guild
    #[serde(rename = "RankBinds")]
    pub rankbinds: Vec<RankBind>,
    /// The array containing all the [GroupBind] of the guild
    #[serde(rename = "GroupBinds")]
    pub groupbinds: Vec<GroupBind>,
    /// The array containing all the [CustomBind] of the guild
    #[serde(rename = "CustomBinds")]
    pub custombinds: Vec<CustomBind>,
    /// The array containing all the [AssetBind] of the guild
    #[serde(rename = "AssetBinds")]
    pub assetbinds: Vec<AssetBind>,
    /// The array containing all the [Blacklist] of the guild
    #[serde(rename = "Blacklists")]
    pub blacklists: Vec<Blacklist>,
    /// The list of channels where commands are disabled
    #[serde(rename = "DisabledChannels")]
    pub disabled_channels: Vec<i64>,
    /// The list of groups that the guild uses for analytics
    #[serde(rename = "RegisteredGroups")]
    pub registered_groups: Vec<i64>,
    /// The list of [EventType] registered with the guild
    #[serde(rename = "EventTypes")]
    pub event_types: Vec<EventType>,
    /// The counter of how many events have been logged with the guild
    #[serde(rename = "EventCounter")]
    pub event_counter: i64,
    /// A non-serialized list of all roles that are covered by the binds
    #[serde(skip_serializing)]
    pub all_roles: Vec<i64>,
}

impl RoGuild {
    pub fn to_backup(
        &self,
        user_id: i64,
        name: &str,
        roles: &HashMap<RoleId, String>,
        channels: &HashMap<ChannelId, String>,
    ) -> BackupGuild {
        let rankbinds = self
            .rankbinds
            .iter()
            .map(|r| r.to_backup(roles))
            .collect_vec();
        let groupbinds = self
            .groupbinds
            .iter()
            .map(|g| g.to_backup(roles))
            .collect_vec();
        let custombinds = self
            .custombinds
            .iter()
            .map(|c| c.to_backup(roles))
            .collect_vec();
        let assetbinds = self
            .assetbinds
            .iter()
            .map(|a| a.to_backup(roles))
            .collect_vec();
        let verification_role = match self.verification_role {
            Some(verification_role) => roles
                .get(&RoleId::new(verification_role as u64).unwrap())
                .cloned(),
            None => None,
        };
        let verified_role = match self.verified_role {
            Some(verified_role) => roles
                .get(&RoleId::new(verified_role as u64).unwrap())
                .cloned(),
            None => None,
        };
        let backup_settings = self.settings.to_backup(roles, channels);

        BackupGuild {
            id: ObjectId::new(),
            user_id,
            name: name.to_string(),
            command_prefix: self.command_prefix.clone(),
            settings: backup_settings,
            verification_role,
            verified_role,
            rankbinds,
            groupbinds,
            custombinds,
            assetbinds,
            blacklists: self.blacklists.clone(),
            registered_groups: self.registered_groups.clone(),
            event_types: self.event_types.clone(),
        }
    }

    // Rewrite this part
    pub async fn from_backup(
        backup: BackupGuild,
        http: Arc<DiscordClient>,
        guild_id: GuildId,
        existing_roles: &[(RoleId, String)],
        existing_channels: &HashMap<String, ChannelId>,
    ) -> Self {
        let mut names_to_ids = HashMap::<String, RoleId>::new();

        let all_roles = backup
            .rankbinds
            .iter()
            .flat_map(|r| r.discord_roles.iter().cloned())
            .chain(
                backup
                    .groupbinds
                    .iter()
                    .flat_map(|g| g.discord_roles.iter().cloned()),
            )
            .chain(
                backup
                    .custombinds
                    .iter()
                    .flat_map(|c| c.discord_roles.iter().cloned()),
            )
            .chain(
                backup
                    .assetbinds
                    .iter()
                    .flat_map(|a| a.discord_roles.iter().cloned()),
            )
            .unique()
            .collect::<Vec<String>>();
        for role_name in all_roles {
            if let Some(r) = existing_roles
                .iter()
                .find(|r| r.1.eq_ignore_ascii_case(&role_name))
            {
                names_to_ids.insert(role_name, r.0);
            } else {
                #[rustfmt::skip]
                let role = http.create_role(guild_id).name(&role_name).exec().await.unwrap().model().await.unwrap();
                names_to_ids.insert(role.name, role.id);
            }
        }

        let rankbinds = backup
            .rankbinds
            .iter()
            .map(|bind| RankBind::from_backup(bind, &names_to_ids))
            .collect_vec();
        let groupbinds = backup
            .groupbinds
            .iter()
            .map(|bind| GroupBind::from_backup(bind, &names_to_ids))
            .collect_vec();
        let custombinds = backup
            .custombinds
            .iter()
            .map(|bind| CustomBind::from_backup(bind, &names_to_ids))
            .collect_vec();
        let assetbinds = backup
            .assetbinds
            .iter()
            .map(|bind| AssetBind::from_backup(bind, &names_to_ids))
            .collect_vec();

        let verification_role = if let Some(verification_name) = backup.verification_role {
            if let Some(r) = names_to_ids.get(&verification_name) {
                r.0.get() as i64
            } else if let Some(r) = existing_roles.iter().find(|e| e.1.eq(&verification_name)) {
                (r.0).0.get() as i64
            } else {
                #[rustfmt::skip]
                let role = http.create_role(guild_id).name(&verification_name).exec().await.unwrap().model().await.unwrap();
                role.id.0.get() as i64
            }
        } else {
            0
        };

        let verified_role = if let Some(verified_name) = backup.verified_role {
            if let Some(r) = names_to_ids.get(&verified_name) {
                r.0.get() as i64
            } else if let Some(r) = existing_roles.iter().find(|e| e.1.eq(&verified_name)) {
                (r.0).0.get() as i64
            } else {
                #[rustfmt::skip]
                let role = http.create_role(guild_id).name(&verified_name).exec().await.unwrap().model().await.unwrap();
                role.id.0.get() as i64
            }
        } else {
            0
        };

        let all_roles = rankbinds
            .iter()
            .flat_map(|r| r.discord_roles.iter().copied())
            .chain(
                groupbinds
                    .iter()
                    .flat_map(|g| g.discord_roles.iter().copied()),
            )
            .chain(
                custombinds
                    .iter()
                    .flat_map(|c| c.discord_roles.iter().copied()),
            )
            .chain(
                assetbinds
                    .iter()
                    .flat_map(|a| a.discord_roles.iter().copied()),
            )
            .unique()
            .collect_vec();
        let settings = GuildSettings::from_backup(
            http,
            backup.settings,
            guild_id,
            &names_to_ids,
            existing_roles,
            existing_channels,
        )
        .await;

        Self {
            id: guild_id.0.get() as i64,
            command_prefix: backup.command_prefix,
            settings,
            verification_role: Some(verification_role),
            verified_role: Some(verified_role),
            rankbinds,
            groupbinds,
            custombinds,
            assetbinds,
            blacklists: backup.blacklists,
            disabled_channels: Vec::new(),
            registered_groups: backup.registered_groups,
            event_types: backup.event_types,
            event_counter: 0,
            all_roles,
        }
    }
}

impl<'de> Deserialize<'de> for RoGuild {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier)]
        enum Field {
            #[serde(rename = "_id")]
            Id,
            Prefix,
            Settings,
            VerificationRole,
            VerifiedRole,
            RankBinds,
            GroupBinds,
            CustomBinds,
            AssetBinds,
            Blacklists,
            DisabledChannels,
            RegisteredGroups,
            EventTypes,
            EventCounter,
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
                let mut registered_groups = None;
                let mut event_types = None;
                let mut event_counter = None;

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
                        Field::Prefix => {
                            if prefix.is_some() {
                                return Err(DeError::duplicate_field("Prefix"));
                            }
                            prefix = Some(map.next_value()?);
                        }
                        Field::Settings => {
                            if settings.is_some() {
                                return Err(DeError::duplicate_field("Settings"));
                            }
                            settings = Some(map.next_value()?);
                        }
                        Field::VerificationRole => {
                            if verification_role.is_some() {
                                return Err(DeError::duplicate_field("VerificationRole"));
                            }
                            verification_role = Some(map.next_value()?);
                        }
                        Field::VerifiedRole => {
                            if verified_role.is_some() {
                                return Err(DeError::duplicate_field("VerifiedRole"));
                            }
                            verified_role = Some(map.next_value()?);
                        }
                        Field::RankBinds => {
                            if rankbinds.is_some() {
                                return Err(DeError::duplicate_field("RankBinds"));
                            }
                            rankbinds = Some(map.next_value()?);
                        }
                        Field::GroupBinds => {
                            if groupbinds.is_some() {
                                return Err(DeError::duplicate_field("GroupBinds"));
                            }
                            groupbinds = Some(map.next_value()?);
                        }
                        Field::CustomBinds => {
                            if custombinds.is_some() {
                                return Err(DeError::duplicate_field("CustomBinds"));
                            }
                            custombinds = Some(map.next_value()?);
                        }
                        Field::AssetBinds => {
                            if assetbinds.is_some() {
                                return Err(DeError::duplicate_field("AssetBinds"));
                            }
                            assetbinds = Some(map.next_value()?);
                        }
                        Field::Blacklists => {
                            if blacklists.is_some() {
                                return Err(DeError::duplicate_field("Blacklists"));
                            }
                            blacklists = Some(map.next_value()?);
                        }
                        Field::DisabledChannels => {
                            if disabled_channels.is_some() {
                                return Err(DeError::duplicate_field("DisabledChannels"));
                            }
                            disabled_channels = Some(map.next_value()?);
                        }
                        Field::RegisteredGroups => {
                            if registered_groups.is_some() {
                                return Err(DeError::duplicate_field("RegisteredGroups"));
                            }
                            registered_groups = Some(map.next_value()?);
                        }
                        Field::EventTypes => {
                            if event_types.is_some() {
                                return Err(DeError::duplicate_field("EventTypes"));
                            }
                            event_types = Some(map.next_value()?);
                        }
                        Field::EventCounter => {
                            if event_counter.is_some() {
                                return Err(DeError::duplicate_field("EventCounter"));
                            }
                            event_counter = Some(map.next_value()?);
                        }
                    }
                }

                let id = id.ok_or_else(|| DeError::missing_field("Id"))?;
                let settings = settings.ok_or_else(|| DeError::missing_field("Settings"))?;
                let verification_role = verification_role.unwrap_or_default();
                let verified_role = verified_role.unwrap_or_default();
                let rankbinds = rankbinds.unwrap_or_default();
                let groupbinds = groupbinds.unwrap_or_default();
                let custombinds = custombinds.unwrap_or_default();
                let assetbinds = assetbinds.unwrap_or_default();
                let blacklists = blacklists.unwrap_or_default();
                let disabled_channels = disabled_channels.unwrap_or_default();
                let registered_groups = registered_groups.unwrap_or_default();
                let event_types = event_types.unwrap_or_default();
                let event_counter = event_counter.unwrap_or_default();
                let all_roles = rankbinds
                    .iter()
                    .flat_map(|r| r.discord_roles.iter().copied())
                    .chain(
                        groupbinds
                            .iter()
                            .flat_map(|g| g.discord_roles.iter().copied()),
                    )
                    .chain(
                        custombinds
                            .iter()
                            .flat_map(|c| c.discord_roles.iter().copied()),
                    )
                    .chain(
                        assetbinds
                            .iter()
                            .flat_map(|a| a.discord_roles.iter().copied()),
                    )
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
                    registered_groups,
                    event_types,
                    event_counter,
                    all_roles,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "_id",
            "Prefix",
            "Settings",
            "VerificationRole",
            "VerifiedRole",
            "RankBinds",
            "GroupBinds",
            "CustomBinds",
            "AssetBinds",
            "Blacklists",
            "DisabledChannels",
            "RegisteredGroups",
            "EventTypes",
            "EventCounter",
        ];

        deserializer.deserialize_struct("RoGuild", FIELDS, RoGuildVisitor)
    }
}

impl_redis!(RoGuild);
