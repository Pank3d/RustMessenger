use {
	crate::{Salt, Sha512Data},
	borsh::{BorshDeserialize, BorshSerialize},
	ed25519_dalek::{Signature, VerifyingKey},
};

mod verifying_key {
	use {
		borsh::{
			io::{self, Error, ErrorKind, Read, Write},
			BorshDeserialize,
			BorshSerialize,
		},
		ed25519_dalek::VerifyingKey,
	};

	pub fn serialize<W: Write>(obj: &VerifyingKey, writer: &mut W) -> io::Result<()> {
		obj.to_bytes().serialize(writer)
	}

	pub fn deserialize<R: Read>(reader: &mut R) -> io::Result<VerifyingKey> {
		Ok(
			VerifyingKey::from_bytes(&<[u8; 32]>::deserialize_reader(reader)?)
				.or_else(|err| Err(Error::new(ErrorKind::InvalidData, Box::new(err))))?,
		)
	}

	pub fn serialize_option<W: Write>(
		obj: &Option<VerifyingKey>,
		writer: &mut W,
	) -> io::Result<()> {
		match obj {
			None => 0u8.serialize(writer),
			Some(inner) => {
				1u8.serialize(writer)?;
				serialize(inner, writer)
			},
		}
	}

	pub fn deserialize_option<R: Read>(reader: &mut R) -> io::Result<Option<VerifyingKey>> {
		match u8::deserialize_reader(reader)? {
			0 => Ok(None),
			1 => Ok(Some(deserialize(reader)?)),
			flag => Err(Error::new(
				ErrorKind::InvalidData,
				format!(
					"Invalid Option representation: {}. The first byte must be 0 or 1",
					flag
				),
			)),
		}
	}
}

mod signature {
	use {
		borsh::{
			io::{self, Read, Write},
			BorshDeserialize,
			BorshSerialize,
		},
		ed25519_dalek::Signature,
	};

	pub fn serialize<W: Write>(obj: &Signature, writer: &mut W) -> io::Result<()> {
		obj.to_bytes().serialize(writer)
	}

	pub fn deserialize<R: Read>(reader: &mut R) -> io::Result<Signature> {
		Ok(Signature::from_bytes(&<[u8; 64]>::deserialize_reader(
			reader,
		)?))
	}
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct ISigner {
	#[borsh(
		serialize_with = "verifying_key::serialize",
		deserialize_with = "verifying_key::deserialize"
	)]
	pub address: VerifyingKey,
	pub salt: Salt,
	pub timestamp: i64,
	#[borsh(
		serialize_with = "signature::serialize",
		deserialize_with = "signature::deserialize"
	)]
	pub signature: Signature,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct IMessage {
	pub hash: Sha512Data,
	#[borsh(
		serialize_with = "verifying_key::serialize",
		deserialize_with = "verifying_key::deserialize"
	)]
	pub sender: VerifyingKey,
	#[borsh(
		serialize_with = "verifying_key::serialize",
		deserialize_with = "verifying_key::deserialize"
	)]
	pub receiver: VerifyingKey,
	pub data_hash: Sha512Data,
	pub timestamp: i64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct ISendMessagePayload {
	#[borsh(
		serialize_with = "verifying_key::serialize",
		deserialize_with = "verifying_key::deserialize"
	)]
	pub receiver: VerifyingKey,
	pub data: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct IGetMessagesPayload {
	pub offset: u32,
	pub limit: u32,
	#[borsh(
		serialize_with = "verifying_key::serialize_option",
		deserialize_with = "verifying_key::deserialize_option"
	)]
	pub with: Option<VerifyingKey>,
	pub mine: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct IDeleteMessagesPayload {
	pub hashes: Vec<Sha512Data>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub enum IRPCRequest {
	SendMessage(ISigner, ISendMessagePayload),
	GetMessages(ISigner, IGetMessagesPayload),
	DeleteMessages(ISigner, IDeleteMessagesPayload),
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub enum IRPCResponse {
	SendMessage,
	GetMessages(Vec<IMessage>),
	DeleteMessages(Vec<bool>),
}

pub mod irpcerror {
	pub const FILE_SYSTEM_WRITE_ERROR: i32 = -10;
	pub const DATABASE_POOL_ERROR: i32 = -20;
	pub const DATABASE_READ_ERROR: i32 = -21;
	pub const DATABASE_WRITE_ERROR: i32 = -22;
	pub const DATABASE_DELETE_ERROR: i32 = -23;
	pub const DESERIALIZATION_ERROR: i32 = -10000;
	pub const MAXIMUM_CONFIRMATION_TIMESTAMP_OFFSET_EXCCEED: i32 = -10100;
	pub const INVALID_SIGNATURE: i32 = -10101;
	pub const LIMIT_DONT_FIT_RANGE: i32 = -10200;
	pub const HASHES_LEN_DONT_FIT_RANGE: i32 = -10201;

	pub fn to_string(code: i32) -> String {
		match code {
			FILE_SYSTEM_WRITE_ERROR => "Internal error: Failed to write file".to_string(),
			DATABASE_POOL_ERROR => "Internal error: Failed to get database pool".to_string(),
			DATABASE_READ_ERROR => "Internal error: Failed to read from database".to_string(),
			DATABASE_WRITE_ERROR => "Internal error: Failed to write to database".to_string(),
			DATABASE_DELETE_ERROR => "Internal error: Failed to delete from database".to_string(),
			DESERIALIZATION_ERROR => "Failed to deserialize data".to_string(),
			MAXIMUM_CONFIRMATION_TIMESTAMP_OFFSET_EXCCEED => {
				"Maximum confirmation timestamp offset excceed".to_string()
			},
			INVALID_SIGNATURE => "Invalid signature".to_string(),
			LIMIT_DONT_FIT_RANGE => "Limit don't fit range".to_string(),
			HASHES_LEN_DONT_FIT_RANGE => "Hashes vector length don't fit range".to_string(),
			unk => format!("Unknown error: {}", unk),
		}
	}
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub enum IWebSocketRequest {
	Authorize(ISigner),
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub enum IWebSocketResponse {
	Authorized,
	NewMessage(IMessage),
}
