use {crate::states::Contact, tokio_rusqlite::Connection};

pub(crate) type InsertContactTaskError = tokio_rusqlite::Error;

pub(crate) async fn run(
	database: Connection,
	contact: Contact,
) -> Result<(), InsertContactTaskError> {
	database
		.call(move |conn| {
			conn.prepare(include_str!("../../sql/insert-contact.sql"))?
				.execute([
					bs58::encode(contact.address.as_bytes()).into_string(),
					contact.name,
				])?;
			Ok(())
		})
		.await
}
