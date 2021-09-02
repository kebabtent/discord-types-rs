use crate::{
	AllowedMentions, ApplicationCommandOption, Component, CowString, Embed, InteractionResponseType,
};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct CreateCommand<'a> {
	pub name: &'a str,
	pub description: &'a str,
	pub options: Vec<ApplicationCommandOption>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Attachment {
	pub name: CowString,
	pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateMessage {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content: Option<CowString>,
	// tts
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub embeds: Vec<Embed>,
	// allowed_mentions
	// message_reference
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub components: Vec<Component>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EditMessage {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub content: Option<CowString>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub embeds: Option<Vec<Embed>>,
	// flags
	// allowed_mentions
	// attachments
	#[serde(skip_serializing_if = "Option::is_none")]
	pub components: Option<Vec<Component>>,
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
	#[serde(skip_serializing_if = "Option::is_none")]
	pub embeds: Option<Vec<Embed>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub allowed_mentions: Option<AllowedMentions>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub flags: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub components: Option<Vec<Component>>,
}

impl Default for InteractionCallbackData<'_> {
	fn default() -> Self {
		Self {
			tts: false,
			content: None,
			embeds: None,
			allowed_mentions: None,
			flags: None,
			components: None,
		}
	}
}
