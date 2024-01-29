use super::ProjectTootMessage;

#[derive(Clone, Debug)]
pub(crate) enum AccountSpecifyMessage {
	Create,
	Edit(usize),
	Delete(usize),
	Choose(usize),
}

impl From<AccountSpecifyMessage> for ProjectTootMessage {
	fn from(value: AccountSpecifyMessage) -> Self {
		Self::AccountSpecify(value)
	}
}
