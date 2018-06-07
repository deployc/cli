use std::fmt;

use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};

#[derive(Clone)]
pub enum EnvVar {
    Value(String),
    Secret(String),
}

impl Serialize for EnvVar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            EnvVar::Value(s) => serializer.serialize_str(s),
            EnvVar::Secret(s) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("secret", s)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for EnvVar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Secret,
        };

        impl<'v> Visitor<'v> for ValueVisitor {
            type Value = EnvVar;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "string or secret(x)")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(EnvVar::Value(v.to_string()))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'v>,
            {
                let mut secret = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Secret => {
                            if secret.is_some() {
                                return Err(de::Error::duplicate_field("secret"));
                            }
                            secret = Some(map.next_value()?);
                        }
                    }
                }

                let secret = secret.ok_or_else(|| de::Error::missing_field("secret"))?;
                Ok(EnvVar::Secret(secret))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
