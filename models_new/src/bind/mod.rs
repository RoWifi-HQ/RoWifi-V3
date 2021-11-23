mod rank;
mod template;
mod asset;
mod group;
mod custom;

pub use rank::Rankbind;
pub use template::Template;

use bytes::BytesMut;
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};

use crate::FromRow;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Bind {
    Rank(Rankbind),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BindType {
    Rank = 0,
    Group = 1,
    Custom = 2,
    Asset = 3
}

impl FromRow for Bind {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let kind = row.try_get("type")?;
        match kind {
            BindType::Rank => Ok(Bind::Rank(Rankbind::from_row(row)?)),
            _ => todo!()
        }
    }
}

impl ToSql for BindType {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        i32::to_sql(&(*self as i32), ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i32 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl<'a> FromSql<'a> for BindType {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let bind_type = i32::from_sql(ty, raw)?;
        match bind_type {
            0 => Ok(BindType::Rank),
            _ => unreachable!(),
        }
    }

    fn accepts(ty: &Type) -> bool {
        <i32 as FromSql>::accepts(ty)
    }
}
