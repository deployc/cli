use std::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

pub enum RestartPolicy {
    Never,
    Always,
    OnFailure(u64),
}

impl Default for RestartPolicy {
    fn default() -> RestartPolicy {
        RestartPolicy::OnFailure(3)
    }
}

impl Serialize for RestartPolicy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RestartPolicy::Never => serializer.serialize_str("never"),
            RestartPolicy::Always => serializer.serialize_str("always"),
            RestartPolicy::OnFailure(t) => serializer.serialize_u64(*t),
        }
    }
}

impl<'de> Deserialize<'de> for RestartPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'v> Visitor<'v> for ValueVisitor {
            type Value = RestartPolicy;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "'always', 'never', or integer")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v.to_lowercase().as_str() {
                    "never" => Ok(RestartPolicy::Never),
                    "always" => Ok(RestartPolicy::Always),
                    _ => Err(de::Error::custom(format!(
                        "Invalid value for restart: {}",
                        v
                    ))),
                }
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(RestartPolicy::OnFailure(v))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
