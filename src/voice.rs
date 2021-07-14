pub use self::command::Command;
use crate::types::SpeakingFlags;
use crate::{GuildId, UserId};
use serde::de;
use serde::de::{IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Debug)]
pub struct EventError;

impl fmt::Display for EventError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Event error")
	}
}

impl std::error::Error for EventError {}

#[derive(Clone, Debug)]
pub enum Event {
	Hello(Hello),
	Ready(Ready),
	Resumed,
	HeartbeatAck(HeartbeatAck),
	SessionDescription(SessionDescription),
	Speaking(Speaking),
	// ClientDisconnect(ClientDisconnect),
	Unknown(u8),
}

impl Event {
	pub fn expect_hello(self) -> Result<Hello, EventError> {
		match self {
			Event::Hello(event) => Ok(event),
			_ => Err(EventError),
		}
	}

	pub fn expect_ready(self) -> Result<Ready, EventError> {
		match self {
			Event::Ready(ready) => Ok(ready),
			_ => Err(EventError),
		}
	}

	pub fn expect_resumed(self) -> Result<(), EventError> {
		match self {
			Event::Resumed => Ok(()),
			_ => Err(EventError),
		}
	}

	pub fn is_heartbeat_ack(&self) -> bool {
		match self {
			Event::HeartbeatAck(_) => true,
			_ => false,
		}
	}

	pub fn is_ready_kind(&self) -> bool {
		match self {
			Event::Ready(_) | Event::Resumed => true,
			_ => false,
		}
	}
}

impl<'de> Deserialize<'de> for Event {
	fn deserialize<D>(deserializer: D) -> Result<Event, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct EventVisitor;

		impl<'de> Visitor<'de> for EventVisitor {
			type Value = Event;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("struct Event")
			}

			fn visit_map<V>(self, mut map: V) -> Result<Event, V::Error>
			where
				V: MapAccess<'de>,
			{
				let mut op: Option<u8> = None;
				let mut event = None;
				while let Some(key) = map.next_key()? {
					match key {
						"op" => {
							if op.is_some() {
								return Err(de::Error::duplicate_field("op"));
							}
							op = Some(map.next_value()?);
						}
						"d" => {
							if event.is_some() {
								return Err(de::Error::duplicate_field("d"));
							}

							let op = op.ok_or_else(|| de::Error::missing_field("op"))?;
							let ev = match op {
								2 => Event::Ready(map.next_value()?),
								4 => Event::SessionDescription(map.next_value()?),
								5 => Event::Speaking(map.next_value()?),
								6 => Event::HeartbeatAck(map.next_value()?),
								8 => Event::Hello(map.next_value()?),
								9 => {
									map.next_value::<IgnoredAny>()?;
									Event::Resumed
								}
								// 13 => Event::ClientDisconnect(map.next_value()?),
								e => {
									map.next_value::<IgnoredAny>()?;
									Event::Unknown(e)
								}
							};
							event = Some(ev);
						}
						_ => {
							map.next_value::<IgnoredAny>()?;
						}
					}
				}
				event.ok_or_else(|| de::Error::missing_field("d"))
			}
		}

		deserializer.deserialize_struct("Event", &["op", "d"], EventVisitor)
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct Hello {
	pub heartbeat_interval: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Ready {
	pub ssrc: u32,
	pub ip: String,
	pub port: u16,
	pub modes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct HeartbeatAck {
	pub nonce: u64,
}

#[derive(Clone, Deserialize)]
pub struct SessionDescription {
	pub mode: String,
	pub secret_key: [u8; 32],
}

impl fmt::Debug for SessionDescription {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("SessionDescription")
			.field("mode", &self.mode)
			.finish()
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Speaking {
	pub speaking: SpeakingFlags,
	#[serde(default)]
	pub delay: u32,
	pub ssrc: u32,
}

pub mod command {
	use super::*;
	use serde::ser::SerializeStruct;
	use serde::{Serialize, Serializer};

	#[derive(Clone, Debug)]
	pub enum Command {
		Identify(Identify),
		SelectProtocol(SelectProtocol),
		Heartbeat(Heartbeat),
		Speaking(Speaking),
		Resume(Resume),
	}

	impl Command {
		pub fn op(&self) -> u8 {
			match self {
				Command::Identify(_) => 0,
				Command::SelectProtocol(_) => 1,
				Command::Heartbeat(_) => 3,
				Command::Speaking(_) => 5,
				Command::Resume(_) => 7,
			}
		}

		pub fn is_heartbeat(&self) -> bool {
			match self {
				Command::Heartbeat(_) => true,
				_ => false,
			}
		}
	}

	impl Serialize for Command {
		fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: Serializer,
		{
			let mut state = serializer.serialize_struct("Command", 2)?;
			state.serialize_field("op", &self.op())?;
			match self {
				Command::Identify(c) => state.serialize_field("d", &c)?,
				Command::SelectProtocol(c) => state.serialize_field("d", &c)?,
				Command::Heartbeat(c) => state.serialize_field("d", &c)?,
				Command::Speaking(c) => state.serialize_field("d", &c)?,
				Command::Resume(c) => state.serialize_field("d", &c)?,
			}
			state.end()
		}
	}

	#[derive(Clone, Debug, Serialize)]
	pub struct Identify {
		#[serde(rename = "server_id")]
		pub guild_id: GuildId,
		pub user_id: UserId,
		pub session_id: String,
		pub token: String,
	}

	impl From<Identify> for Command {
		fn from(identify: Identify) -> Self {
			Command::Identify(identify)
		}
	}

	#[derive(Clone, Debug, Serialize)]
	pub struct Resume {
		#[serde(rename = "server_id")]
		pub guild_id: GuildId,
		pub session_id: String,
		pub token: String,
	}

	impl From<Resume> for Command {
		fn from(resume: Resume) -> Self {
			Command::Resume(resume)
		}
	}

	#[derive(Clone, Debug, Serialize)]
	#[serde(transparent)]
	pub struct Heartbeat {
		pub nonce: u64,
	}

	impl Heartbeat {
		pub fn new(nonce: u64) -> Self {
			Self { nonce }
		}
	}

	impl From<Heartbeat> for Command {
		fn from(heartbeat: Heartbeat) -> Self {
			Command::Heartbeat(heartbeat)
		}
	}

	#[derive(Clone, Debug, Serialize)]
	pub struct SelectProtocol {
		pub protocol: String,
		pub data: SelectProtocolData,
	}

	#[derive(Clone, Debug, Serialize)]
	pub struct SelectProtocolData {
		pub address: String,
		pub port: u16,
		pub mode: String,
	}
}
