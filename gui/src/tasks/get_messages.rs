use {
	crate::{
		states::{Account, Contact, Message},
		utils,
	},
	ed25519_dalek::VerifyingKey,
	std::path::PathBuf,
	tokio_rusqlite::Connection,
};

pub(crate) type GetMessagesTaskError = tokio_rusqlite::Error;

pub(crate) async fn run(
	basedir: PathBuf,
	account: Account,
	contact: Contact,
	database: Connection,
) -> Result<Vec<Message>, GetMessagesTaskError> {
	let mut messages = database
		.call(move |conn| {
			Ok(conn
				.prepare(include_str!("../../sql/get-messages.sql"))?
				.query_map(
					[
						bs58::encode(account.address.as_bytes()).into_string(),
						bs58::encode(contact.address.as_bytes()).into_string(),
					],
					|row| {
						Ok(Message {
							hash: bs58::decode(row.get::<_, String>(0)?)
								.into_vec()
								.or_else(|err| {
									Err(rusqlite::Error::UserFunctionError(Box::new(err)))
								})?
								.try_into()
								.unwrap(),
							sender: VerifyingKey::from_bytes(
								&bs58::decode(row.get::<_, String>(1)?)
									.into_vec()
									.or_else(|err| {
										Err(rusqlite::Error::UserFunctionError(Box::new(err)))
									})?
									.try_into()
									.unwrap(),
							)
							.or_else(|err| {
								Err(rusqlite::Error::UserFunctionError(Box::new(err)))
							})?,
							receiver: VerifyingKey::from_bytes(
								&bs58::decode(row.get::<_, String>(2)?)
									.into_vec()
									.or_else(|err| {
										Err(rusqlite::Error::UserFunctionError(Box::new(err)))
									})?
									.try_into()
									.unwrap(),
							)
							.or_else(|err| {
								Err(rusqlite::Error::UserFunctionError(Box::new(err)))
							})?,
							data_hash: bs58::decode(row.get::<_, String>(3)?)
								.into_vec()
								.or_else(|err| {
									Err(rusqlite::Error::UserFunctionError(Box::new(err)))
								})?
								.try_into()
								.unwrap(),
							timestamp: row.get(4)?,
							success: row.get(5)?,
							content: String::new(),
						})
					},
				)?
				.filter(|x| x.is_ok())
				.map(|x| x.unwrap())
				.collect::<Vec<_>>())
		})
		.await?;

	for message in messages.iter_mut() {
		let res = super::read_data::run(basedir.clone(), message.data_hash).await;
		if let Ok(data) = res {
			message.content = utils::decrypt(
				&account,
				&if account.address == message.sender {
					message.receiver
				} else {
					message.sender
				},
				&data,
			);
		} else {
			eprintln!("{:#?}", res.unwrap_err());
		}
	}

	Ok(messages)
}
