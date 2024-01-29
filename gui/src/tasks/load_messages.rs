use {
	crate::{
		client::{ClientError, ProjectTootClient},
		states::Account,
		utils,
	},
	project_toot_sdk::{
		IDeleteMessagesPayload,
		IGetMessagesPayload,
		IMessage,
		ISigner,
		Salt,
		Sha512Data,
	},
	sha2::{Digest, Sha512},
	std::path::PathBuf,
	tokio::{fs, io::AsyncWriteExt},
	tokio_rusqlite::Connection,
	url::ParseError as UrlParseError,
};

#[derive(Debug)]
pub(crate) enum LoadMessagesTaskError {
	UrlError(UrlParseError),
	IoError(std::io::Error),
	ReqwestError(reqwest::Error),
	TokioRusqliteError(tokio_rusqlite::Error),
	ClientError(ClientError),
}

impl From<UrlParseError> for LoadMessagesTaskError {
	fn from(value: UrlParseError) -> Self {
		Self::UrlError(value)
	}
}

impl From<std::io::Error> for LoadMessagesTaskError {
	fn from(value: std::io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<reqwest::Error> for LoadMessagesTaskError {
	fn from(value: reqwest::Error) -> Self {
		Self::ReqwestError(value)
	}
}

impl From<tokio_rusqlite::Error> for LoadMessagesTaskError {
	fn from(value: tokio_rusqlite::Error) -> Self {
		Self::TokioRusqliteError(value)
	}
}

impl From<ClientError> for LoadMessagesTaskError {
	fn from(value: ClientError) -> Self {
		Self::ClientError(value)
	}
}

pub(crate) async fn run(
	basedir: PathBuf,
	client: ProjectTootClient,
	database: Connection,
	account: Account,
) -> Result<(), LoadMessagesTaskError> {
	let mut messages = Vec::<IMessage>::new();
	loop {
		let salt = rand::random::<Salt>();
		let timestamp = chrono::Utc::now().timestamp_micros();
		let ret = client
			.clone()
			.get_messages(
				ISigner {
					address: account.address,
					salt,
					timestamp,
					signature: {
						let mut hasher: Sha512 = Digest::new();
						hasher.update(b"get-messages");
						hasher.update(salt);
						hasher.update(timestamp.to_le_bytes());
						account.secret.sign_prehashed(hasher, None).unwrap()
					},
				},
				IGetMessagesPayload {
					offset: 0,
					limit: 10_000,
					with: None,
					mine: false,
				},
			)
			.await?;
		let done = ret.len() < 10_000;
		messages.extend(ret);

		if done {
			break;
		}
	}

	let messages_cloned = messages.clone();
	let found = database
		.call(move |conn| {
			let mut ret = Vec::<Sha512Data>::new();
			let mut stmt = conn.prepare(include_str!("../../sql/get-existing-messages.sql"))?;

			for hash in messages_cloned.iter().map(|x| x.hash) {
				if stmt
					.query_row([bs58::encode(hash).into_string()], |_| Ok(()))
					.is_ok()
				{
					ret.push(hash);
				}
			}

			Ok(ret)
		})
		.await?;

	let messages_cloned = messages.clone();
	database
		.call(move |conn| {
			let mut stmt = conn.prepare(include_str!("../../sql/insert-message.sql"))?;

			for message in messages_cloned
				.iter()
				.filter(|x| !found.iter().any(|y| y == &x.hash))
			{
				stmt.execute((
					bs58::encode(message.hash).into_string(),
					bs58::encode(message.sender.as_bytes()).into_string(),
					bs58::encode(message.receiver.as_bytes()).into_string(),
					bs58::encode(message.data_hash).into_string(),
					message.timestamp,
					true,
				))?;
			}

			Ok(())
		})
		.await?;

	let baseurl = client.baseurl().join("cdn/")?;

	for msg in messages.iter() {
		let hash = hex::encode(msg.data_hash);
		let url = baseurl.join(&(hash.clone() + ".dat"))?;
		let resp = &mut client.http().get(url).send().await?;

		if resp.status().is_success() {
			let dirpath = utils::data_path(basedir.clone(), hex::encode(msg.data_hash));
			fs::create_dir_all(&dirpath).await?;
			let file = &mut fs::File::create(dirpath.join(hash + ".dat")).await?;

			while let Some(bytes) = resp.chunk().await? {
				file.write(&bytes).await?;
			}

			file.flush().await?;
		}
	}

	for chunk in messages.chunks(10_000) {
		let salt = rand::random::<Salt>();
		let timestamp = chrono::Utc::now().timestamp_micros();
		client
			.clone()
			.delete_messages(
				ISigner {
					address: account.address,
					salt,
					timestamp,
					signature: {
						let mut hasher: Sha512 = Digest::new();
						hasher.update(b"delete-messages");
						hasher.update(salt);
						hasher.update(timestamp.to_le_bytes());
						account.secret.sign_prehashed(hasher, None).unwrap()
					},
				},
				IDeleteMessagesPayload {
					hashes: chunk.into_iter().map(|x| x.hash).collect::<Vec<_>>(),
				},
			)
			.await?;
	}

	Ok(())
}
