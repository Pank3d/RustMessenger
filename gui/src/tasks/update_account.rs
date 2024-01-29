use {ed25519_dalek::VerifyingKey, tokio_rusqlite::Connection};

pub(crate) type UpdateAccountTaskError = tokio_rusqlite::Error;

pub(crate) async fn run(
	database: Connection,
	address: VerifyingKey,
	name: String,
) -> Result<(), UpdateAccountTaskError> {
	database
		.call(move |conn| {
			conn.prepare(include_str!("../../sql/update-account.sql"))?
				.execute([bs58::encode(address.as_bytes()).into_string(), name])?;
			Ok(())
		})
		.await
}
