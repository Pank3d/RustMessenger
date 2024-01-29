use {
	crate::{
		cdn::CDN_PATH,
		states::{AppState, Borsh},
	},
	axum::{
		extract::State,
		response::{IntoResponse, Response},
	},
	chrono::{TimeZone, Utc},
	ed25519_dalek::VerifyingKey,
	project_toot_sdk::{
		irpcerror,
		IDeleteMessagesPayload,
		IGetMessagesPayload,
		IMessage,
		IRPCRequest,
		IRPCResponse,
		ISendMessagePayload,
		ISigner,
		IWebSocketResponse,
		Sha512Data,
		MAXIMUM_CONFIRMATION_TIMESTAMP_OFFSET,
	},
	sha2::{Digest, Sha512},
	std::path::PathBuf,
	tokio::fs,
};

pub(crate) async fn send_message(
	state: AppState,
	ISigner {
		address: sender,
		salt,
		timestamp,
		signature,
	}: ISigner,
	ISendMessagePayload { receiver, data }: ISendMessagePayload,
) -> Result<IRPCResponse, i32> {
	let now = Utc::now();
	let datetime = Utc
		.timestamp_micros(timestamp)
		.single()
		.ok_or(irpcerror::DESERIALIZATION_ERROR)?;
	if datetime < (now - MAXIMUM_CONFIRMATION_TIMESTAMP_OFFSET) || datetime > now {
		return Err(irpcerror::MAXIMUM_CONFIRMATION_TIMESTAMP_OFFSET_EXCCEED)
	}

	let hash = {
		let mut hasher: Sha512 = Digest::new();
		hasher.update(b"send-message");
		hasher.update(&salt);
		hasher.update(timestamp.to_le_bytes());

		sender
			.verify_prehashed_strict(hasher.clone(), None, &signature)
			.and_then(|_| Ok(hasher.finalize()))
			.or(Err(irpcerror::INVALID_SIGNATURE))?
	};

	let data_hash = {
		let mut hasher: Sha512 = Digest::new();
		hasher.update(&data);
		hasher.finalize()
	};

	fs::write(
		PathBuf::from(CDN_PATH)
			.join(hex::encode(data_hash.clone()))
			.with_extension("dat"),
		data,
	)
	.await
	.or(Err(irpcerror::FILE_SYSTEM_WRITE_ERROR))?;

	state
		.pool
		.get()
		.await
		.or(Err(irpcerror::DATABASE_POOL_ERROR))?
		.execute(
			include_str!("../sql/send-message.sql"),
			&[
				&bs58::encode(hash).into_string(),
				&bs58::encode(sender).into_string(),
				&bs58::encode(receiver).into_string(),
				&bs58::encode(data_hash).into_string(),
				&now.naive_utc(),
			],
		)
		.await
		.or(Err(irpcerror::DATABASE_WRITE_ERROR))?;

	if let Some(tx) = state.websockets.read().await.get(receiver.as_bytes()) {
		let _ = tx.send(IWebSocketResponse::NewMessage(IMessage {
			hash: hash.into(),
			sender,
			receiver,
			data_hash: data_hash.into(),
			timestamp,
		}));
	}

	Ok(IRPCResponse::SendMessage)
}

pub(crate) async fn get_messages(
	state: AppState,
	ISigner {
		address: sender,
		salt,
		timestamp,
		signature,
	}: ISigner,
	IGetMessagesPayload {
		offset,
		limit,
		with,
		mine,
	}: IGetMessagesPayload,
) -> Result<IRPCResponse, i32> {
	let now = Utc::now();
	let datetime = Utc
		.timestamp_micros(timestamp)
		.single()
		.ok_or(irpcerror::DESERIALIZATION_ERROR)?;
	if datetime < (now - MAXIMUM_CONFIRMATION_TIMESTAMP_OFFSET) || datetime > now {
		return Err(irpcerror::MAXIMUM_CONFIRMATION_TIMESTAMP_OFFSET_EXCCEED)
	}

	if !(1..=10_000).contains(&limit) {
		return Err(irpcerror::LIMIT_DONT_FIT_RANGE)
	}

	{
		let mut hasher: Sha512 = Digest::new();
		hasher.update(b"get-messages");
		hasher.update(&salt);
		hasher.update(timestamp.to_le_bytes());

		sender
			.verify_prehashed_strict(hasher.clone(), None, &signature)
			.or(Err(irpcerror::INVALID_SIGNATURE))?
	}

	Ok(IRPCResponse::GetMessages({
		let database = state
			.pool
			.get()
			.await
			.or(Err(irpcerror::DATABASE_POOL_ERROR))?;
		if let Some(second) = with {
			database
				.query(
					include_str!("../sql/get-messages-with.sql"),
					&[
						&(mine as i32),
						&bs58::encode(sender).into_string(),
						&bs58::encode(second).into_string(),
						&(offset as i64),
						&(limit as i64),
					],
				)
				.await
		} else {
			database
				.query(
					include_str!("../sql/get-messages-all.sql"),
					&[
						&(mine as i32),
						&bs58::encode(sender).into_string(),
						&(offset as i64),
						&(limit as i64),
					],
				)
				.await
		}
		.or(Err(irpcerror::DATABASE_READ_ERROR))?
		.into_iter()
		.map(|x| IMessage {
			hash: bs58::decode(x.get::<_, &str>(0))
				.into_vec()
				.unwrap()
				.try_into()
				.unwrap(),
			sender: VerifyingKey::try_from(
				bs58::decode(x.get::<_, &str>(1))
					.into_vec()
					.unwrap()
					.as_slice(),
			)
			.unwrap(),
			receiver: VerifyingKey::try_from(
				bs58::decode(x.get::<_, &str>(2))
					.into_vec()
					.unwrap()
					.as_slice(),
			)
			.unwrap(),
			data_hash: bs58::decode(x.get::<_, &str>(3))
				.into_vec()
				.unwrap()
				.try_into()
				.unwrap(),
			timestamp: x.get::<_, chrono::NaiveDateTime>(4).timestamp_micros(),
		})
		.collect::<Vec<_>>()
	}))
}

pub(crate) async fn delete_messages(
	state: AppState,
	ISigner {
		address: sender,
		salt,
		timestamp,
		signature,
	}: ISigner,
	IDeleteMessagesPayload { hashes }: IDeleteMessagesPayload,
) -> Result<IRPCResponse, i32> {
	let now = Utc::now();
	let datetime = Utc
		.timestamp_micros(timestamp)
		.single()
		.ok_or(irpcerror::DESERIALIZATION_ERROR)?;
	if datetime < (now - MAXIMUM_CONFIRMATION_TIMESTAMP_OFFSET) || datetime > now {
		return Err(irpcerror::MAXIMUM_CONFIRMATION_TIMESTAMP_OFFSET_EXCCEED)
	}

	if !(1..10_000).contains(&hashes.len()) {
		return Err(irpcerror::HASHES_LEN_DONT_FIT_RANGE)
	}

	{
		let mut hasher: Sha512 = Digest::new();
		hasher.update(b"delete-messages");
		hasher.update(&salt);
		hasher.update(timestamp.to_le_bytes());

		sender
			.verify_prehashed_strict(hasher.clone(), None, &signature)
			.or(Err(irpcerror::INVALID_SIGNATURE))?
	}

	let deleted = state
		.pool
		.get()
		.await
		.or(Err(irpcerror::DATABASE_POOL_ERROR))?
		.query(
			include_str!("../sql/delete-messages.sql"),
			&[
				&bs58::encode(sender).into_string(),
				&hashes
					.iter()
					.map(|x| bs58::encode(x).into_string())
					.collect::<Vec<_>>(),
			],
		)
		.await
		.or(Err(irpcerror::DATABASE_DELETE_ERROR))?
		.into_iter()
		.map(|row| {
			bs58::decode(row.get::<_, String>(0))
				.into_vec()
				.unwrap()
				.try_into()
				.unwrap()
		})
		.collect::<Vec<Sha512Data>>();

	Ok(IRPCResponse::DeleteMessages(
		hashes
			.into_iter()
			.map(|x| deleted.contains(&x))
			.collect::<Vec<_>>(),
	))
}

pub(crate) async fn handler(
	State(state): State<AppState>,
	Borsh((id, data)): Borsh<(u128, IRPCRequest)>,
) -> Response {
	let res: Result<IRPCResponse, i32> = match data {
		IRPCRequest::SendMessage(signer, payload) => send_message(state, signer, payload).await,
		IRPCRequest::GetMessages(signer, payload) => get_messages(state, signer, payload).await,
		IRPCRequest::DeleteMessages(signer, payload) => {
			delete_messages(state, signer, payload).await
		},
	};

	Borsh((id, res)).into_response()
}
