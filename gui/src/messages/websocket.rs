use {
	super::ProjectTootMessage,
	project_toot_sdk::{IMessage, IWebSocketRequest},
	tokio::sync::mpsc::Sender,
};

#[derive(Clone, Debug)]
pub(crate) enum WebSocketMessage {
	Connected(Sender<IWebSocketRequest>),
	Disconnected,
	Authorized,
	NewMessage(IMessage),
}

impl From<WebSocketMessage> for ProjectTootMessage {
	fn from(value: WebSocketMessage) -> Self {
		Self::WebSocket(value)
	}
}
