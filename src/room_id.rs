use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub struct RoomId {
    id: u32,
}

impl RoomId {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

impl FromStr for RoomId {
    type Err = RoomIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match u32::from_str_radix(s, 16) {
            Ok(id) => Ok(RoomId::new(id)),
            Err(_) => Err(RoomIdParseError),
        }
    }
}

impl Display for RoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.id)
    }
}

#[derive(Debug)]
pub struct RoomIdParseError;
