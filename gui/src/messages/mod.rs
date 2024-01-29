mod account_editing;
mod account_specify;
mod chat;
mod contact_creation;
mod database;
mod websocket;
pub(crate) use {
	account_editing::AccountEditingMessage,
	account_specify::AccountSpecifyMessage,
	chat::ChatMessage,
	contact_creation::ContactCreationMessage,
	database::RusqliteMessage,
	websocket::WebSocketMessage,
};

#[derive(Clone, Debug, Default)]
pub(crate) enum ProjectTootMessage {
	#[default]
	None,
	AccountSpecify(AccountSpecifyMessage),
	AccountEditing(AccountEditingMessage),
	Chat(ChatMessage),
	ContactCreation(ContactCreationMessage),
	Database(RusqliteMessage),
	WebSocket(WebSocketMessage),
	NewMessage(project_toot_sdk::IMessage, Vec<u8>, bool),
}
