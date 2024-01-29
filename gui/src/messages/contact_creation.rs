use super::ProjectTootMessage;

#[derive(Clone, Debug)]
pub(crate) enum ContactCreationMessage {
	UpdateAddress(String),
	UpdateName(String),
	End(bool),
}

impl From<ContactCreationMessage> for ProjectTootMessage {
	fn from(value: ContactCreationMessage) -> Self {
		Self::ContactCreation(value)
	}
}
