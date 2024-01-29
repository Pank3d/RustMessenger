use {crate::states::Account, tokio_rusqlite::Connection};

pub(crate) type InsertAccountTaskError = tokio_rusqlite::Error;

pub(crate) async fn run(
	database: Connection,
	account: Account,
) -> Result<(), InsertAccountTaskError> {
	database
		.call(move |conn| {
			conn.prepare(include_str!("../../sql/insert-account.sql"))?
				.execute([
					bs58::encode(account.secret.as_bytes()).into_string(),
					bs58::encode(account.secret.verifying_key().as_bytes()).into_string(),
					account.name,
				])?;
			Ok(())
		})
		.await
}
