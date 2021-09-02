# discord_types

Y = Yes, P = Partial

| Event                      | Impl? |
| -------------------------- |:-----:|
| Hello                      | Y     |
| Ready                      | Y     |
| Resumed                    | Y     |
| Reconnect                  | Y     |
| InvalidSession             | Y     |
| ApplicationCommandCreate   | Y     |
| ApplicationCommandUpdate   | Y     |
| ApplicationCommandDelete   | Y     |
| ChannelCreate              | Y     |
| ChannelUpdate              | Y     |
| ChannelDelete              | Y     |
| ChannelPinsUpdate          |       |
| GuildCreate                | Y     |
| GuildUpdate                | Y     |
| GuildDelete                | Y     |
| GuildBanAdd                |       |
| GuildBanRemove             |       |
| GuildEmojisUpdate          |       |
| GuildIntegrationsUpdate    |       |
| GuildMemberAdd             | Y     |
| GuildMemberRemove          | Y     |
| GuildMemberUpdate          | Y     |
| GuildMembersChunk          | Y     |
| GuildRoleCreate            | Y     |
| GuildRoleUpdate            | Y     |
| GuildRoleDelete            | Y     |
| InteractionCreate          | Y     |
| InviteCreate               |       |
| InviteDelete               |       |
| MessageCreate              | Y     |
| MessageUpdate              | P (?) |
| MessageDelete              | Y     |
| MessageDeleteBulk          |       |
| MessageReactionAdd         |       |
| MessageReactionRemove      |       |
| MessageReactionRemoveAll   |       |
| MessageReactionRemoveEmoji |       |
| PresenceUpdate             |       |
| TypingStart                |       |
| UserUpdate                 |       |
| VoiceStateUpdate           |       |
| VoiceServerUpdate          |       |
| WebhooksUpdate             |       |

| Command             | Impl? |
| ------------------- |:-----:|
| Identify            | Y     |
| Resume              | Y     |
| Heartbeat           | Y     |
| RequestGuildMembers | Y     |
| UpdateVoiceState    | Y     |
| UpdateStatus        | P     |

| Structs            | Impl? |
| ------------------ |:-----:|
| Snowflake          | Y     |
| Application        | P     |
| Channel            | P     |
| Guild              | P     |
| Role               | P     |
| Member             | Y     |
| User               | Y     |
| Message            | P     |
| ApplicationCommand | Y     |
| Interaction        | P     |
| Component          | P     |

| Enums         | Impl? |
| ------------- |:-----:|
| Intents       | Y     |
| Status        | Y     |
| ChannelType   | Y     |
| UserFlags     | Y     |
| MessageType   | Y     |
| ComponentType | Y     |
| ButtonStyle   | Y     |