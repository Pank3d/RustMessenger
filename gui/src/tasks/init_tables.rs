use tokio_rusqlite::Connection;

pub(crate) type InitTablesTaskError = tokio_rusqlite::Error;

pub(crate) async fn run(database: Connection) -> Result<(), InitTablesTaskError> {
	database
		.call(|conn| {
			conn.execute_batch(include_str!("../../sql/tables.sql"))?;
			Ok(())
		})
		.await
}
