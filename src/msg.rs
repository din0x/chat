use crate::room_name::RoomName;

use super::room_id::{RoomId, RoomIdParseError};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[derive(Serialize, Deserialize, Debug)]
pub enum FromClient {
    Name(Box<str>),
    Message(Box<str>),
    Create(RoomName),
    Join(RoomId),
    Leave,
}

impl FromStr for FromClient {
    type Err = ClientMessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("!!") || !s.starts_with('!') {
            return Ok(Self::Message(s.strip_prefix("!!").unwrap_or(s).into()));
        }

        let s = s.split_at(1).1;

        if s.starts_with("create") {
            let name = s.split_at(6).1.trim();

            return Ok(Self::Create(name.into()));
        } else if s.starts_with("join") {
            let id = RoomId::from_str(s.split_at(4).1.trim());

            return match id {
                Ok(id) => Ok(Self::Join(id)),
                Err(err) => Err(Self::Err::RoomIdParseError(err)),
            };
        } else if s.starts_with("leave") {
            return if s.split_at(5).1.trim().is_empty() {
                Ok(Self::Leave)
            } else {
                Err(Self::Err::TooManyArgs)
            };
        } else if s.starts_with("name") {
            let name = s.split_at(4).1.trim();

            return Ok(Self::Name(name.into()));
        }

        Err(Self::Err::Unknown)
    }
}

pub enum ClientMessageParseError {
    Unknown,
    RoomIdParseError(RoomIdParseError),
    TooManyArgs,
}

impl Display for ClientMessageParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown command"),
            Self::RoomIdParseError(_) => write!(f, "Invalid room id"),
            Self::TooManyArgs => write!(f, "Too many arguments"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum FromServer {
    NewRoom(RoomId),
    Joined(RoomName),
    Message { name: Box<str>, msg: Box<str> },
    Sent,
    Left,
    RoomNotFound,
    NotJoined,
    Renamed,
    Error,
}
