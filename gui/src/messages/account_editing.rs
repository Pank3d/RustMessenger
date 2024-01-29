use super::ProjectTootMessage;

#[derive(Clone, Debug)]
pub(crate) enum AccountEditingMessage {
	UpdateName(String),
	End(bool),
	Copy,
}

impl From<AccountEditingMessage> for ProjectTootMessage {
	fn from(value: AccountEditingMessage) -> Self {
		Self::AccountEditing(value)
	}
}
