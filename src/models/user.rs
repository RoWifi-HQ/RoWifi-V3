use itertools::Itertools;
use serde::{Serialize, Deserialize};
use twilight::{
    http::Client as Http,
    model::id::RoleId
};
use std::{sync::Arc, borrow::Cow, collections::HashSet};

use crate::utils::{Roblox, error::RoError};
use crate::cache::{CachedGuild, CachedMember};
use super::{guild::{RoGuild, BlacklistActionType}, command::RoCommandUser};

#[derive(Debug, Serialize, Deserialize)]
pub struct RoUser {
    #[serde(rename = "_id")]
    pub discord_id: i64,

    #[serde(rename = "RobloxId")]
    pub roblox_id: i64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueueUser{
    #[serde(rename = "_id")]
    pub roblox_id: i64,

    #[serde(rename = "DiscordId")]
    pub discord_id: i64,

    #[serde(rename = "Verified")]
    pub verified: bool
}

impl RoUser {
    pub async fn update(&self, http: impl AsRef<Http>, member: Arc<CachedMember>, rbx: Arc<Roblox>, server: Arc<CachedGuild>, guild: &RoGuild, guild_roles: HashSet<RoleId>) 
        -> Result<(Vec<RoleId>, Vec<RoleId>, String), RoError> {
        let mut added_roles = Vec::<RoleId>::new();
        let mut removed_roles = Vec::<RoleId>::new();

        let verification_role = RoleId(guild.verification_role as u64);
        if guild_roles.get(&verification_role).is_some() && member.roles.contains(&verification_role) {
            removed_roles.push(verification_role);
        }

        let verified_role = RoleId(guild.verified_role as u64);
        if guild_roles.get(&verified_role).is_some() && !member.roles.contains(&verified_role) {
            added_roles.push(verified_role);
        }

        let user_roles = rbx.get_user_roles(self.roblox_id).await?;
        let username = rbx.get_username_from_id(self.roblox_id).await?;
        let command_user = RoCommandUser {user: &self, member: Arc::clone(&member), ranks: &user_roles, username: &username };

        if !guild.blacklists.is_empty() {
            let success = guild.blacklists.iter().find(|b| b.evaluate(&command_user).unwrap());
            if let Some(success) = success {
                match guild.settings.blacklist_action {
                    BlacklistActionType::None => {},
                    BlacklistActionType::Kick => {let _ = http.as_ref().remove_guild_member(server.id, member.user.id).await;},
                    BlacklistActionType::Ban => {let _ = http.as_ref().create_ban(server.id, member.user.id).await;},
                };
                return Err(RoError::Blacklist(success.reason.to_owned()))
            }
        }

        let rankbinds_to_add = guild.rankbinds.iter().filter(|r| 
            match user_roles.get(&r.group_id) {
                Some(rank_id) => *rank_id == r.rank_id as i64,
                None => r.rank_id == 0
            }
        ).collect_vec();
        let groupbinds_to_add = guild.groupbinds.iter().filter(|g| 
            user_roles.contains_key(&g.group_id)
        );
        let custombinds_to_add = guild.custombinds.iter().filter(|c| 
            c.command.evaluate(&command_user).unwrap()
        ).collect_vec();
        let mut assetbinds_to_add = Vec::new();
        for asset in &guild.assetbinds {
            if rbx.has_asset(self.roblox_id, asset.id, &asset.asset_type.to_string()).await? {
                assetbinds_to_add.push(asset);
            }
        }

        let roles_to_add = rankbinds_to_add.iter()
            .flat_map(|r| r.discord_roles.iter().map(|r| Cow::Borrowed(r)))
            .chain(groupbinds_to_add.flat_map(|g| g.discord_roles.iter().map(|r| Cow::Borrowed(r))))
            .chain(custombinds_to_add.iter().flat_map(|c| c.discord_roles.iter().map(|r| Cow::Borrowed(r))))
            .chain(assetbinds_to_add.iter().flat_map(|a| a.discord_roles.iter().map(|r| Cow::Borrowed(r))))
            .collect_vec();

        let all_roles = guild.rankbinds.iter()
            .flat_map(|r| r.discord_roles.iter().map(|r| Cow::Borrowed(r)))
            .chain(guild.groupbinds.iter().flat_map(|g| g.discord_roles.iter().map(|r| Cow::Borrowed(r))))
            .chain(guild.custombinds.iter().flat_map(|c| c.discord_roles.iter().map(|r| Cow::Borrowed(r))))
            .chain(guild.assetbinds.iter().flat_map(|a| a.discord_roles.iter().map(|r| Cow::Borrowed(r))))
            .unique()
            .collect_vec();

        for bind_role in all_roles {
            let r = RoleId(*bind_role as u64);
            if let Some(_role) = guild_roles.get(&r) {
                if roles_to_add.contains(&bind_role) {
                    if !member.roles.contains(&r) {
                        added_roles.push(r);
                    }
                } else if member.roles.contains(&r) {
                    removed_roles.push(r);
                }
            }
        }

        let nick_bind = rankbinds_to_add.iter().sorted_by_key(|r| -r.priority).next();
        let custom = custombinds_to_add.iter().sorted_by_key(|c| -c.priority).next();

        let prefix: &str;
        if nick_bind.is_none() && custom.is_none() {prefix = "N/A";}
        else if nick_bind.is_none() {prefix = &custom.unwrap().prefix;}
        else if custom.is_none() {prefix = &nick_bind.unwrap().prefix;}
        else {prefix = if custom.unwrap().priority > nick_bind.unwrap().priority {&custom.unwrap().prefix} else {&nick_bind.unwrap().prefix}; }

        let display_name = member.nick.as_ref().map_or_else(|| Cow::Owned(member.user.name.clone()), Cow::Borrowed);
        let mut disc_nick = display_name.clone();
        if prefix.eq_ignore_ascii_case("N/A") {disc_nick = Cow::Borrowed(&username);}
        else if prefix.eq_ignore_ascii_case("Disable") {}
        else {disc_nick = Cow::Owned(format!("{} {}", prefix, username));}

        if disc_nick.len() > 32 {
            return Err(RoError::NicknameTooLong(format!("The supposed nickname {} was found to be more than 32 characters", disc_nick)))
        }

        let update = http.as_ref().update_guild_member(server.id, member.user.id);
        println!("{:?} {:?}", added_roles, removed_roles);
        let role_changes = !added_roles.is_empty() || !removed_roles.is_empty();
        let mut roles = member.roles.clone();
        roles.extend_from_slice(&added_roles);
        roles.retain(|r| !removed_roles.contains(r));
        println!("{:?} {:?}", added_roles, removed_roles);
        
        let nick_changes = false;//disc_nick != display_name;
        

        if role_changes || nick_changes {
            //update.roles(roles).nick(disc_nick.to_string()).unwrap().await?;
            update.roles(roles).await?;
        }

        Ok((added_roles, removed_roles, disc_nick.to_string()))
    }
}