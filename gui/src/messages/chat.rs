use super::ProjectTootMessage;

#[derive(Clone, Debug)]
pub(crate) enum ChatMessage {
	CreateContact,
	DeleteContact(usize),
	ChooseContact(usize),
	UpdateMessage(String),
	SendMessage,
}

impl From<ChatMessage> for ProjectTootMessage {
	fn from(value: ChatMessage) -> Self {
		Self::Chat(value)
	}
}
