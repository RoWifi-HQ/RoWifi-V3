#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::if_not_else,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::match_on_vec_items,
    clippy::map_err_ignore,
    clippy::missing_panics_doc
)]

macro_rules! impl_redis {
    ($ty: ty) => {
        impl rowifi_redis::redis::FromRedisValue for $ty {
            fn from_redis_value(
                v: &rowifi_redis::redis::Value,
            ) -> rowifi_redis::redis::RedisResult<Self> {
                use rowifi_redis::redis::{ErrorKind, RedisError, Value};

                if let Value::Data(bytes) = v {
                    let res = serde_cbor::from_slice::<Self>(bytes).map_err(|err| {
                        RedisError::from((
                            ErrorKind::TypeError,
                            "Deserialization Error",
                            err.to_string(),
                        ))
                    });
                    return res;
                }
                Err(RedisError::from((
                    ErrorKind::TypeError,
                    "Invalid Redis Value",
                )))
            }
        }

        impl rowifi_redis::redis::ToRedisArgs for $ty {
            fn write_redis_args<W>(&self, out: &mut W)
            where
                W: ?Sized + rowifi_redis::redis::RedisWrite,
            {
                let res = serde_cbor::to_vec(self).unwrap();
                out.write_arg(&res);
            }
        }
    };
}

pub mod analytics;
pub mod bind;
pub mod blacklist;
pub mod events;
pub mod guild;
pub mod roblox;
pub mod rolang;
pub mod stats;
pub mod user;
