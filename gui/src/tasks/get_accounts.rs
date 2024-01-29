use {crate::states::Account, ed25519_dalek::SigningKey, tokio_rusqlite::Connection};

pub(crate) type GetAccountsTaskError = tokio_rusqlite::Error;

pub(crate) async fn run(database: Connection) -> Result<Vec<Account>, GetAccountsTaskError> {
	database
		.call(|conn| {
			Ok(conn
				.prepare(include_str!("../../sql/get-accounts.sql"))?
				.query_map([], |row| {
					let secret = SigningKey::from_bytes(
						&bs58::decode(row.get::<_, String>(0)?)
							.into_vec()
							.unwrap()
							.try_into()
							.unwrap(),
					);
					let address = secret.verifying_key();
					Ok(Account {
						secret,
						address,
						name: row.get(1)?,
					})
				})?
				.filter(|x| x.is_ok())
				.map(|x| x.unwrap())
				.collect::<Vec<_>>())
		})
		.await
}
