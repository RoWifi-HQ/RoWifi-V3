use std::fmt::{Display, Formatter, Result as FmtResult};

pub enum Route<'a> {
    GroupRoles {
        group_id: u64,
    },
    UserInventoryAsset {
        user_id: u64,
        asset_id: u64,
        asset_type: &'a str,
    },
    UserById {
        user_id: u64,
    },
    UsersById,
    UsersByUsername,
    UserGroupRoles {
        user_id: u64,
    },
}

impl Display for Route<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Route::GroupRoles { group_id } => {
                write!(f, "https://groups.roblox.com/v1/groups/{}/roles", group_id)
            }
            Route::UserInventoryAsset {
                user_id,
                asset_id,
                asset_type,
            } => write!(
                f,
                "https://inventory.roblox.com/v1/users/{}/items/{}/{}",
                user_id, asset_id, asset_type
            ),
            Route::UserById { user_id } => {
                write!(f, "https://users.roblox.com/v1/users/{}", user_id)
            }
            Route::UsersById => write!(f, "https://users.roblox.com/v1/users"),
            Route::UsersByUsername => write!(f, "https://users.roblox.com/v1/usernames/users"),
            Route::UserGroupRoles { user_id } => write!(
                f,
                "https://groups.roblox.com/v2/users/{}/groups/roles",
                user_id
            ),
        }
    }
}
