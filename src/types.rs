use chrono::{TimeZone, Utc};
use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::collections::HashSet;
use std::fmt;
use std::num::ParseIntError;
use std::ops::Deref;
use std::str::FromStr;

const DISCORD_EPOCH: u64 = 1_420_070_400_000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Snowflake(u64);

impl Snowflake {
	pub fn date_time(&self) -> DateTime {
		let timestamp = (self.0 >> 22) + DISCORD_EPOCH;
		Utc.timestamp_millis(timestamp as i64).into()
	}

	pub fn worker(&self) -> u8 {
		((self.0 & 0x3E0000) >> 17) as u8
	}

	pub fn process(&self) -> u8 {
		((self.0 & 0x1F000) >> 12) as u8
	}

	pub fn increment(&self) -> u16 {
		(self.0 & 0xFFF) as u16
	}
}

impl fmt::Display for Snowflake {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(&self.0, f)
	}
}

impl From<u64> for Snowflake {
	fn from(snowflake: u64) -> Self {
		Self(snowflake)
	}
}

impl FromStr for Snowflake {
	type Err = ParseIntError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let id = s.parse::<u64>()?;
		Ok(id.into())
	}
}

impl PartialEq<u64> for Snowflake {
	fn eq(&self, other: &u64) -> bool {
		self.0 == *other
	}
}

impl From<Snowflake> for u64 {
	fn from(snowflake: Snowflake) -> Self {
		snowflake.0
	}
}

impl<'de> Deserialize<'de> for Snowflake {
	fn deserialize<D>(deserializer: D) -> Result<Snowflake, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct SnowflakeVisitor;

		impl<'de> Visitor<'de> for SnowflakeVisitor {
			type Value = Snowflake;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("u64 or string storing a snowflake id")
			}

			fn visit_u64<E>(self, val: u64) -> Result<Snowflake, E>
			where
				E: serde::de::Error,
			{
				Ok(Snowflake(val))
			}

			fn visit_str<E>(self, val: &str) -> Result<Snowflake, E>
			where
				E: serde::de::Error,
			{
				val.parse()
					.map(|v| Snowflake(v))
					.map_err(|_| E::invalid_value(Unexpected::Str(val), &self))
			}
		}

		deserializer.deserialize_any(SnowflakeVisitor)
	}
}

impl Serialize for Snowflake {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&format!("{}", self.0))
	}
}

#[cfg(feature = "sqlx")]
impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for Snowflake
where
	i64: sqlx::Decode<'r, DB>,
{
	fn decode(
		value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
	) -> Result<Snowflake, Box<dyn std::error::Error + 'static + Send + Sync>> {
		let id = i64::decode(value)? as u64;
		Ok(Snowflake::from(id))
	}
}

#[cfg(feature = "sqlx")]
impl<'q, DB: sqlx::Database> sqlx::Encode<'q, DB> for Snowflake
where
	i64: sqlx::Encode<'q, DB>,
{
	fn encode_by_ref(
		&self,
		buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
	) -> sqlx::encode::IsNull {
		let id = self.0 as i64;
		id.encode_by_ref(buf)
	}
}

#[cfg(feature = "sqlx")]
impl<DB: sqlx::Database> sqlx::Type<DB> for Snowflake
where
	i64: sqlx::Type<DB>,
{
	fn type_info() -> DB::TypeInfo {
		i64::type_info()
	}
}

#[derive(Clone, Debug, Eq, Ord)]
pub struct DateTime(pub chrono::DateTime<Utc>);

impl DateTime {
	pub fn format<'a>(&self, fmt: &'a str) -> impl fmt::Display + 'a {
		self.0.format(fmt)
	}
}

impl PartialEq for DateTime {
	fn eq(&self, other: &Self) -> bool {
		PartialEq::eq(&self.0, &other.0)
	}
}

impl PartialOrd for DateTime {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		PartialOrd::partial_cmp(&self.0, &other.0)
	}
}

impl From<chrono::DateTime<Utc>> for DateTime {
	fn from(datetime: chrono::DateTime<Utc>) -> Self {
		Self(datetime)
	}
}

impl<'de> Deserialize<'de> for DateTime {
	fn deserialize<D>(deserializer: D) -> Result<DateTime, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct DateTimeVisitor;

		impl<'de> Visitor<'de> for DateTimeVisitor {
			type Value = DateTime;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("iso8601 timestamp")
			}

			fn visit_str<E>(self, val: &str) -> Result<DateTime, E>
			where
				E: serde::de::Error,
			{
				val.parse()
					.map(|v| DateTime(v))
					.map_err(|_| E::invalid_value(Unexpected::Str(val), &self))
			}
		}

		deserializer.deserialize_any(DateTimeVisitor)
	}
}

// Separate type for each id so we can't confuse them
// Just a thin wrapper around a Snowflake with appropriate trait impls
macro_rules! id_type {
	($t:ident) => {
		#[derive(
			Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
		)]
		pub struct $t(Snowflake);

		impl fmt::Display for $t {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				fmt::Display::fmt(&self.0, f)
			}
		}

		impl From<Snowflake> for $t {
			fn from(id: Snowflake) -> Self {
				$t(id)
			}
		}

		impl From<&$t> for $t {
			fn from(id: &$t) -> Self {
				*id
			}
		}

		impl From<&Snowflake> for $t {
			fn from(id: &Snowflake) -> Self {
				$t(*id)
			}
		}

		impl From<u64> for $t {
			fn from(id: u64) -> Self {
				$t(Snowflake::from(id))
			}
		}

		impl FromStr for $t {
			type Err = ParseIntError;
			fn from_str(s: &str) -> Result<Self, Self::Err> {
				let id = s.parse::<u64>()?;
				Ok(id.into())
			}
		}

		impl PartialEq<u64> for $t {
			fn eq(&self, other: &u64) -> bool {
				PartialEq::eq(&self.0, other)
			}
		}

		impl From<$t> for u64 {
			fn from(id: $t) -> Self {
				id.0 .0
			}
		}

		impl Deref for $t {
			type Target = Snowflake;

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}

		#[cfg(feature = "sqlx")]
		impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for $t
		where
			i64: sqlx::Decode<'r, DB>,
		{
			fn decode(
				value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
			) -> Result<$t, Box<dyn std::error::Error + 'static + Send + Sync>> {
				Ok($t::from(Snowflake::decode(value)?))
			}
		}

		#[cfg(feature = "sqlx")]
		impl<'q, DB: sqlx::Database> sqlx::Encode<'q, DB> for $t
		where
			i64: sqlx::Encode<'q, DB>,
		{
			fn encode_by_ref(
				&self,
				buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
			) -> sqlx::encode::IsNull {
				self.0.encode_by_ref(buf)
			}
		}

		#[cfg(feature = "sqlx")]
		impl<DB: sqlx::Database> sqlx::Type<DB> for $t
		where
			i64: sqlx::Type<DB>,
		{
			fn type_info() -> DB::TypeInfo {
				i64::type_info()
			}
		}
	};
}

macro_rules! id_types {
	($($t:ident),+) => { $(id_type!($t);)+ }
}

id_types!(ApplicationId, ChannelId, GuildId, MessageId, RoleId, UserId);

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Application {
	pub id: ApplicationId,
	pub flags: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Channel {
	pub id: ChannelId,
	#[serde(rename = "type")]
	pub channel_type: ChannelType,
	#[serde(default)]
	pub guild_id: Option<GuildId>,
	#[serde(default)]
	pub position: Option<u16>,
	// permission_overwrites
	#[serde(default)]
	pub name: Option<String>,
	#[serde(default)]
	pub topic: Option<String>,
	#[serde(default)]
	pub nsfw: Option<bool>,
	// last_message_id
	#[serde(default)]
	pub bitrate: Option<u32>,
	#[serde(default)]
	pub user_limit: Option<u16>,
	#[serde(default)]
	pub rate_limit_per_user: Option<u16>,
	// recipients
	// #[serde(default)]
	// pub icon: Option<String>,
	// #[serde(default)]
	// pub owner_id: Option<Snowflake>,
	// #[serde(default)]
	// pub application_id: Option<Snowflake>,
	#[serde(default)]
	pub parent_id: Option<ChannelId>,
	// last_pin_timestamp
}

impl fmt::Display for Channel {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.name {
			Some(n) => fmt::Display::fmt(n, f),
			None => write!(f, "??"),
		}
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct Guild {
	pub id: GuildId,
	pub name: String,
	pub icon: Option<String>,
	// icon_hash
	pub splash: Option<String>,
	pub discovery_splash: Option<String>,
	#[serde(default)]
	pub owner: Option<bool>,
	pub owner_id: UserId,
	#[serde(default)]
	pub permissions: Option<u64>,
	pub region: String,
	pub afk_channel_id: Option<ChannelId>,
	pub afk_timeout: u64,
	#[serde(default)]
	pub widget_enabled: Option<bool>,
	#[serde(default)]
	pub widget_channel_id: Option<ChannelId>,
	pub verification_level: u8,
	pub default_message_notifications: u8,
	pub explicit_content_filter: u8,
	pub roles: Vec<Role>,
	// pub emojis: Vec<Emoji>,
	// pub features: Vec<String>,
	pub mfa_level: u8,
	pub application_id: Option<ApplicationId>,
	pub system_channel_id: Option<ChannelId>,
	pub system_channel_flags: u8,
	pub rules_channel_id: Option<ChannelId>,
	#[serde(default)]
	pub joined_at: Option<DateTime>,
	#[serde(default)]
	pub large: Option<bool>,
	#[serde(default)]
	pub unavailable: Option<bool>,
	#[serde(default)]
	pub member_count: Option<u16>,
	// #[serde(default)]
	// pub voice_states: Vec<VoiceState>,
	#[serde(default)]
	pub members: Vec<Member>,
	#[serde(default)]
	pub channels: Vec<Channel>,
	// presences
	// max_presences
	#[serde(default)]
	pub max_members: Option<u32>,
	pub vanity_url_code: Option<String>,
	pub description: Option<String>,
	pub banner: Option<String>,
	pub premium_tier: u8,
	#[serde(default)]
	pub premium_subscription_count: Option<u16>,
	pub preferred_locale: String,
	pub public_updates_channel_id: Option<ChannelId>,
	#[serde(default)]
	pub max_video_channel_users: Option<u16>,
	#[serde(default)]
	pub approximate_member_count: Option<u16>,
	#[serde(default)]
	pub approximate_presence_count: Option<u16>,
	// welcome_screen
}

#[derive(Clone, Debug, Deserialize)]
pub struct Role {
	pub id: RoleId,
	pub name: String,
	pub color: u32,
	pub hoist: bool,
	pub position: u16,
	pub permissions: String,
	pub managed: bool,
	pub mentionable: bool,
	// tags
}

#[derive(Clone, Debug, Deserialize)]
pub struct Member {
	#[serde(default)]
	pub user: Option<User>,
	#[serde(default)]
	pub nick: Option<String>,
	pub roles: HashSet<RoleId>,
	pub joined_at: DateTime,
	#[serde(default)]
	pub premium_since: Option<DateTime>,
	pub deaf: bool,
	pub mute: bool,
	#[serde(default)]
	pub pending: Option<bool>,
	#[serde(default)]
	pub permissions: Option<String>,
}

impl fmt::Display for Member {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.nick {
			Some(n) => fmt::Display::fmt(n, f)?,
			None => write!(f, "??")?,
		}
		if let Some(user) = &self.user {
			write!(f, " ({})", user)?;
		}
		Ok(())
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct User {
	pub id: UserId,
	pub username: String,
	pub discriminator: String,
	pub avatar: Option<String>,
	#[serde(default)]
	pub bot: Option<bool>,
	#[serde(default)]
	pub system: Option<bool>,
	#[serde(default)]
	pub mfa_enabled: Option<bool>,
	#[serde(default)]
	pub locale: Option<String>,
	#[serde(default)]
	pub verified: Option<bool>,
	#[serde(default)]
	pub email: Option<String>,
	#[serde(default)]
	pub flags: Option<UserFlags>,
	#[serde(default)]
	pub premium_type: Option<u64>,
	#[serde(default)]
	pub public_flags: Option<UserFlags>,
}

impl User {
	pub fn is_bot(&self) -> bool {
		self.bot.unwrap_or(false)
	}

	pub fn is_system(&self) -> bool {
		self.system.unwrap_or(false)
	}
}

impl fmt::Display for User {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}#{}", self.username, self.discriminator)
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct Message {
	pub id: MessageId,
	pub channel_id: ChannelId,
	#[serde(default)]
	pub guild_id: Option<GuildId>,
	#[serde(default)]
	pub author: Option<User>,
	#[serde(default)]
	pub member: Option<Member>,
	#[serde(default)]
	pub content: String,
	#[serde(default)]
	pub timestamp: Option<DateTime>,
	pub edited_timestamp: Option<DateTime>,
	#[serde(default)]
	pub tts: bool,
	#[serde(default)]
	pub mention_everyone: bool,
	#[serde(default)]
	pub mentions: Vec<User>,
	// mention_roles
	// #[serde(default)]
	// pub mention_channels: Vec<ChannelMention>,
	// attachments
	// embeds
	// reactions
	// nonce
	#[serde(default)]
	pub pinned: bool,
	#[serde(default)]
	pub webhook_id: Option<Snowflake>,
	#[serde(rename = "type")]
	pub message_type: MessageType,
	// activity
	// application
	// message_reference
	// flags
	// stickers
	// referenced_message
	// interaction
}

impl fmt::Display for Message {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(&self.content, f)
	}
}

impl From<&Message> for ChannelId {
	fn from(message: &Message) -> Self {
		message.channel_id
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Embed {
	// title
	// type
	// description
	#[serde(default)]
	pub url: Option<String>,
	// timestamp
	// color
	// footer
	#[serde(default)]
	pub image: Option<EmbedImage>,
	#[serde(default)]
	pub thumbnail: Option<EmbedThumbnail>,
	// video
	// provider
	// author
	// fields
}

impl Embed {
	pub fn new() -> Self {
		Self {
			url: None,
			image: None,
			thumbnail: None,
		}
	}

	pub fn url(mut self, url: String) -> Self {
		self.url = Some(url);
		self
	}

	pub fn image(mut self, url: String) -> Self {
		self.image = Some(EmbedImage {
			url: Some(url),
			height: None,
			width: None,
		});
		self
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmbedImage {
	#[serde(default)]
	pub url: Option<String>,
	// #[serde(default)]
	// pub proxy_url: Option<String>,
	#[serde(default)]
	pub height: Option<u16>,
	#[serde(default)]
	pub width: Option<u16>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmbedThumbnail {
	#[serde(default)]
	pub url: Option<String>,
	// #[serde(default)]
	// pub proxy_url: Option<String>,
	// #[serde(default)]
	// pub height: Option<u32>,
	// #[serde(default)]
	// pub width: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationCommand {
	pub id: Snowflake,
	pub application_id: ApplicationId,
	pub name: String,
	pub description: String,
	#[serde(default)]
	pub options: Vec<ApplicationCommandOption>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplicationCommandOption {
	#[serde(rename = "type")]
	pub option_type: ApplicationCommandOptionType,
	pub name: String,
	pub description: String,
	#[serde(default)]
	pub required: bool,
	#[serde(default)]
	pub choices: Vec<ApplicationCommandOptionChoice>,
	#[serde(default)]
	pub options: Vec<ApplicationCommandOption>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplicationCommandOptionChoice {
	pub name: String,
	pub value: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Interaction {
	pub id: Snowflake,
	#[serde(rename = "type")]
	pub interaction_type: InteractionType,
	pub data: InteractionData,
	#[serde(default)]
	pub guild_id: Option<GuildId>,
	#[serde(default)]
	pub channel_id: Option<ChannelId>,
	#[serde(default)]
	pub member: Option<Member>,
	pub user: Option<User>,
	pub token: String,
	pub version: u8,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InteractionData {
	pub id: Snowflake,
	pub name: String,
	#[serde(default)]
	pub options: Vec<InteractionDataOption>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InteractionDataOption {
	pub name: String,
	#[serde(default)]
	pub value: Option<String>,
	#[serde(default)]
	pub options: Vec<InteractionDataOption>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VoiceState {
	#[serde(default)]
	pub guild_id: Option<GuildId>,
	#[serde(default)]
	pub channel_id: Option<ChannelId>,
	pub user_id: UserId,
	#[serde(default)]
	pub member: Option<Member>,
	pub session_id: String,
	pub deaf: bool,
	pub mute: bool,
	pub self_deaf: bool,
	pub self_mute: bool,
	#[serde(default)]
	pub self_stream: bool,
	pub self_video: bool,
	pub suppress: bool,
	#[serde(default)]
	pub request_to_speak_timestamp: Option<DateTime>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AllowedMentions {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub parse: Option<Vec<String>>,
}

impl AllowedMentions {
	pub fn none() -> Self {
		Self {
			parse: Some(Vec::new()),
		}
	}
}

bitflags::bitflags! {
	#[derive(Deserialize, Serialize)]
	#[serde(transparent)]
	pub struct Intents: u16 {
		const GUILDS = 1 << 0;
		const GUILD_MEMBERS = 1 << 1;
		const GUILD_BANS = 1 << 2;
		const GUILD_EMOJIS = 1 << 3;
		const GUILD_INTEGRATIONS = 1 << 4;
		const GUILD_WEBHOOKS = 1 << 5;
		const GUILD_INVITES = 1 << 6;
		const GUILD_VOICE_STATES = 1 << 7;
		const GUILD_PRESENCES = 1 << 8;
		const GUILD_MESSAGES = 1 << 9;
		const GUILD_MESSAGE_REACTIONS = 1 << 10;
		const GUILD_MESSAGE_TYPING = 1 << 11;
		const DIRECT_MESSAGES = 1 << 12;
		const DIRECT_MESSAGE_REACTIONS = 1 << 13;
		const DIRECT_MESSAGE_TYPING = 1 << 14;

		const GUILD_ALL = Self::GUILDS.bits | Self::GUILD_MEMBERS.bits | Self::GUILD_BANS.bits |
			Self::GUILD_EMOJIS.bits | Self::GUILD_INTEGRATIONS.bits | Self::GUILD_WEBHOOKS.bits |
			Self::GUILD_INVITES.bits | Self::GUILD_VOICE_STATES.bits | Self::GUILD_PRESENCES.bits |
			Self::GUILD_MESSAGES.bits | Self::GUILD_MESSAGE_REACTIONS.bits |
			Self::GUILD_MESSAGE_TYPING.bits;
		const DIRECT_MESSAGE_ALL = Self::DIRECT_MESSAGES.bits | Self::DIRECT_MESSAGE_REACTIONS.bits |
			Self::DIRECT_MESSAGE_TYPING.bits;
		const ALL = Self::GUILD_ALL.bits | Self::DIRECT_MESSAGE_ALL.bits;
	}
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
	Online,
	#[serde(rename = "dnd")]
	DoNotDisturb,
	Idle,
	Invisible,
	Offline,
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, PartialEq, Serialize_repr)]
#[repr(u8)]
pub enum ChannelType {
	GuildText = 0,
	DirectMessage = 1,
	GuildVoice = 2,
	GroupDirectMessage = 3,
	GuildCategory = 4,
	GuildNews = 5,
	GuildStore = 6,
	GuildNewsThread = 10,
	GuildPublicThread = 11,
	GuildPrivateThread = 12,
	GuildStageVoice = 13,
}

bitflags::bitflags! {
	#[derive(Deserialize, Serialize)]
	#[serde(transparent)]
	pub struct UserFlags: u32 {
		const DISCORD_EMPLOYEE = 1 << 0;
		const DISCORD_PARTNER = 1 << 1;
		const HYPESQUAD_EVENTS = 1 << 2;
		const BUG_HUNTER_LEVEL_1 = 1 << 3;
		const HOUSE_BRAVERY = 1 << 6;
		const HOUSE_BRILLIANCE = 1 << 7;
		const HOUSE_BALANCE = 1 << 8;
		const EARLY_SUPPORTER = 1 << 9;
		const TEAM_USER = 1 << 10;
		const SYSTEM = 1 << 12;
		const BUG_HUNTER_LEVEL_2 = 1 << 14;
		const VERIFIED_BOT = 1 << 16;
		const VERIFIED_BOT_DEVELOPER = 1 << 17;
	}
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, PartialEq, Serialize_repr)]
#[repr(u8)]
pub enum MessageType {
	Default = 0,
	RecipientAdd = 1,
	RecipientRemove = 2,
	Call = 3,
	ChannelNameChange = 4,
	ChannelIconChange = 5,
	ChannelPinnedMessage = 6,
	GuildMemberJoin = 7,
	UserPremiumGuildSubscription = 8,
	UserPremiumGuildSubscriptionTier1 = 9,
	UserPremiumGuildSubscriptionTier2 = 10,
	UserPremiumGuildSubscriptionTier3 = 11,
	ChannelFollowAdd = 12,
	GuildDiscoveryDisqualified = 14,
	GuildDiscoveryRequalified = 15,
	Reply = 19,
	ApplicationCommand = 20,
}

impl MessageType {
	pub fn is_textual(&self) -> bool {
		match self {
			Self::Default | Self::Reply | Self::ApplicationCommand => true,
			_ => false,
		}
	}
}

bitflags::bitflags! {
	#[derive(Deserialize)]
	#[serde(transparent)]
	pub struct MessageFlags: u32 {
		const CROSSPOSTED = 1 << 0;
		const IS_CROSSPOST = 1 << 1;
		const SUPPRESS_EMBEDS = 1 << 2;
		const SOURCE_MESSAGE_DELETED = 1 << 3;
		const URGENT = 1 << 4;
	}
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, PartialEq, Serialize_repr)]
#[repr(u8)]
pub enum ApplicationCommandOptionType {
	SubCommand = 1,
	SubCommandGroup = 2,
	String = 3,
	Integer = 4,
	Boolean = 5,
	User = 6,
	Channel = 7,
	Role = 8,
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, PartialEq, Serialize_repr)]
#[repr(u8)]
pub enum InteractionType {
	Ping = 1,
	Command = 2,
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, PartialEq, Serialize_repr)]
#[repr(u8)]
pub enum InteractionResponseType {
	Pong = 1,
	ChannelMessage = 4,
	DeferredChannelMessage = 5,
}

bitflags::bitflags! {
	#[derive(Deserialize, Serialize)]
	#[serde(transparent)]
	pub struct SpeakingFlags: u8 {
		const MICROPHONE = 1 << 0;
		const SOUNDSHARE = 1 << 1;
		const PRIORITY = 1 << 2;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn snowflake() {
		let id = Snowflake(175_928_847_299_117_063);
		let time = Utc.ymd(2016, 04, 30).and_hms_milli(11, 18, 25, 796);
		assert_eq!(id.date_time(), time);
		assert_eq!(id.worker(), 1);
		assert_eq!(id.process(), 0);
		assert_eq!(id.increment(), 7);
	}
}
