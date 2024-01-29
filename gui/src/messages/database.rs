use {
	super::ProjectTootMessage,
	crate::states::{Account, Contact, Message},
	tokio_rusqlite::Connection,
};

#[derive(Clone, Debug)]
pub(crate) enum RusqliteMessage {
	Connected(Connection),
	LoadedAccounts(Vec<Account>),
	LoadedContacts(Vec<Contact>),
	LoadedMessages(Vec<Message>),
}

impl From<RusqliteMessage> for ProjectTootMessage {
	fn from(value: RusqliteMessage) -> Self {
		Self::Database(value)
	}
}
