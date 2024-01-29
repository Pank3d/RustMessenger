use {
	ed25519_dalek::{SigningKey, VerifyingKey},
	project_toot_sdk::Sha512Data,
};

#[derive(Clone, Debug)]
pub(crate) struct Account {
	pub secret: SigningKey,
	pub address: VerifyingKey,
	pub name: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Contact {
	pub address: VerifyingKey,
	pub name: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Message {
	pub hash: Sha512Data,
	pub sender: VerifyingKey,
	pub receiver: VerifyingKey,
	pub data_hash: Sha512Data,
	pub timestamp: i64,
	pub success: bool,
	pub content: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Data {
	pub accounts: Vec<Account>,
	pub contacts: Vec<Contact>,
	pub messages: Vec<Message>,
}
