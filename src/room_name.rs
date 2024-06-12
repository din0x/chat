use std::fmt::Display;

use rand::random;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct RoomName {
    #[serde(deserialize_with = "deserialize")]
    inner: Box<str>,
}

impl Display for RoomName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<&str> for RoomName {
    fn from(s: &str) -> Self {
        if !s.trim().is_empty() {
            return RoomName { inner: s.into() };
        }

        RoomName {
            inner: format!("room-{}", random::<usize>()).into(),
        }
    }
}

fn deserialize<'de, D>(deserializer: D) -> Result<Box<str>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;

    if !s.trim().is_empty() {
        return Ok(s.into());
    }

    Ok(format!("room-{}", random::<usize>()).into())
}
