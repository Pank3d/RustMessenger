use {crate::states::Message, tokio_rusqlite::Connection};

pub(crate) type InsertMessageTaskError = tokio_rusqlite::Error;

pub(crate) async fn run(
	database: Connection,
	message: Message,
) -> Result<(), InsertMessageTaskError> {
	database
		.call(move |conn| {
			let res = conn
				.prepare(include_str!("../../sql/get-message.sql"))?
				.query_row((bs58::encode(message.hash).into_string(),), |_| Ok(()));

			if let Err(rusqlite::Error::QueryReturnedNoRows) = res {
				conn.prepare(include_str!("../../sql/insert-message.sql"))?
					.execute((
						bs58::encode(message.hash).into_string(),
						bs58::encode(message.sender.as_bytes()).into_string(),
						bs58::encode(message.receiver.as_bytes()).into_string(),
						bs58::encode(message.data_hash).into_string(),
						message.timestamp,
						message.success,
					))?;

				Ok(())
			} else {
				Ok(res?)
			}
		})
		.await
}
