use crate::bitflags::BitFlagsVisitor;
use crate::CowString;
use chrono::{Duration, TimeZone, Utc};
use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt;
use std::num::ParseIntError;
use std::ops::{Deref, Sub};
use std::str::FromStr;

const DISCORD_EPOCH: u64 = 1_420_070_400_000;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Snowflake(u64);

impl Snowflake {
	pub fn date_time(&self) -> DateTime {
		let timestamp = (self.0 >> 22) + DISCORD_EPOCH;
		Utc.timestamp_millis_opt(timestamp as i64).unwrap().into()
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

	fn compatible(ty: &DB::TypeInfo) -> bool {
		i64::compatible(ty)
	}
}

#[derive(Clone, Debug, Eq, Ord)]
pub struct DateTime(chrono::DateTime<Utc>);

impl DateTime {
	pub fn now() -> Self {
		Self(Utc::now())
	}

	pub fn time_passed(self) -> Duration {
		Self::now() - self
	}

	pub fn into_inner(self) -> chrono::DateTime<Utc> {
		self.0
	}
}

impl Deref for DateTime {
	type Target = chrono::DateTime<Utc>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl PartialEq for DateTime {
	fn eq(&self, other: &Self) -> bool {
		PartialEq::eq(&self.0, &other.0)
	}
}

impl Sub<DateTime> for DateTime {
	type Output = Duration;

	fn sub(self, rhs: DateTime) -> Duration {
		self.0 - rhs.0
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

#[cfg(feature = "sqlx")]
impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for DateTime
where
	i64: sqlx::Decode<'r, DB>,
{
	fn decode(
		value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
	) -> Result<DateTime, Box<dyn std::error::Error + 'static + Send + Sync>> {
		match Utc.timestamp_opt(i64::decode(value)?, 0) {
			LocalResult::Single(dt) => Ok(DateTime(dt)),
			_ => Err(format!("Invalid date").into()),
		}
	}
}

#[cfg(feature = "sqlx")]
impl<'q, DB: sqlx::Database> sqlx::Encode<'q, DB> for DateTime
where
	i64: sqlx::Encode<'q, DB>,
{
	fn encode_by_ref(
		&self,
		buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
	) -> sqlx::encode::IsNull {
		self.0.timestamp().encode_by_ref(buf)
	}
}

#[cfg(feature = "sqlx")]
impl<DB: sqlx::Database> sqlx::Type<DB> for DateTime
where
	i64: sqlx::Type<DB>,
{
	fn type_info() -> DB::TypeInfo {
		i64::type_info()
	}

	fn compatible(ty: &DB::TypeInfo) -> bool {
		i64::compatible(ty)
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

			fn compatible(ty: &DB::TypeInfo) -> bool {
				i64::compatible(ty)
			}
		}
	};
}

macro_rules! id_types {
	($($t:ident),+) => { $(id_type!($t);)+ }
}

id_types!(
	ApplicationId,
	ChannelId,
	GuildId,
	InteractionId,
	MessageId,
	RoleId,
	UserId
);

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
	pub color: Color,
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
	#[serde(default)]
	pub reactions: Vec<Reaction>,
	// nonce
	#[serde(default)]
	pub pinned: bool,
	#[serde(default)]
	pub webhook_id: Option<Snowflake>,
	#[serde(rename = "type")]
	pub message_type: MessageType,
	#[serde(default)]
	pub interaction: Option<MessageInteraction>,
	#[serde(default)]
	pub components: Vec<Component>,
	// activity
	// application
	// message_reference
	// flags
	// referenced_message
	// thread
	// sticker_items
	// stickers
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

#[derive(Clone, Debug, Deserialize)]
pub struct MessageInteraction {
	pub id: InteractionId,
	#[serde(rename = "type")]
	pub interaction_type: InteractionType,
	pub name: String,
	pub user: User,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Embed {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub title: Option<CowString>,
	// type
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub description: Option<CowString>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub url: Option<CowString>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub timestamp: Option<chrono::DateTime<Utc>>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub color: Option<Color>,
	// footer
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub image: Option<EmbedImage>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub thumbnail: Option<EmbedThumbnail>,
	// video
	// provider
	// author
	// fields
}

impl Embed {
	pub fn new() -> Self {
		Self {
			title: None,
			description: None,
			url: None,
			timestamp: None,
			color: None,
			image: None,
			thumbnail: None,
		}
	}

	pub fn title<T: Into<CowString>>(mut self, title: T) -> Self {
		self.title = Some(title.into());
		self
	}

	pub fn description<T: Into<CowString>>(mut self, description: T) -> Self {
		self.description = Some(description.into());
		self
	}

	pub fn url<T: Into<CowString>>(mut self, url: T) -> Self {
		self.url = Some(url.into());
		self
	}

	pub fn timestamp<T: Into<chrono::DateTime<Utc>>>(mut self, timestamp: T) -> Self {
		self.timestamp = Some(timestamp.into());
		self
	}

	pub fn color<T: Into<Color>>(mut self, color: T) -> Self {
		self.color = Some(color.into());
		self
	}

	pub fn image<T: Into<CowString>>(mut self, url: T) -> Self {
		self.image = Some(EmbedImage {
			url: Some(url.into()),
			height: None,
			width: None,
		});
		self
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmbedImage {
	#[serde(default)]
	pub url: Option<CowString>,
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
	pub url: Option<CowString>,
	// #[serde(default)]
	// pub proxy_url: Option<String>,
	// #[serde(default)]
	// pub height: Option<u32>,
	// #[serde(default)]
	// pub width: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Reaction {
	pub count: u64,
	pub me: bool,
	pub emoji: PartialEmoji,
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
	pub name: CowString,
	pub description: CowString,
	#[serde(default)]
	pub required: bool,
	#[serde(default)]
	pub choices: Vec<ApplicationCommandOptionChoice>,
	#[serde(default)]
	pub options: Vec<ApplicationCommandOption>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplicationCommandOptionChoice {
	pub name: CowString,
	pub value: CowString,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Interaction {
	pub id: InteractionId,
	pub application_id: Snowflake,
	#[serde(rename = "type")]
	pub interaction_type: InteractionType,
	pub data: InteractionData,
	#[serde(default)]
	pub guild_id: Option<GuildId>,
	#[serde(default)]
	pub channel_id: Option<ChannelId>,
	#[serde(default)]
	pub member: Option<Member>,
	#[serde(default)]
	pub user: Option<User>,
	pub token: String,
	pub version: u8,
	#[serde(default)]
	pub message: Option<Message>,
}

impl Interaction {
	pub fn is_command_interaction(&self) -> bool {
		self.interaction_type == InteractionType::Component
	}

	pub fn is_component_interaction(&self) -> bool {
		self.interaction_type == InteractionType::Component
	}
}

#[derive(Clone, Debug, Deserialize)]
pub struct InteractionData {
	#[serde(default)]
	pub id: Option<Snowflake>,
	#[serde(default)]
	pub name: Option<String>,
	#[serde(default)]
	pub options: Vec<InteractionDataOption>,
	#[serde(default)]
	pub values: Vec<String>,
	#[serde(default)]
	pub custom_id: Option<String>,
	#[serde(default)]
	pub component_type: Option<ComponentType>,
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
	pub parse: Option<Vec<CowString>>,
}

impl AllowedMentions {
	pub fn none() -> Self {
		Self {
			parse: Some(Vec::new()),
		}
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Component {
	#[serde(rename = "type")]
	pub component_type: ComponentType,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub style: Option<ButtonStyle>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub label: Option<CowString>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub emoji: Option<PartialEmoji>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub custom_id: Option<CowString>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub url: Option<CowString>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub disabled: Option<bool>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub default: Option<bool>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub components: Vec<Component>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub options: Vec<SelectOption>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub placeholder: Option<CowString>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectOption {
	pub label: CowString,
	pub value: CowString,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub description: Option<CowString>,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub emoji: Option<PartialEmoji>,
	#[serde(default)]
	pub default: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartialEmoji {
	#[serde(default)]
	pub id: Option<Snowflake>,
	#[serde(default)]
	pub name: Option<CowString>,
	#[serde(default)]
	pub animated: bool,
}

impl From<&emoji::Emoji> for PartialEmoji {
	fn from(emoji: &emoji::Emoji) -> Self {
		Self {
			id: None,
			name: Some(emoji.glyph.into()),
			animated: false,
		}
	}
}

impl From<Snowflake> for PartialEmoji {
	fn from(id: Snowflake) -> Self {
		Self {
			id: Some(id),
			name: None,
			animated: false,
		}
	}
}

impl TryFrom<String> for PartialEmoji {
	type Error = ();

	fn try_from(value: String) -> Result<Self, Self::Error> {
		if let Ok(id) = value.parse::<Snowflake>() {
			Ok(id.into())
		} else if let Some(emoji) = emoji::lookup_by_glyph::lookup(&value) {
			Ok(emoji.into())
		} else {
			Err(())
		}
	}
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Color(u32);

macro_rules! named_colors {
	($($n:ident: $v:literal),+) => {
		$(pub const $n: Color = Color($v);)+
	};
}

impl Color {
	named_colors!(
		DEFAULT: 0x000000,
		WHITE: 0xffffff,
		AQUA: 0x1abc9c,
		GREEN: 0x57f287,
		BLUE: 0x3498db,
		YELLOW: 0xfee75c,
		PURPLE: 0x9b59b6,
		LUMINOUS_VIVID_PINK: 0xe91e63,
		FUCHSIA: 0xeb459e,
		GOLD: 0xf1c40f,
		ORANGE: 0xe67e22,
		RED: 0xed4245,
		GREY: 0x95a5a6,
		NAVY: 0x34495e,
		DARK_AQUA: 0x11806a,
		DARK_GREEN: 0x1f8b4c,
		DARK_BLUE: 0x206694,
		DARK_PURPLE: 0x71368a,
		DARK_VIVID_PINK: 0xad1457,
		DARK_GOLD: 0xc27c0e,
		DARK_ORANGE: 0xa84300,
		DARK_RED: 0x992d22,
		DARK_GREY: 0x979c9f,
		DARKER_GREY: 0x7f8c8d,
		LIGHT_GREY: 0xbcc0c0,
		DARK_NAVY: 0x2c3e50,
		BLURPLE: 0x5865f2,
		GREYPLE: 0x99aab5,
		DARK_BUT_NOT_BLACK: 0x2c2f33,
		NOT_QUITE_BLACK: 0x23272a
	);

	pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
		Color(((r as u32) << 16) + ((g as u32) << 8) + (b as u32))
	}

	pub fn rgb(self) -> [u8; 3] {
		let v = self.0;
		[
			(v >> 16 & 0xFF) as u8,
			(v >> 8 & 0xFF) as u8,
			(v & 0xFF) as u8,
		]
	}
}

impl Default for Color {
	fn default() -> Self {
		Color::DEFAULT
	}
}

impl fmt::Debug for Color {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let rgb = self.rgb();
		f.debug_struct("Color")
			.field("r", &rgb[0])
			.field("g", &rgb[1])
			.field("b", &rgb[2])
			.finish()
	}
}

impl fmt::Display for Color {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "#{:06x}", self.0)
	}
}

impl TryFrom<u32> for Color {
	type Error = ();
	fn try_from(val: u32) -> Result<Self, ()> {
		if val <= 0xFFFFFF {
			Ok(Color(val))
		} else {
			Err(())
		}
	}
}

impl TryFrom<u64> for Color {
	type Error = ();
	fn try_from(val: u64) -> Result<Self, ()> {
		if val <= u32::MAX as u64 {
			Self::try_from(val as u32)
		} else {
			Err(())
		}
	}
}

impl FromStr for Color {
	type Err = ();
	fn from_str(mut val: &str) -> Result<Self, Self::Err> {
		if val.starts_with("#") {
			val = &val[1..];
		} else if val.starts_with("0x") {
			val = &val[2..];
		}
		if val.len() != 6 {
			return Err(());
		}
		let val = u32::from_str_radix(val, 16).map_err(|_| ())?;
		Self::try_from(val)
	}
}

impl From<Color> for u32 {
	fn from(color: Color) -> u32 {
		color.0
	}
}

impl<'de> Deserialize<'de> for Color {
	fn deserialize<D>(deserializer: D) -> Result<Color, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct ColorVisitor;

		impl<'de> Visitor<'de> for ColorVisitor {
			type Value = Color;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("24-bit integer")
			}

			fn visit_u64<E>(self, val: u64) -> Result<Color, E>
			where
				E: serde::de::Error,
			{
				Color::try_from(val).map_err(|_| E::custom("value out of range"))
			}
		}

		deserializer.deserialize_u64(ColorVisitor)
	}
}

impl Serialize for Color {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_u32(self.0)
	}
}

bitflags::bitflags! {
	#[repr(transparent)]
	#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
	pub struct Intents: u32 {
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
		const MESSAGE_CONTENT = 1 << 15;
		const GUILD_SCHEDULED_EVENTS = 1 << 16;
		const AUTO_MODERATION_CONFIGURATION = 1 << 20;
		const AUTO_MODERATION_EXECUTION = 1 << 21;

		const GUILD_ALL = Self::GUILDS.bits() | Self::GUILD_MEMBERS.bits() | Self::GUILD_BANS.bits() |
			Self::GUILD_EMOJIS.bits() | Self::GUILD_INTEGRATIONS.bits() | Self::GUILD_WEBHOOKS.bits() |
			Self::GUILD_INVITES.bits() | Self::GUILD_VOICE_STATES.bits() | Self::GUILD_PRESENCES.bits() |
			Self::GUILD_MESSAGES.bits() | Self::GUILD_MESSAGE_REACTIONS.bits() |
			Self::GUILD_MESSAGE_TYPING.bits() | Self::GUILD_SCHEDULED_EVENTS.bits();
		const DIRECT_MESSAGE_ALL = Self::DIRECT_MESSAGES.bits() | Self::DIRECT_MESSAGE_REACTIONS.bits() |
			Self::DIRECT_MESSAGE_TYPING.bits();
		const ALL = Self::GUILD_ALL.bits() | Self::DIRECT_MESSAGE_ALL.bits() | Self::MESSAGE_CONTENT.bits() |
			Self::AUTO_MODERATION_CONFIGURATION.bits() | Self::AUTO_MODERATION_EXECUTION.bits();
	}
}

impl serde::Serialize for Intents {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_u32(self.bits())
	}
}

impl<'de> serde::Deserialize<'de> for Intents {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_u32(BitFlagsVisitor::new())
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
	GuildDirectory = 14,
	GuildForum = 15,
	#[serde(other)]
	Unknown = 255,
}

bitflags::bitflags! {
	#[repr(transparent)]
	#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
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
		const CERTIFIED_MODERATOR = 1 << 18;
		const BOT_HTTP_INTERACTIONS = 1 << 19;
		const ACTIVE_DEVELOPER = 1 << 22;
	}
}

impl serde::Serialize for UserFlags {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_u64(self.bits() as u64)
	}
}

impl<'de> serde::Deserialize<'de> for UserFlags {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_u64(BitFlagsVisitor::new())
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
	GuildDiscoveryGracePeriodInitialWarning = 16,
	GuildDiscoveryGracePeriodFinalWarning = 17,
	ThreadCreated = 18,
	Reply = 19,
	ApplicationCommand = 20,
	ThreadStarterMessage = 21,
	GuildInviteReminder = 22,
	ContextMenuCommand = 23,
	AutoModerationAction = 24,
	RoleSubscriptionPurchase = 25,
	InteractionPremiumUpsell = 26,
	StageStart = 27,
	StageEnd = 28,
	StageSpeaker = 29,
	StageTopic = 31,
	GuildApplicationPremiumSubscription = 32,
	#[serde(other)]
	Unknown = 255,
}

impl MessageType {
	pub fn is_textual(&self) -> bool {
		match self {
			Self::Default | Self::Reply | Self::ApplicationCommand | Self::ThreadStarterMessage => {
				true
			}
			_ => false,
		}
	}
}

bitflags::bitflags! {
	#[repr(transparent)]
	#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
	pub struct MessageFlags: u32 {
		const CROSSPOSTED = 1 << 0;
		const IS_CROSSPOST = 1 << 1;
		const SUPPRESS_EMBEDS = 1 << 2;
		const SOURCE_MESSAGE_DELETED = 1 << 3;
		const URGENT = 1 << 4;
	}
}

impl<'de> serde::Deserialize<'de> for MessageFlags {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_u32(BitFlagsVisitor::new())
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
	#[serde(other)]
	Unknown = 255,
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, PartialEq, Serialize_repr)]
#[repr(u8)]
pub enum InteractionType {
	Ping = 1,
	Command = 2,
	Component = 3,
	#[serde(other)]
	Unknown = 255,
}

impl InteractionType {
	pub fn is_command_interaction(&self) -> bool {
		self == &InteractionType::Command
	}

	pub fn is_component_interaction(&self) -> bool {
		self == &InteractionType::Component
	}
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, PartialEq, Serialize_repr)]
#[repr(u8)]
pub enum InteractionResponseType {
	Pong = 1,
	ChannelMessage = 4,
	DeferredChannelMessage = 5,
	DeferredUpdateMessage = 6,
	UpdateMessage = 7,
	#[serde(other)]
	Unknown = 255,
}

bitflags::bitflags! {
	#[repr(transparent)]
	#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
	pub struct SpeakingFlags: u32 {
		const MICROPHONE = 1 << 0;
		const SOUNDSHARE = 1 << 1;
		const PRIORITY = 1 << 2;
	}
}

impl serde::Serialize for SpeakingFlags {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_u32(self.bits())
	}
}

impl<'de> serde::Deserialize<'de> for SpeakingFlags {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_u32(BitFlagsVisitor::new())
	}
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, PartialEq, Serialize_repr)]
#[repr(u8)]
pub enum ComponentType {
	ActionRow = 1,
	Button = 2,
	StringSelect = 3,
	TextInput = 4,
	UserSelect = 5,
	RoleSelect = 6,
	MentionableSelect = 7,
	ChannelSelect = 8,
	#[serde(other)]
	Unknown = 255,
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, PartialEq, Serialize_repr)]
#[repr(u8)]
pub enum ButtonStyle {
	Primary = 1,
	Secondary = 2,
	Success = 3,
	Danger = 4,
	Link = 5,
	#[serde(other)]
	Unknown = 255,
}

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::NaiveDate;
	use serde_test::{assert_tokens, Configure, Token};

	#[test]
	fn snowflake() {
		let id = Snowflake(175_928_847_299_117_063);
		let time = NaiveDate::from_ymd_opt(2016, 4, 30)
			.unwrap()
			.and_hms_milli_opt(11, 18, 25, 796)
			.unwrap();
		let time: chrono::DateTime<Utc> = chrono::DateTime::from_utc(time, Utc);
		assert_eq!(id.date_time().into_inner(), time);
		assert_eq!(id.worker(), 1);
		assert_eq!(id.process(), 0);
		assert_eq!(id.increment(), 7);
	}

	#[test]
	fn bitflags() {
		assert_tokens(&(Intents::GUILD_ALL).readable(), &[Token::U32(69631)]);
	}

	#[test]
	fn user() {
		let json = r#"{"verified":true,"username":"[DEV] Galaxy of Dreams","mfa_enabled":true,"id":"292738137426362368","global_name":null,"flags":0,"email":null,"discriminator":"6948","bot":true,"avatar":null}"#;
		let user: User = serde_json::from_str(json).unwrap();
	}

	#[test]
	fn member() {
		let json = r#"{"user":{"username":"anon","public_flags":4195072,"id":"1","global_name":"anon","display_name":"anon","discriminator":"0","bot":false,"avatar_decoration":null},"roles":[],"premium_since":null,"pending":false,"nick":null,"mute":false,"joined_at":"2023-01-01T00:00:00.000000+00:00","flags":0,"deaf":false,"communication_disabled_until":null,"avatar":null}"#;
		let member: Member = serde_json::from_str(json).unwrap();
	}
}
