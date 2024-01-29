use {
	crate::utils,
	std::path::PathBuf,
	tokio::{fs, io::AsyncWriteExt},
};

pub(crate) type WriteDataTaskError = std::io::Error;

pub(crate) async fn run(
	basedir: PathBuf,
	hash: impl AsRef<[u8]>,
	data: impl AsRef<[u8]>,
) -> Result<(), WriteDataTaskError> {
	let hash = hex::encode(hash.as_ref());
	let dir = utils::data_path(basedir, hash.clone());
	fs::create_dir_all(dir.clone()).await?;
	fs::File::create(dir.join(hash).with_extension("dat"))
		.await?
		.write_all(data.as_ref())
		.await?;

	Ok(())
}
