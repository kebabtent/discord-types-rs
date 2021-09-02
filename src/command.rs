use crate::{ChannelId, CowString, GuildId, Intents, Status, UserId};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Debug)]
pub enum Command {
	Heartbeat(Heartbeat),
	Identify(Identify),
	UpdateStatus(UpdateStatus),
	UpdateVoiceState(UpdateVoiceState),
	Resume(Resume),
	RequestGuildMembers(RequestGuildMembers),
}

impl Command {
	pub fn op(&self) -> u8 {
		match self {
			Command::Heartbeat(_) => 1,
			Command::Identify(_) => 2,
			Command::UpdateStatus(_) => 3,
			Command::UpdateVoiceState(_) => 4,
			Command::Resume(_) => 6,
			Command::RequestGuildMembers(_) => 8,
		}
	}

	pub fn is_heartbeat(&self) -> bool {
		match self {
			Command::Heartbeat(_) => true,
			_ => false,
		}
	}

	pub fn heartbeat<T: Into<Heartbeat>>(heartbeat: T) -> Self {
		Command::Heartbeat(heartbeat.into())
	}
}

impl fmt::Display for Command {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Command::Identify(_) => write!(f, "Identify"),
			Command::Resume(_) => write!(f, "Resume"),
			Command::Heartbeat(c) => write!(f, "Heartbeat({})", c),
			Command::RequestGuildMembers(_) => write!(f, "RequestGuildMembers"),
			Command::UpdateVoiceState(_) => write!(f, "UpdateVoiceState"),
			Command::UpdateStatus(_) => write!(f, "UpdateStatus"),
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
			Command::Heartbeat(d) => state.serialize_field("d", d)?,
			Command::Identify(d) => state.serialize_field("d", d)?,
			Command::UpdateStatus(d) => state.serialize_field("d", d)?,
			Command::UpdateVoiceState(d) => state.serialize_field("d", d)?,
			Command::Resume(d) => state.serialize_field("d", d)?,
			Command::RequestGuildMembers(d) => state.serialize_field("d", d)?,
		}
		state.end()
	}
}

#[derive(Clone, Debug, Serialize)]
pub struct Identify {
	pub token: CowString,
	pub properties: ConnectionProperties,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub compress: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub large_threshold: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub shard: Option<(u8, u8)>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub presence: Option<UpdateStatus>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub guild_subscriptions: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub intents: Option<Intents>,
}

impl From<Identify> for Command {
	fn from(identify: Identify) -> Command {
		Command::Identify(identify)
	}
}

#[derive(Clone, Debug, Serialize)]
pub struct Resume {
	pub token: CowString,
	pub session_id: CowString,
	pub seq: u64,
}

impl From<Resume> for Command {
	fn from(resume: Resume) -> Command {
		Command::Resume(resume)
	}
}

#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct Heartbeat {
	pub sequence: u64,
}

impl fmt::Display for Heartbeat {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.sequence)
	}
}

impl<'a> From<Heartbeat> for Command {
	fn from(heartbeat: Heartbeat) -> Command {
		Command::Heartbeat(heartbeat)
	}
}

impl From<u64> for Heartbeat {
	fn from(sequence: u64) -> Self {
		Heartbeat { sequence }
	}
}

#[derive(Clone, Debug, Serialize)]
pub struct RequestGuildMembers {
	pub guild_id: GuildId,
	pub query: CowString,
	pub limit: u32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub presences: Option<bool>,
	#[serde(skip_serializing_if = "HashSet::is_empty")]
	pub user_ids: HashSet<UserId>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub nonce: Option<CowString>,
}

impl From<RequestGuildMembers> for Command {
	fn from(request: RequestGuildMembers) -> Command {
		Command::RequestGuildMembers(request)
	}
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateVoiceState {
	pub guild_id: GuildId,
	pub channel_id: Option<ChannelId>,
	pub self_mute: bool,
	pub self_deaf: bool,
}

impl From<UpdateVoiceState> for Command {
	fn from(update: UpdateVoiceState) -> Command {
		Command::UpdateVoiceState(update)
	}
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateStatus {
	pub since: Option<u64>,
	// pub activities: Vec<Activity>,
	pub status: Status,
	pub afk: bool,
}

impl From<UpdateStatus> for Command {
	fn from(update: UpdateStatus) -> Command {
		Command::UpdateStatus(update)
	}
}

#[derive(Clone, Debug, Serialize)]
pub struct ConnectionProperties {
	#[serde(rename = "$os")]
	pub os: CowString,
	#[serde(rename = "$browser")]
	pub browser: CowString,
	#[serde(rename = "$device")]
	pub device: CowString,
}
