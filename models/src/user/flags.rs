use bitflags::bitflags;
use bytes::BytesMut;
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

bitflags! {
    pub struct UserFlags: i64 {
        const NONE = 0;
        const ALPHA = 1;
        const BETA = 1 << 1;
        const STAFF = 1 << 2;
        const PARTNER = 1 << 3;
    }
}

impl<'a> FromSql<'a> for UserFlags {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let bits = i64::from_sql(ty, raw)?;
        Ok(Self::from_bits_truncate(bits))
    }

    fn accepts(ty: &Type) -> bool {
        <i64 as FromSql>::accepts(ty)
    }
}

impl ToSql for UserFlags {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        i64::to_sql(&self.bits, ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i64 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl<'de> Deserialize<'de> for UserFlags {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_bits_truncate(i64::deserialize(deserializer)?))
    }
}

impl Serialize for UserFlags {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i64(self.bits())
    }
}
