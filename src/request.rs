use crate::{AllowedMentions, ApplicationCommandOption, Embed, InteractionResponseType};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct CreateCommand<'a> {
	pub name: &'a str,
	pub description: &'a str,
	pub options: Vec<ApplicationCommandOption>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Attachment {
	pub name: String,
	pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateMessage {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content: Option<String>,
	// nonce
	// tts
	#[serde(skip_serializing_if = "Option::is_none")]
	pub embed: Option<Embed>,
	// allowed_mentions
	// message_reference
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateGuildBan<'a> {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub delete_message_days: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub reason: Option<&'a str>,
}

#[derive(Clone, Debug, Serialize)]
pub struct InteractionResponse<'a> {
	#[serde(rename = "type")]
	pub response_type: InteractionResponseType,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub data: Option<InteractionCallbackData<'a>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct InteractionCallbackData<'a> {
	pub tts: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content: Option<&'a str>,
	pub embeds: Vec<Embed>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub allowed_mentions: Option<AllowedMentions>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub flags: Option<u32>,
}

impl Default for InteractionCallbackData<'_> {
	fn default() -> Self {
		Self {
			tts: false,
			content: None,
			embeds: Vec::new(),
			allowed_mentions: None,
			flags: None,
		}
	}
}
