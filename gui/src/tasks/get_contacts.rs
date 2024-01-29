use {crate::states::Contact, ed25519_dalek::VerifyingKey, tokio_rusqlite::Connection};

pub(crate) type GetContactsTaskError = tokio_rusqlite::Error;

pub(crate) async fn run(database: Connection) -> Result<Vec<Contact>, GetContactsTaskError> {
	database
		.call(|conn| {
			Ok(conn
				.prepare(include_str!("../../sql/get-contacts.sql"))?
				.query_map([], |row| {
					Ok(Contact {
						address: VerifyingKey::from_bytes(
							&bs58::decode(row.get::<_, String>(0)?)
								.into_vec()
								.unwrap()
								.try_into()
								.unwrap(),
						)
						.or_else(|err| Err(rusqlite::Error::UserFunctionError(Box::new(err))))?,
						name: row.get(1)?,
					})
				})?
				.filter(|x| x.is_ok())
				.map(|x| x.unwrap())
				.collect::<Vec<_>>())
		})
		.await
}
