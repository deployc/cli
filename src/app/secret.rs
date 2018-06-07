use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

pub enum SecretType {
    Raw,
    Certificate,
    Credentials,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: SecretType,
    pub created_at: DateTime<Utc>,
}

impl SecretType {
    pub fn as_str(&self) -> &str {
        match self {
            SecretType::Raw => "raw",
            SecretType::Certificate => "certificate",
            SecretType::Credentials => "credentials",
        }
    }

    pub fn is_raw(&self) -> bool {
        match self {
            SecretType::Raw => true,
            _ => false,
        }
    }
}

impl FromStr for SecretType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "raw" => Ok(SecretType::Raw),
            "certificate" => Ok(SecretType::Certificate),
            "credentials" => Ok(SecretType::Credentials),
            _ => Err(format!(
                "Expected 'raw', 'certificate', or 'credentials'. Found '{}'",
                s
            )),
        }
    }
}

impl Serialize for SecretType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SecretType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SecretType::from_str(&s).map_err(|err| de::Error::custom(err))
    }
}
