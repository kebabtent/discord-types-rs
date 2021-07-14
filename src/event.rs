use crate::*;
use serde::de;
use serde::de::{IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashSet;
use std::fmt;

#[derive(Debug)]
pub struct EventError;

#[derive(Clone, Debug)]
pub struct Payload {
	pub sequence: Option<u64>,
	pub event: Event,
}

impl fmt::Display for Payload {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.sequence {
			Some(seq) => write!(f, "{}@{}", self.event, seq),
			None => write!(f, "{}", self.event),
		}
	}
}

impl<'de> Deserialize<'de> for Payload {
	fn deserialize<D>(deserializer: D) -> Result<Payload, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct PayloadVisitor;

		impl<'de> Visitor<'de> for PayloadVisitor {
			type Value = Payload;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("struct Payload")
			}

			fn visit_map<V>(self, mut map: V) -> Result<Payload, V::Error>
			where
				V: MapAccess<'de>,
			{
				let mut t: Option<String> = None;
				let mut sequence = None;
				let mut op: Option<u8> = None;
				let mut payload = None;
				while let Some(key) = map.next_key()? {
					match key {
						"t" => {
							if t.is_some() {
								return Err(de::Error::duplicate_field("t"));
							}
							t = map.next_value()?;
						}
						"s" => {
							if sequence.is_some() {
								return Err(de::Error::duplicate_field("s"));
							}
							sequence = map.next_value()?;
						}
						"op" => {
							if op.is_some() {
								return Err(de::Error::duplicate_field("op"));
							}
							op = Some(map.next_value()?);
						}
						"d" => {
							if payload.is_some() {
								return Err(de::Error::duplicate_field("d"));
							}

							let op = op.ok_or_else(|| de::Error::missing_field("op"))?;
							let event = match (op, t.as_deref()) {
								(9, _) => Event::InvalidSession(map.next_value()?),
								(10, _) => Event::Hello(map.next_value()?),
								(11, _) => {
									map.next_value::<IgnoredAny>()?;
									Event::HeartbeatAck
								}
								(0, Some(t)) => match t {
									"READY" => Event::Ready(map.next_value()?),
									"RESUMED" => {
										map.next_value::<IgnoredAny>()?;
										Event::Resumed
									}
									"GUILD_CREATE" => Event::GuildCreate(map.next_value()?),
									"GUILD_UPDATE" => Event::GuildUpdate(map.next_value()?),
									"GUILD_DELETE" => Event::GuildDelete(map.next_value()?),
									"MESSAGE_CREATE" => Event::MessageCreate(map.next_value()?),
									"MESSAGE_UPDATE" => Event::MessageUpdate(map.next_value()?),
									"MESSAGE_DELETE" => Event::MessageDelete(map.next_value()?),
									/*"MESSAGE_DELETE_BULK" => {
										Event::MessageDeleteBulk(map.next_value()?)
									}*/
									"GUILD_MEMBER_ADD" => Event::GuildMemberAdd(map.next_value()?),
									"GUILD_MEMBER_UPDATE" => {
										Event::GuildMemberUpdate(map.next_value()?)
									}
									"GUILD_MEMBER_REMOVE" => {
										Event::GuildMemberRemove(map.next_value()?)
									}
									"GUILD_MEMBERS_CHUNK" => {
										Event::GuildMembersChunk(map.next_value()?)
									}
									"GUILD_ROLE_CREATE" => {
										Event::GuildRoleCreate(map.next_value()?)
									}
									"GUILD_ROLE_UPDATE" => {
										Event::GuildRoleUpdate(map.next_value()?)
									}
									"GUILD_ROLE_DELETE" => {
										Event::GuildRoleDelete(map.next_value()?)
									}
									"CHANNEL_CREATE" => Event::ChannelCreate(map.next_value()?),
									"CHANNEL_UPDATE" => Event::ChannelUpdate(map.next_value()?),
									"CHANNEL_DELETE" => Event::ChannelDelete(map.next_value()?),
									/*"MESSAGE_REACTION_ADD" => {
										Event::MessageReactionAdd(map.next_value()?)
									}
									"MESSAGE_REACTION_REMOVE" => {
										Event::MessageReactionRemove(map.next_value()?)
									}
									"MESSAGE_REACTION_REMOVE_ALL" => {
										Event::MessageReactionRemoveAll(map.next_value()?)
									}
									"MESSAGE_REACTION_REMOVE_EMOJI" => {
										Event::MessageReactionRemoveEmoji(map.next_value()?)
									}*/
									"APPLICATION_COMMAND_CREATE" => {
										Event::ApplicationCommandCreate(map.next_value()?)
									}
									"APPLICATION_COMMAND_UPDATE" => {
										Event::ApplicationCommandUpdate(map.next_value()?)
									}
									"APPLICATION_COMMAND_DELETE" => {
										Event::ApplicationCommandDelete(map.next_value()?)
									}
									"INTERACTION_CREATE" => {
										Event::InteractionCreate(map.next_value()?)
									}
									"VOICE_STATE_UPDATE" => {
										Event::VoiceStateUpdate(map.next_value()?)
									}
									"VOICE_SERVER_UPDATE" => {
										Event::VoiceServerUpdate(map.next_value()?)
									}
									t => {
										map.next_value::<IgnoredAny>()?;
										Event::Unknown(t.into())
									}
								},
								(0, None) => return Err(de::Error::missing_field("t")),
								(e, _) => {
									map.next_value::<IgnoredAny>()?;
									Event::Unknown(format!("{}", e))
								}
							};
							payload = Some(Payload { sequence, event });
						}
						_ => {
							map.next_value::<IgnoredAny>()?;
						}
					}
				}
				payload.ok_or_else(|| de::Error::missing_field("d"))
			}
		}

		deserializer.deserialize_struct("Payload", &["t", "s", "op", "d"], PayloadVisitor)
	}
}

#[derive(Clone, Debug, Deserialize)]
pub enum Event {
	Hello(Hello),
	Ready(Ready),
	Resumed,
	InvalidSession(InvalidSession),
	HeartbeatAck,
	GuildCreate(GuildCreate),
	GuildUpdate(GuildUpdate),
	GuildDelete(GuildDelete),
	MessageCreate(MessageCreate),
	MessageUpdate(MessageUpdate),
	MessageDelete(MessageDelete),
	// MessageDeleteBulk(MessageDeleteBulk),
	GuildMemberAdd(GuildMemberAdd),
	GuildMemberUpdate(GuildMemberUpdate),
	GuildMemberRemove(GuildMemberRemove),
	GuildMembersChunk(GuildMembersChunk),
	GuildRoleCreate(GuildRoleCreate),
	GuildRoleUpdate(GuildRoleUpdate),
	GuildRoleDelete(GuildRoleDelete),
	ChannelCreate(ChannelCreate),
	ChannelUpdate(ChannelUpdate),
	ChannelDelete(ChannelDelete),
	// MessageReactionAdd(MessageReactionAdd),
	// MessageReactionRemove(MessageReactionRemove),
	// MessageReactionRemoveAll(MessageReactionRemoveAll),
	// MessageReactionRemoveEmoji(MessageReactionRemoveEmoji),
	ApplicationCommandCreate(ApplicationCommandCreate),
	ApplicationCommandUpdate(ApplicationCommandUpdate),
	ApplicationCommandDelete(ApplicationCommandDelete),
	InteractionCreate(InteractionCreate),
	VoiceStateUpdate(VoiceStateUpdate),
	VoiceServerUpdate(VoiceServerUpdate),
	Unknown(String),
}

impl Event {
	pub fn guild_id(&self) -> Option<GuildId> {
		match self {
			Event::GuildCreate(e) => Some(e.guild.id),
			Event::GuildUpdate(e) => Some(e.guild.id),
			Event::GuildDelete(e) => Some(e.id),
			Event::MessageCreate(e) => e.message.guild_id,
			Event::MessageUpdate(e) => e.guild_id,
			Event::MessageDelete(e) => e.guild_id,
			// Event::MessageDeleteBulk(e) => e.guild_id,
			Event::GuildMemberAdd(e) => Some(e.guild_id),
			Event::GuildMemberUpdate(e) => Some(e.guild_id),
			Event::GuildMemberRemove(e) => Some(e.guild_id),
			Event::GuildMembersChunk(e) => Some(e.guild_id),
			Event::GuildRoleCreate(e) => Some(e.guild_id),
			Event::GuildRoleUpdate(e) => Some(e.guild_id),
			Event::GuildRoleDelete(e) => Some(e.guild_id),
			Event::ChannelCreate(e) => e.channel.guild_id,
			Event::ChannelUpdate(e) => e.channel.guild_id,
			Event::ChannelDelete(e) => e.channel.guild_id,
			// Event::MessageReactionAdd(e) => e.guild_id,
			// Event::MessageReactionRemove(e) => e.guild_id,
			// Event::MessageReactionRemoveAll(e) => e.guild_id,
			// Event::MessageReactionRemoveEmoji(e) => e.guild_id,
			Event::ApplicationCommandCreate(e) => e.guild_id,
			Event::ApplicationCommandUpdate(e) => e.guild_id,
			Event::ApplicationCommandDelete(e) => e.guild_id,
			Event::InteractionCreate(e) => e.interaction.guild_id,
			Event::VoiceStateUpdate(e) => e.voice_state.guild_id,
			Event::VoiceServerUpdate(e) => Some(e.guild_id),
			_ => None,
		}
	}

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

	pub fn is_heartbeat_ack(&self) -> bool {
		match self {
			Event::HeartbeatAck => true,
			_ => false,
		}
	}
}

impl fmt::Display for Event {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Event::Hello(_) => write!(f, "Hello"),
			Event::Ready(e) => write!(f, "Ready(username={})", e.user.username),
			Event::Resumed => write!(f, "Resumed"),
			Event::InvalidSession(_) => write!(f, "InvalidSession"),
			Event::HeartbeatAck => write!(f, "HeartbeatAck"),
			Event::GuildCreate(e) => {
				write!(f, "GuildCreate(id={}, name={})", e.guild.id, e.guild.name)
			}
			Event::GuildUpdate(e) => write!(f, "GuildUpdate(id={})", e.guild.id),
			Event::GuildDelete(e) => write!(f, "GuildDelete(id={})", e.id),
			Event::MessageCreate(e) => {
				let author = match &e.message.author {
					Some(u) => format!("{}", u),
					None => String::from("?"),
				};

				write!(
					f,
					"MessageCreate(user={}, channel_id={})",
					author, e.message.channel_id
				)
			}
			Event::MessageUpdate(e) => {
				write!(f, "MessageUpdate(channel_id={})", e.channel_id)
			}
			Event::MessageDelete(e) => {
				write!(f, "MessageDelete(channel_id={}, id={})", e.channel_id, e.id)
			}
			/*Event::MessageDeleteBulk(e) => write!(
				f,
				"MessageDeleteBulk(channel_id={}, count={})",
				e.channel_id,
				e.ids.len()
			),*/
			Event::GuildMemberAdd(e) => {
				write!(f, "GuildMemberAdd(guild={}, user={})", e.guild_id, e.member)
			}
			Event::GuildMemberUpdate(e) => write!(
				f,
				"GuildMemberUpdate(guild={}, user={})",
				e.guild_id, e.user
			),
			Event::GuildMemberRemove(e) => write!(
				f,
				"GuildMemberRemove(guild={}, user={})",
				e.guild_id, e.user
			),
			Event::GuildMembersChunk(e) => write!(
				f,
				"GuildMembersChunk(guild={}, chunk={}/{})",
				e.guild_id, e.chunk_index, e.chunk_count
			),
			Event::GuildRoleCreate(e) => write!(
				f,
				"GuildRoleCreate(guild={}, role={})",
				e.guild_id, e.role.id
			),
			Event::GuildRoleUpdate(e) => write!(
				f,
				"GuildRoleUpdate(guild={}, role={})",
				e.guild_id, e.role.id
			),
			Event::GuildRoleDelete(e) => write!(
				f,
				"GuildRoleDelete(guild={}, role={})",
				e.guild_id, e.role_id
			),
			Event::ChannelCreate(e) => write!(f, "ChannelCreate(id={})", e.channel.id),
			Event::ChannelUpdate(e) => write!(f, "ChannelUpdate(id={})", e.channel.id),
			Event::ChannelDelete(e) => write!(f, "ChannelDelete(id={})", e.channel.id),
			/*Event::MessageReactionAdd(e) => write!(
				f,
				"MessageReactionAdd(message={}, emoji={})",
				e.message_id, e.emoji
			),
			Event::MessageReactionRemove(e) => write!(
				f,
				"MessageReactionRemove(message={}, emoji={})",
				e.message_id, e.emoji
			),
			Event::MessageReactionRemoveAll(e) => {
				write!(f, "MessageReactionRemoveAll(message={})", e.message_id)
			}
			Event::MessageReactionRemoveEmoji(e) => write!(
				f,
				"MessageReactionRemoveEmoji(message={}, emoji={})",
				e.message_id, e.emoji
			),*/
			Event::ApplicationCommandCreate(e) => {
				write!(f, "ApplicationCommandCreate(command={})", e.command.name)
			}
			Event::ApplicationCommandUpdate(e) => {
				write!(f, "ApplicationCommandUpdate(command={})", e.command.name)
			}
			Event::ApplicationCommandDelete(e) => {
				write!(f, "ApplicationCommandDelete(command={})", e.command.name)
			}
			Event::InteractionCreate(e) => {
				let user = match &e.interaction.user {
					Some(u) => format!("{}", u),
					None => String::from("?"),
				};
				write!(
					f,
					"InteractionCreate(command={}, user={})",
					e.interaction.data.name, user
				)
			}
			Event::VoiceStateUpdate(e) => {
				write!(f, "VoiceStateUpdate")?;
				if let Some(id) = e.voice_state.guild_id {
					write!(f, "(guild={})", id)?;
				}
				Ok(())
			}
			Event::VoiceServerUpdate(e) => write!(f, "VoiceServerUpdate(guild={})", e.guild_id),
			Event::Unknown(n) => write!(f, "Unknown({})", n),
		}
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct Hello {
	pub heartbeat_interval: u64,
}
//
#[derive(Clone, Debug, Deserialize)]
pub struct Ready {
	pub v: u8,
	pub user: User,
	// private_channels
	#[serde(deserialize_with = "guild_list")]
	pub guilds: HashSet<GuildId>,
	pub session_id: String,
	pub shard: Option<(u8, u8)>,
	pub application: Application,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct InvalidSession {
	pub resumable: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct ChannelCreate {
	pub channel: Channel,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct ChannelUpdate {
	pub channel: Channel,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct ChannelDelete {
	pub channel: Channel,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct GuildCreate {
	pub guild: Guild,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct GuildUpdate {
	pub guild: Guild,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildDelete {
	pub id: GuildId,
	#[serde(default)]
	pub unavailable: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct MessageCreate {
	pub message: Message,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MessageUpdate {
	pub id: MessageId,
	pub channel_id: ChannelId,
	#[serde(default)]
	pub guild_id: Option<GuildId>,
	#[serde(default)]
	pub content: Option<String>,
	#[serde(default)]
	pub edited_timestamp: Option<DateTime>,
	#[serde(default)]
	pub mention_everyone: bool,
	#[serde(default)]
	pub mentions: Vec<User>,
	#[serde(default)]
	pub pinned: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MessageDelete {
	pub id: MessageId,
	pub channel_id: ChannelId,
	#[serde(default)]
	pub guild_id: Option<GuildId>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildMemberAdd {
	pub guild_id: GuildId,
	#[serde(flatten)]
	pub member: Member,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildMemberUpdate {
	pub guild_id: GuildId,
	pub roles: HashSet<RoleId>,
	pub user: User,
	#[serde(default)]
	pub nick: Option<String>,
	#[serde(default)]
	pub premium_since: Option<DateTime>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildMemberRemove {
	pub guild_id: GuildId,
	pub user: User,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildMembersChunk {
	pub guild_id: GuildId,
	pub members: Vec<Member>,
	pub chunk_index: u16,
	pub chunk_count: u16,
	// not_found
	// presences
	#[serde(default)]
	pub nonce: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildRoleCreate {
	pub guild_id: GuildId,
	pub role: Role,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildRoleUpdate {
	pub guild_id: GuildId,
	pub role: Role,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildRoleDelete {
	pub guild_id: GuildId,
	pub role_id: RoleId,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct InteractionCreate {
	pub interaction: Interaction,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationCommandCreate {
	#[serde(flatten)]
	pub command: ApplicationCommand,
	#[serde(default)]
	pub guild_id: Option<GuildId>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationCommandUpdate {
	#[serde(flatten)]
	pub command: ApplicationCommand,
	#[serde(default)]
	pub guild_id: Option<GuildId>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationCommandDelete {
	#[serde(flatten)]
	pub command: ApplicationCommand,
	#[serde(default)]
	pub guild_id: Option<GuildId>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct VoiceStateUpdate {
	pub voice_state: VoiceState,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VoiceServerUpdate {
	pub token: String,
	pub guild_id: GuildId,
	#[serde(default)]
	pub endpoint: Option<String>,
}

fn guild_list<'de, D>(d: D) -> Result<HashSet<GuildId>, D::Error>
where
	D: Deserializer<'de>,
{
	#[derive(Deserialize)]
	struct Guild {
		id: GuildId,
	}

	struct GuildsVisitor;

	impl<'de> Visitor<'de> for GuildsVisitor {
		type Value = HashSet<GuildId>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("list of guilds")
		}

		fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
		where
			V: SeqAccess<'de>,
		{
			let mut list = HashSet::new();
			while let Some(guild) = seq.next_element::<Guild>()? {
				list.insert(guild.id);
			}
			Ok(list)
		}
	}

	d.deserialize_seq(GuildsVisitor)
}
