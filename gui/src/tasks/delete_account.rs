use {ed25519_dalek::VerifyingKey, tokio_rusqlite::Connection};

pub(crate) type DeleteAccountTaskError = tokio_rusqlite::Error;

pub(crate) async fn run(
	database: Connection,
	address: VerifyingKey,
) -> Result<(), DeleteAccountTaskError> {
	database
		.call(move |conn| {
			conn.prepare(include_str!("../../sql/delete-account.sql"))?
				.execute([bs58::encode(address.as_bytes()).into_string()])?;
			Ok(())
		})
		.await
}
